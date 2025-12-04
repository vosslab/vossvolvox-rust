use crate::voxel_grid::grid::Grid3D;
use crate::voxel_grid::raster::Atom;

const MAX_VDW: f32 = 2.0;

/// Computed grid parameters based on atom bounds, probe, and grid spacing.
#[derive(Debug, Clone)]
pub struct GridParams {
	pub xmin: f32,
	pub xmax: f32,
	pub ymin: f32,
	pub ymax: f32,
	pub zmin: f32,
	pub zmax: f32,
	pub len_i: usize,
	pub len_j: usize,
	pub len_k: usize,
	pub grid: f32,
}

impl GridParams {
	/// Compute grid parameters mimicking the legacy padding/alignment rules.
	pub fn from_atoms(atoms: &[Atom], probe: f32, grid: f32) -> Option<Self> {
		if atoms.len() < 3 {
			return None;
		}

		let mut min_x = f32::MAX;
		let mut min_y = f32::MAX;
		let mut min_z = f32::MAX;
		let mut max_x = f32::MIN;
		let mut max_y = f32::MIN;
		let mut max_z = f32::MIN;
		let mut counted = 0usize;

		for atom in atoms {
			let r = atom.radius;
			if r <= 0.0 || r >= 100.0 {
				continue;
			}
			counted += 1;
			if atom.x < min_x {
				min_x = atom.x;
			}
			if atom.x > max_x {
				max_x = atom.x;
			}
			if atom.y < min_y {
				min_y = atom.y;
			}
			if atom.y > max_y {
				max_y = atom.y;
			}
			if atom.z < min_z {
				min_z = atom.z;
			}
			if atom.z > max_z {
				max_z = atom.z;
			}
		}

		if counted < 3 {
			return None;
		}

		// Initial padding: MAX_VDW + probe + 2*grid, aligned to 4*grid boundaries.
		let fact = MAX_VDW + probe + 2.0 * grid;
		min_x = align_down_four(min_x - fact, grid);
		min_y = align_down_four(min_y - fact, grid);
		min_z = align_down_four(min_z - fact, grid);
		max_x = align_up_four(max_x + fact, grid);
		max_y = align_up_four(max_y + fact, grid);
		max_z = align_up_four(max_z + fact, grid);

		// Safety padding based on probe/grid ratio.
		let safety_cells = (probe / grid).ceil() as i32 + 2;
		if safety_cells > 0 {
			let padding = safety_cells as f32 * grid;
			min_x -= padding;
			min_y -= padding;
			min_z -= padding;
			max_x += padding;
			max_y += padding;
			max_z += padding;
		}

		let len_i = calculate_dimension(min_x, max_x, grid);
		let len_j = calculate_dimension(min_y, max_y, grid);
		let len_k = calculate_dimension(min_z, max_z, grid);

		Some(Self {
			xmin: min_x,
			xmax: max_x,
			ymin: min_y,
			ymax: max_y,
			zmin: min_z,
			zmax: max_z,
			len_i,
			len_j,
			len_k,
			grid,
		})
	}

	/// Instantiate a `Grid3D` using these parameters.
	pub fn build_grid(&self) -> Grid3D {
		let mut grid = Grid3D::new(self.len_i, self.len_j, self.len_k, self.grid);
		grid.x_shift = self.xmin;
		grid.y_shift = self.ymin;
		grid.z_shift = self.zmin;
		grid
	}
}

fn calculate_dimension(min: f32, max: f32, grid: f32) -> usize {
	let span = (max - min) / grid;
	(((span / 4.0) + 1.0).ceil() as usize) * 4
}

fn align_down_four(value: f32, grid: f32) -> f32 {
	let factor = value / (4.0 * grid);
	let t = factor as i32; // truncates toward zero like C int cast
	((t - 1) as f32) * 4.0 * grid
}

fn align_up_four(value: f32, grid: f32) -> f32 {
	let factor = value / (4.0 * grid);
	let t = factor as i32; // truncates toward zero like C int cast
	((t + 1) as f32) * 4.0 * grid
}
