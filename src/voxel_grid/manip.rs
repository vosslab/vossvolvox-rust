use crate::voxel_grid::grid::Grid3D;
use indicatif::{ProgressBar, ProgressStyle};

impl Grid3D {
	pub fn compute_offsets(&self, radius: f64) -> Vec<isize> {
		let mut offsets = Vec::new();
		let r_int = 1 + radius as isize;
		let r2 = (radius * radius) as f64;

		for di in -r_int..=r_int {
			let di2 = di * di;
			for dj in -r_int..=r_int {
				let dj2 = dj * dj;
				for dk in -r_int..=r_int {
					// convert to f64 in last step
					let dist = (di2 + dj2 + dk * dk) as f64;
					if dist <= r2 {
						// Compute relative shift
						let shift = self.ijk_to_shift(di, dj, dk);
						offsets.push(shift);
					}
				}
			}
		}
		offsets
	}

	/// Modify a sphere (add or remove) using precomputed 1D offsets with progress bar
	pub fn modify_sphere_with_offsets(&mut self, ci: usize, cj: usize, ck: usize, offsets: &[isize], set_value: bool) {
		let center_index = self.ijk_to_index(ci, cj, ck) as isize; // Compute center index

		// Setup progress bar
		let pb = ProgressBar::new(offsets.len() as u64);
		pb.set_style(
			ProgressStyle::default_bar()
			.template("Updating Voxels: [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
			.unwrap()
			.progress_chars("#>-"),
		);

		for &shift in offsets.iter() {
			let voxel_index = center_index + shift; // Apply relative shift

			// Ensure voxel_index is valid
			if voxel_index >= 0 && (voxel_index as usize) < self.total_voxels {
				self.set_voxel_index(voxel_index as usize, set_value);
			}

			pb.inc(1); // Increment progress
		}

		pb.finish_with_message("Voxel modification complete!");
	}

	/// Compute offsets, then modify a sphere (add or remove)
	pub fn modify_sphere(&mut self, ci: usize, cj: usize, ck: usize, radius: f64, set_value: bool) {
		let offsets = self.compute_offsets(radius);
		self.modify_sphere_with_offsets(ci, cj, ck, &offsets, set_value);
	}

	/// Compute sphere offsets and then add a sphere
	pub fn add_sphere(&mut self, ci: usize, cj: usize, ck: usize, radius: f64) {
		self.modify_sphere(ci, cj, ck, radius, true);
	}

	/// Remove a sphere by calling `modify_sphere` with `false`
	pub fn remove_sphere(&mut self, ci: usize, cj: usize, ck: usize, radius: f64) {
		self.modify_sphere(ci, cj, ck, radius, false);
	}

}
