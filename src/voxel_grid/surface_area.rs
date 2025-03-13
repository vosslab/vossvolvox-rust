use crate::voxel_grid::grid::Grid3D;

impl Grid3D {
	/// Estimate the surface area using edge detection
	pub fn estimate_surface_area(&self) -> usize {
		let mut surface_count = 0;
		let dirs: [(isize, isize, isize); 6] = [
			(1, 0, 0), (-1, 0, 0),
			(0, 1, 0), (0, -1, 0),
			(0, 0, 1), (0, 0, -1),
		];

		for i in 0..self.len_i {
			for j in 0..self.len_j {
				for k in 0..self.len_k {
					if self.get_voxel_ijk(i, j, k) {
						for (di, dj, dk) in &dirs {
							let ni = i as isize + di;
							let nj = j as isize + dj;
							let nk = k as isize + dk;

							// Check bounds before accessing
							if ni < 0 || nj < 0 || nk < 0 ||
								ni >= self.len_i as isize || nj >= self.len_j as isize || nk >= self.len_k as isize ||
								!self.get_voxel_ijk(ni as usize, nj as usize, nk as usize) {
									surface_count += 1;
								}
						}
					}
				}
			}
		}

		surface_count
	}
}
