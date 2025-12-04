use crate::voxel_grid::grid::Grid3D;

/// Edge classification types as in C++ `classifyEdgePoint`.
pub fn classify_edge_point(grid: &Grid3D, idx: usize) -> usize {
	let stride_i = 1usize;
	let stride_j = grid.len_i;
	let stride_k = grid.len_i * grid.len_j;

	let im = neighbor_filled(grid, idx, stride_i, false);
	let ip = neighbor_filled(grid, idx, stride_i, true);
	let jm = neighbor_filled(grid, idx, stride_j, false);
	let jp = neighbor_filled(grid, idx, stride_j, true);
	let km = neighbor_filled(grid, idx, stride_k, false);
	let kp = neighbor_filled(grid, idx, stride_k, true);

	let nb_empty =
		(!im as usize) + (!ip as usize) + (!jm as usize) + (!jp as usize) + (!km as usize) + (!kp as usize);

	match nb_empty {
		0 | 1 => nb_empty,
		2 => {
			if (!im && !ip) || (!jm && !jp) || (!km && !kp) {
				7
			} else {
				2
			}
		}
		3 => {
			if (!im && !ip) || (!jm && !jp) || (!km && !kp) {
				4
			} else {
				3
			}
		}
		4 => {
			if (im && ip) || (jm && jp) || (km && kp) {
				8
			} else {
				5
			}
		}
		5 => 6,
		6 => 9,
		_ => 0,
	}
}

impl Grid3D {
	/// Estimate surface area using legacy edge classification weights (matches C++ utils-main.cpp).
	pub fn estimate_surface_area_with_edges(&self) -> (f64, [f64; 10]) {
		// Weighting factors indexed by classified edge type (1-based).
		let wt = [0.0_f64, 0.894, 1.3409, 1.5879, 4.0, 2.6667, 3.3333, 1.79, 2.68, 4.08, 0.0];

		let mut edges = [0usize; 10];
		for k in 0..self.len_k {
			for j in 0..self.len_j {
				for i in 0..self.len_i {
					let idx = i + j * self.len_i + k * self.len_i * self.len_j;
					if !self.data[idx] {
						continue;
					}
					let typ = classify_edge_point(self, idx);
					if typ < edges.len() {
						edges[typ] += 1;
					}
				}
			}
		}

		let mut surf = 0.0_f64;
		let mut edges_f = [0.0_f64; 10];
		for (ty, &count) in edges.iter().enumerate() {
			edges_f[ty] = count as f64;
			if ty < wt.len() {
				surf += (count as f64) * wt[ty];
			}
		}
		let surface = surf * (self.grid_size as f64) * (self.grid_size as f64);
		(surface, edges_f)
	}
}

fn neighbor_filled(grid: &Grid3D, pt: usize, stride: usize, positive: bool) -> bool {
	if positive {
		let idx = pt + stride;
		if idx >= grid.total_voxels {
			false
		} else {
			grid.data[idx]
		}
	} else {
		match pt.checked_sub(stride) {
			Some(idx) => grid.data[idx],
			None => false,
		}
	}
}
