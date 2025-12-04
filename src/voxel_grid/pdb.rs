use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::OnceLock;

use regex::Regex;

use crate::voxel_grid::raster::Atom;

/// Filtering options analogous to the C++ flags.
#[derive(Debug, Clone, Default)]
pub struct Filters {
	pub exclude_water: bool,
	pub exclude_ions: bool,
	pub exclude_ligands: bool,
	pub exclude_hetatm: bool,
	pub exclude_nucleic_acids: bool,
	pub exclude_amino_acids: bool,
}

#[derive(Debug, Clone)]
pub struct PdbOptions {
	pub use_united: bool,
	pub filters: Filters,
}

impl Default for PdbOptions {
	fn default() -> Self {
		Self {
			use_united: true,
			filters: Filters::default(),
		}
	}
}

#[derive(Debug, Clone)]
struct RadiusEntry {
	explicit: f32,
	united: f32,
	explicit_text: String,
	united_text: String,
}

#[derive(Debug)]
struct PatternEntry {
	residue: Regex,
	atom: Regex,
	key: String,
}

#[derive(Debug)]
struct RadiusTable {
	patterns: Vec<PatternEntry>,
	radii: HashMap<String, RadiusEntry>,
}

static RADIUS_TABLE: OnceLock<RadiusTable> = OnceLock::new();

fn load_atmtypenumbers_text() -> &'static str {
	// Include the C++ table and slice out the R"ATM(... )ATM" payload.
	const RAW: &str = include_str!("../../vossvolvox-cpp/src/atmtypenumbers_data.h");
	let start = RAW
		.find("R\"ATM(")
		.map(|i| i + "R\"ATM(".len())
		.unwrap_or(0);
	let end = RAW[start..]
		.find(")ATM\"")
		.map(|i| start + i)
		.unwrap_or_else(|| RAW.len());
	&RAW[start..end]
}

fn parse_radius_table() -> RadiusTable {
	let text = load_atmtypenumbers_text();
	let mut radii: HashMap<String, RadiusEntry> = HashMap::new();
	let mut patterns: Vec<PatternEntry> = Vec::new();

	for raw_line in text.lines() {
		let line_no_comment = raw_line
			.split_once('#')
			.map(|(before, _)| before)
			.unwrap_or(raw_line)
			.trim();
		let line = line_no_comment;
		if line.is_empty() || line.starts_with('#') {
			continue;
		}
		let tokens: Vec<&str> = line.split_whitespace().collect();
		if tokens.is_empty() {
			continue;
		}

		if tokens[0] == "radius" {
			if tokens.len() < 4 {
				continue;
			}
			let key = tokens[1].to_string();
			let explicit_text = tokens[3].to_string();
			let explicit: f32 = tokens[3].parse().unwrap_or(0.01);
			let united_text = tokens
				.get(4)
				.map(|s| s.to_string())
				.unwrap_or_else(|| explicit_text.clone());
			let united: f32 = united_text.parse().unwrap_or(explicit);
			radii.insert(
				key,
				RadiusEntry {
					explicit,
					united,
					explicit_text,
					united_text,
				},
			);
			continue;
		}

		if tokens.len() < 3 {
			continue;
		}
		let mut residue_pattern = tokens[0];
		let atom_pattern = tokens[1].replace('_', " ");
		if residue_pattern == "*" {
			residue_pattern = ".*";
		}
		let residue_regex = format!("^{}$", residue_pattern);
		let atom_regex = format!("^{}$", atom_pattern);
		if let (Ok(r_res), Ok(r_atom)) = (Regex::new(&residue_regex), Regex::new(&atom_regex)) {
			patterns.push(PatternEntry {
				residue: r_res,
				atom: r_atom,
				key: tokens[2].to_string(),
			});
		}
	}

	RadiusTable { patterns, radii }
}

fn radius_table() -> &'static RadiusTable {
	RADIUS_TABLE.get_or_init(parse_radius_table)
}

fn trim(s: &str) -> &str {
	let start = s.find(|c: char| !c.is_whitespace()).unwrap_or(s.len());
	let end = s
		.rfind(|c: char| !c.is_whitespace())
		.map(|i| i + 1)
		.unwrap_or(start);
	&s[start..end]
}

