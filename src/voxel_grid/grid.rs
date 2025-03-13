use bitvec::vec::BitVec;

/// 3D Voxel Grid with bit-packed storage
#[derive(Clone)]
pub struct Grid3D {
	pub len_i: usize,  // Number of voxels along I
	pub len_j: usize,  // Number of voxels along J
	pub len_k: usize,  // Number of voxels along K
	pub total_voxels: usize, // Total number of voxels IxJxK
	pub grid_size: f32,  // Size of each voxel in angstroms
	pub x_shift: f32,  // Offset for X to align with I=0
	pub y_shift: f32,  // Offset for Y to align with J=0
	pub z_shift: f32,  // Offset for Z to align with K=0
	pub data: BitVec,  // 1-bit per voxel storage
}

impl Grid3D {
	/// Create a new voxel grid, fully allocated with all voxels set to `false`
	pub fn new(len_i: usize, len_j: usize, len_k: usize, grid_size: f32) -> Self {
		let total_voxels = len_i * len_j * len_k;

		Self {
			len_i,
			len_j,
			len_k,
			total_voxels,
			grid_size,
			x_shift: 0.0,
			y_shift: 0.0,
			z_shift: 0.0,
			data: BitVec::repeat(false, total_voxels), // Pre-allocate full grid
		}
	}
}
