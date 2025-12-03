use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::thread;

use bitvec::vec::BitVec;

use crate::voxel_grid::grid::Grid3D;

/// Minimal atom representation for rasterization
#[derive(Debug, Clone)]
pub struct Atom {
	pub x: f32,
	pub y: f32,
	pub z: f32,
	pub radius: f32,
}

impl Grid3D {
	/// Fill the grid with spheres (accessible volume) in parallel.
	/// Atoms are specified in physical units; `probe` is added to each atom radius.
	/// Returns the number of filled voxels.
	pub fn fill_accessible_parallel(&mut self, atoms: &[Atom], probe: f32) -> usize {
		if atoms.is_empty() {
			self.data.fill(false);
			return 0;
		}

		let total_voxels = self.total_voxels;
		let grid_size = self.grid_size;
		let len_i = self.len_i as isize;
		let len_j = self.len_j as isize;
		let len_k = self.len_k as isize;
		let x_shift = self.x_shift;
		let y_shift = self.y_shift;
		let z_shift = self.z_shift;

		// Thread-friendly backing buffer; each cell is 0/1.
		let backing = Arc::new(vec![AtomicU8::new(0); total_voxels]);

		let threads = thread::available_parallelism()
			.map(|n| n.get())
			.unwrap_or(1);
		let chunk_size = (atoms.len() + threads - 1) / threads;

		thread::scope(|scope| {
			for atom_chunk in atoms.chunks(chunk_size) {
				let data = Arc::clone(&backing);
				scope.spawn(move || {
					for atom in atom_chunk {
						let effective_r = atom.radius + probe;
						let r_grid = effective_r / grid_size;
						if r_grid <= 0.0 {
							continue;
						}
						let cutoff = r_grid * r_grid;

						let xk = (atom.x - x_shift) / grid_size;
						let yk = (atom.y - y_shift) / grid_size;
						let zk = (atom.z - z_shift) / grid_size;

						// Bounding box in voxel coordinates, clamped to grid.
						let imin = ((xk - r_grid - 1.0).floor() as isize).clamp(0, len_i - 1);
						let jmin = ((yk - r_grid - 1.0).floor() as isize).clamp(0, len_j - 1);
						let kmin = ((zk - r_grid - 1.0).floor() as isize).clamp(0, len_k - 1);
						let imax = ((xk + r_grid + 1.0).ceil() as isize).clamp(0, len_i - 1);
						let jmax = ((yk + r_grid + 1.0).ceil() as isize).clamp(0, len_j - 1);
						let kmax = ((zk + r_grid + 1.0).ceil() as isize).clamp(0, len_k - 1);

						for i in imin..=imax {
							let dx = xk - i as f32;
							let dx2 = dx * dx;
							for j in jmin..=jmax {
								let dy = yk - j as f32;
								let dy2 = dy * dy;
								for k in kmin..=kmax {
									let dz = zk - k as f32;
									let dist2 = dx2 + dy2 + dz * dz;
									if dist2 < cutoff {
										let idx = i as usize + j as usize * (len_i as usize) + k as usize * (len_i as usize) * (len_j as usize);
										data[idx].store(1, Ordering::Relaxed);
									}
								}
							}
						}
					}
				});
			}
		});

		// Consolidate into BitVec and count filled voxels.
		let mut filled = 0usize;
		let mut bits = BitVec::with_capacity(total_voxels);
		for cell in backing.iter() {
			let v = cell.load(Ordering::Relaxed) != 0;
			if v {
				filled += 1;
			}
			bits.push(v);
		}
		self.data = bits;
		filled
	}
}