fn to_upper(s: &str) -> String {
	s.chars()
		.map(|c| c.to_ascii_uppercase())
		.collect::<String>()
}

fn get_field(line: &str, start: usize, len: usize) -> &str {
	if line.len() <= start {
		return "";
	}
	let end = (start + len).min(line.len());
	&line[start..end]
}

fn normalize_atom_name(raw: &str) -> String {
	let mut chars: Vec<char> = raw.chars().collect();
	while chars.len() < 2 {
		chars.push(' ');
	}
	if chars.len() > 2 {
		chars.truncate(2);
	}
	let c0 = chars[0];
	let c1 = chars[1];
	let c0_upper = c0.to_ascii_uppercase();
	let c1_upper = c1.to_ascii_uppercase();
	let first_blank_digit = c0 == ' ' || c0.is_ascii_digit();
	let second_h_like = c1_upper == 'H' || c1_upper == 'D';
	if first_blank_digit && second_h_like {
		return "H".to_string();
	}
	let first_h = c0_upper == 'H';
	let second_g = c1_upper == 'G';
	if first_h && !second_g {
		return "H".to_string();
	}
	let mut trimmed = raw.trim().to_string();
	trimmed.retain(|c| c != ' ');
	trimmed
}

#[derive(Debug, Clone)]
struct AtomRecord {
	x: String,
	y: String,
	z: String,
	residue: String,
	atom: String,
	resnum: String,
	chain: String,
	element: String,
	record: String,
}

#[derive(Debug, Clone)]
struct ResidueInfo {
	name: String,
	#[allow(dead_code)]
	chain: String,
	#[allow(dead_code)]
	resnum: String,
	atom_count: usize,
	polymer_flag: bool,
	hetatm_only: bool,
	elements: HashSet<String>,
	is_water: bool,
	is_nucleic: bool,
	is_amino: bool,
	is_ion: bool,
	is_ligand: bool,
}

const WATER_RESIDUES: &[&str] = &[
	"HOH", "H2O", "DOD", "WAT", "SOL", "TIP", "TIP3", "TIP3P", "TIP4", "TIP4P", "TIP5P", "SPC",
	"OH2",
];
const AMINO_RESIDUES: &[&str] = &[
	"ALA", "ARG", "ASN", "ASP", "ASX", "CYS", "GLN", "GLU", "GLX", "GLY", "HIS", "HID", "HIE",
	"HIP", "HISN", "HISL", "ILE", "LEU", "LYS", "MET", "MSE", "PHE", "PRO", "SER", "THR", "TRP",
	"TYR", "VAL", "SEC", "PYL", "ASH", "GLH",
];
const NUCLEIC_RESIDUES: &[&str] = &[
	"A", "C", "G", "U", "I", "T", "DA", "DG", "DC", "DT", "DI", "ADE", "GUA", "CYT", "URI",
	"THY", "PSU", "OMC", "OMU", "OMG", "5IU", "H2U", "M2G", "7MG", "1MA", "1MG", "2MG",
];
const ION_RESIDUES: &[&str] = &[
	"NA", "K", "MG", "MN", "FE", "ZN", "CU", "CA", "CL", "BR", "I", "LI", "CO", "NI", "HG", "CD",
	"SR", "CS", "BA", "YB", "MO", "RU", "OS", "IR", "AU", "AG", "PT", "TI", "AL", "GA", "V", "W",
	"ZN2", "FE2",
];
const ION_ELEMENTS: &[&str] = &[
	"NA", "K", "MG", "MN", "FE", "ZN", "CU", "CA", "CL", "BR", "I", "LI", "CO", "NI", "HG", "CD",
	"SR", "CS", "BA", "YB", "MO", "RU", "OS", "IR", "AU", "AG", "PT", "TI", "AL", "GA", "V", "W",
];

