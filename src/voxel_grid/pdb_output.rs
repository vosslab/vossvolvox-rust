/// Generate PDB string from grid position
pub fn ijk_to_pdb(i: usize, j: usize, k: usize, index: usize) -> String {
	format!("ATOM  {:5}  C   RES A   1    {:8.3} {:8.3} {:8.3}",
		index, i as f32, j as f32, k as f32)
}
