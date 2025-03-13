use std::fs::File;
use std::io::{Write, Result};
use crate::voxel_grid::grid;
use std::time::{SystemTime, UNIX_EPOCH};
use std::time::Instant;

/// MRC Header Struct
#[repr(C)]
#[derive(Debug)]
pub struct MRCHeader {
	len_i: i32, len_j: i32, len_k: i32,  // Grid dimensions
	mode: i32,                  // Data mode (0: Byte)
	istart: i32, jstart: i32, kstart: i32,  // Start positions
	m_i: i32, m_j: i32, m_k: i32,  // Grid size
	x_length: f32, y_length: f32, z_length: f32,  // Physical size
	alpha: f32, beta: f32, gamma: f32,  // Angles
	mapc: i32, mapr: i32, maps: i32,  // Axis mapping
	amin: f32, amax: f32, amean: f32,  // Data range
	ispg: i32, nsymbt: i32,  // Symmetry
	extra: [i32; 25],  // User space
	xorigin: f32, yorigin: f32, zorigin: f32,  // Origin (shifted for PDB alignment)
	map: i32, mach: i32, rms: f32, nlabl: i32,  // Metadata
	label: [[u8; 80]; 10],  // Labels
}

impl MRCHeader {
	/// Create a new MRC header
	pub fn new(len_i: usize, len_j: usize, len_k: usize, grid_size: f32, x_shift: f32, y_shift: f32, z_shift: f32) -> Self {
		MRCHeader {
			len_i: len_i as i32, len_j: len_j as i32, len_k: len_k as i32,
			mode: 0,  // BYTE mode
			istart: 0, jstart: 0, kstart: 0,
			m_i: len_i as i32, m_j: len_j as i32, m_k: len_k as i32,
			x_length: (len_i as f32) * grid_size,
			y_length: (len_j as f32) * grid_size,
			z_length: (len_k as f32) * grid_size,
			alpha: 90.0, beta: 90.0, gamma: 90.0,
			mapc: 1, mapr: 2, maps: 3,
			amin: 0.0, amax: 1.0, amean: 0.1,
			ispg: 0, nsymbt: 0,
			extra: [0; 25],
			xorigin: x_shift, yorigin: y_shift, zorigin: z_shift,
			map: 542130509,  // "MAP " ASCII identifier
			mach: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i32,
			rms: 0.0,
			nlabl: 0,
			label: [[0; 80]; 10],
		}
	}

	/// Write the header to an MRC file
	pub fn write_to_file(&self, file: &mut File) -> Result<()> {
		let header_bytes = unsafe {
			std::slice::from_raw_parts(
				(self as *const MRCHeader) as *const u8,
												std::mem::size_of::<MRCHeader>(),
			)
		};
		file.write_all(header_bytes)?;
		Ok(())
	}
}

impl grid::Grid3D {
	/// Save the voxel grid as an MRC file and report save time
	pub fn write_to_mrc_file(&self, filename: &str) {
		if let Ok(mut file) = File::create(filename) {
			let start_time = Instant::now(); // ⏱ Start Timer

			// Create and write the MRC header
			let header = MRCHeader::new(
				self.len_i, self.len_j, self.len_k,
				self.grid_size, self.x_shift, self.y_shift, self.z_shift,
			);

			if let Err(e) = header.write_to_file(&mut file) {
				eprintln!("Failed to write MRC header: {}", e);
				return;
			}

			// Store voxel data as `u8` (no `i8`)
			let mut voxel_bytes = vec![0u8; self.total_voxels];
			self.data.iter().enumerate().for_each(|(i, bit)| {
				voxel_bytes[i] = if *bit { 1u8 } else { 0u8 }; // Store as `0` or `1`
			});

			// Write voxel data directly as `u8`
			if let Err(e) = file.write_all(&voxel_bytes) {
				eprintln!("Failed to write voxel data: {}", e);
				return;
			}

			let elapsed_time = start_time.elapsed(); // ⏱ Stop Timer
			eprintln!("MRC file saved: {}", filename);
			eprintln!("Save Time: {:.3} seconds", elapsed_time.as_secs_f64());
		} else {
			eprintln!("Failed to create file: {}", filename);
		}
	}
}