fn looks_like_nucleic(name: &str) -> bool {
	if name.len() == 1 {
		return "ACGUIT".contains(name.chars().next().unwrap_or(' '));
	}
	if name.len() == 2 && name.starts_with('D') {
		return "ACGUIT".contains(name.chars().nth(1).unwrap_or(' '));
	}
	false
}

fn is_water(name: &str) -> bool {
	let upper = to_upper(name);
	if WATER_RESIDUES.contains(&upper.as_str()) {
		return true;
	}
	upper.starts_with("HOH") || upper.starts_with("TIP")
}

fn is_amino(name: &str) -> bool {
	AMINO_RESIDUES.contains(&to_upper(name).as_str())
}

fn is_nucleic(name: &str) -> bool {
	let upper = to_upper(name);
	NUCLEIC_RESIDUES.contains(&upper.as_str()) || looks_like_nucleic(&upper)
}

fn is_ion(info: &ResidueInfo) -> bool {
	let upper = to_upper(&info.name);
	if ION_RESIDUES.contains(&upper.as_str()) {
		return true;
	}
	if info.atom_count <= 1 {
		for el in &info.elements {
			if ION_ELEMENTS.contains(&el.as_str()) {
				return true;
			}
		}
		if ION_ELEMENTS.contains(&upper.as_str()) {
			return true;
		}
	}
	false
}

fn make_residue_key(atom: &AtomRecord) -> String {
	format!(
		"{}|{}|{}",
		to_upper(&atom.chain),
		atom.resnum,
		to_upper(&atom.residue)
	)
}

fn classify_residues(atoms: &[AtomRecord]) -> HashMap<String, ResidueInfo> {
	let mut residues: HashMap<String, ResidueInfo> = HashMap::new();
	for atom in atoms {
		let key = make_residue_key(atom);
		let entry = residues.entry(key).or_insert_with(|| ResidueInfo {
			name: atom.residue.clone(),
			chain: atom.chain.clone(),
			resnum: atom.resnum.clone(),
			atom_count: 0,
			polymer_flag: false,
			hetatm_only: true,
			elements: HashSet::new(),
			is_water: false,
			is_nucleic: false,
			is_amino: false,
			is_ion: false,
			is_ligand: false,
		});
		entry.atom_count += 1;
		if !atom.element.is_empty() {
			entry.elements.insert(to_upper(&atom.element));
		}
		if atom.record.to_ascii_uppercase() == "ATOM" {
			entry.polymer_flag = true;
		}
		if atom.record.to_ascii_uppercase() != "HETATM" {
			entry.hetatm_only = false;
		}
	}

	for info in residues.values_mut() {
		if is_amino(&info.name) || is_nucleic(&info.name) {
			info.polymer_flag = true;
		}
		info.is_water = is_water(&info.name);
		info.is_amino = is_amino(&info.name);
		info.is_nucleic = is_nucleic(&info.name);
		info.is_ion = is_ion(info);
		info.is_ligand = !info.polymer_flag && !info.is_water && !info.is_ion;
	}
	residues
}

fn should_filter(info: &ResidueInfo, filters: &Filters) -> bool {
	if filters.exclude_water && info.is_water {
		return true;
	}
	if filters.exclude_ions && info.is_ion {
		return true;
	}
	if filters.exclude_ligands && info.is_ligand {
		return true;
	}
	if filters.exclude_hetatm && info.hetatm_only {
		return true;
	}
	if filters.exclude_nucleic_acids && info.is_nucleic {
		return true;
	}
	if filters.exclude_amino_acids && info.is_amino {
		return true;
	}
	false
}

fn radius_for(residue: &str, atom: &str, use_united: bool) -> f32 {
	let table = radius_table();
	for entry in &table.patterns {
		if entry.residue.is_match(residue) && entry.atom.is_match(atom) {
			if let Some(r) = table.radii.get(&entry.key) {
				return if use_united { r.united } else { r.explicit };
			}
		}
	}
	0.01
}

fn radius_text_for(residue: &str, atom: &str, use_united: bool) -> String {
	let table = radius_table();
	for entry in &table.patterns {
		if entry.residue.is_match(residue) && entry.atom.is_match(atom) {
			if let Some(r) = table.radii.get(&entry.key) {
				return if use_united {
					r.united_text.clone()
				} else {
					r.explicit_text.clone()
				};
			}
		}
	}
	"0.01".to_string()
}

fn parse_float(s: &str) -> f32 {
	s.trim().parse::<f32>().unwrap_or(0.0)
}

/// Parse a PDB file into atoms with radii according to the embedded atmtypenumbers table.
pub fn load_atoms_from_pdb_path(path: &str, opts: &PdbOptions) -> io::Result<Vec<Atom>> {
	let file = File::open(path)?;
	let reader = BufReader::new(file);
	load_atoms_from_reader(reader, opts)
}

pub fn load_atoms_from_reader<R: BufRead>(
	reader: R,
	opts: &PdbOptions,
) -> io::Result<Vec<Atom>> {
	let atoms = parse_atom_records(reader)?;

	let residue_map = classify_residues(&atoms);
	let mut out: Vec<Atom> = Vec::new();
	for rec in atoms {
		let key = make_residue_key(&rec);
		if let Some(info) = residue_map.get(&key) {
			if should_filter(info, &opts.filters) {
				continue;
			}
		}
		let radius = radius_for(&rec.residue, &rec.atom, opts.use_united);
		out.push(Atom {
			x: parse_float(&rec.x),
			y: parse_float(&rec.y),
			z: parse_float(&rec.z),
			radius,
		});
	}

	Ok(out)
}

/// Write XYZR lines to writer. Returns number of atoms written.
pub fn write_xyzr_from_path(path: &str, opts: &PdbOptions, mut w: impl Write) -> io::Result<usize> {
	let file = File::open(path)?;
	let reader = BufReader::new(file);
	write_xyzr_from_reader(reader, opts, &mut w)
}

pub fn write_xyzr_from_reader<R: BufRead>(
	reader: R,
	opts: &PdbOptions,
	mut w: impl Write,
) -> io::Result<usize> {
	let atoms = parse_atom_records(reader)?;
	let residue_map = classify_residues(&atoms);
	let mut count = 0usize;
	for rec in atoms {
		let key = make_residue_key(&rec);
		if let Some(info) = residue_map.get(&key) {
			if should_filter(info, &opts.filters) {
				continue;
			}
		}
		let radius_text = radius_text_for(&rec.residue, &rec.atom, opts.use_united);
		writeln!(
			w,
			"{:>8} {:>8} {:>8} {}",
			rec.x.trim(),
			rec.y.trim(),
			rec.z.trim(),
			radius_text
		)?;
		count += 1;
	}
	Ok(count)
}

fn parse_atom_records<R: BufRead>(reader: R) -> io::Result<Vec<AtomRecord>> {
	let mut atoms: Vec<AtomRecord> = Vec::new();
	for line_res in reader.lines() {
		let line = line_res?;
		if line.len() < 6 {
			continue;
		}
		let record = trim(&line[..6]).to_ascii_uppercase();
		if record != "ATOM" && record != "HETATM" {
			continue;
		}
		let raw_x = get_field(&line, 30, 8);
		let raw_y = get_field(&line, 38, 8);
		let raw_z = get_field(&line, 46, 8);
		if trim(raw_x).is_empty() || trim(raw_y).is_empty() || trim(raw_z).is_empty() {
			continue;
		}
		let residue = trim(get_field(&line, 17, 3)).to_string();
		let atom_name = normalize_atom_name(get_field(&line, 12, 4));
		let resnum = trim(get_field(&line, 22, 4)).to_string();
		let chain = trim(get_field(&line, 21, 1)).to_string();
		let mut element = trim(get_field(&line, 76, 2)).to_string();
		if element.is_empty() && !atom_name.is_empty() {
			element = atom_name.chars().next().unwrap_or(' ').to_ascii_uppercase().to_string();
		}
		atoms.push(AtomRecord {
			x: raw_x.to_string(),
			y: raw_y.to_string(),
			z: raw_z.to_string(),
			residue,
			atom: atom_name,
			resnum,
			chain,
			element,
			record,
		});
	}
	Ok(atoms)
}
