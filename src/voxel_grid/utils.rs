use std::mem::size_of;
use bitvec::prelude::BitVec;
use crate::voxel_grid::grid;

/// Format large numbers with KB, MB, GB, TB suffixes
fn format_bytes(bytes: usize) -> String {
	const KB: usize = 1024;
	const MB: usize = KB * 1024;
	const GB: usize = MB * 1024;
	const TB: usize = GB * 1024;

	if bytes >= TB {
		format!("{:.2} TB", bytes as f64 / TB as f64)
	} else if bytes >= GB {
		format!("{:.2} GB", bytes as f64 / GB as f64)
	} else if bytes >= MB {
		format!("{:.2} MB", bytes as f64 / MB as f64)
	} else if bytes >= KB {
		format!("{:.2} KB", bytes as f64 / KB as f64)
	} else {
		format!("{} bytes", bytes)
	}
}

impl grid::Grid3D {
	/// Report memory usage and print a detailed breakdown
	pub fn report_memory(&self) {
		let struct_overhead = size_of::<Self>() - size_of::<BitVec>(); // Exclude dynamic storage
		let bitvec_bits = self.data.capacity(); // Total bits allocated in BitVec
		let bitvec_bytes = bitvec_bits / 8; // Convert bits to bytes
		let total_memory = struct_overhead + bitvec_bytes;

		eprintln!("Grid3D Memory Report:");
		eprintln!("-------------------------");
		eprintln!("  Dimensions: {} x {} x {}", self.len_i, self.len_j, self.len_k);
		eprintln!("  Total Voxels: {:e}", self.total_voxels as f64); // Scientific notation
		eprintln!("  Grid Size: {:.2} A", self.grid_size);
		eprintln!("  Struct Overhead: {}", format_bytes(struct_overhead));
		eprintln!("  BitVec Capacity: {}", format_bytes(bitvec_bytes));
		eprintln!("  Total Memory Used: {}", format_bytes(total_memory));
		eprintln!("-------------------------");
	}

	/// Convert (i, j, k) to a linear index
	#[inline]
	pub fn ijk_to_index(&self, i: usize, j: usize, k: usize) -> usize {
		i + j * self.len_i + k * self.len_i * self.len_j
	}

	/// Convert (i, j, k) to a linear shift (+/-)
	#[inline]
	pub fn ijk_to_shift(&self, i: isize, j: isize, k: isize) -> isize {
		i + j * self.len_i as isize + k * self.len_i as isize * self.len_j as isize
	}

	/// Convert a linear index back to (i, j, k)
	#[inline]
	pub fn index_to_ijk(&self, index: usize) -> (usize, usize, usize) {
		let k = index / (self.len_i * self.len_j);
		let j = (index % (self.len_i * self.len_j)) / self.len_i;
		let i = index % self.len_i;
		(i, j, k)
	}

	/// Get a voxel value by linear index (panics if out of bounds)
	#[inline]
	pub fn get_voxel_index(&self, index: usize) -> bool {
		self.data[index]
	}

	/// Get a voxel value using (i, j, k) coordinates
	#[inline]
	pub fn get_voxel_ijk(&self, i: usize, j: usize, k: usize) -> bool {
		let index = self.ijk_to_index(i, j, k);
		self.get_voxel_index(index)
	}

	/// Set a voxel value by linear index (panics if out of bounds)
	#[inline]
	pub fn set_voxel_index(&mut self, index: usize, value: bool) {
		self.data.set(index, value);
	}

	/// Set a voxel value using (i, j, k) coordinates (assumes valid bounds)
	#[inline]
	pub fn set_voxel_ijk(&mut self, i: usize, j: usize, k: usize, value: bool) {
		let index = self.ijk_to_index(i, j, k);
		self.set_voxel_index(index, value);
	}

	/// Set a voxel to `true`
	#[inline]
	pub fn fill_voxel_ijk(&mut self, i: usize, j: usize, k: usize) {
		self.set_voxel_ijk(i, j, k, true);
	}

	/// Set a voxel to `true` using linear index
	#[inline]
	pub fn fill_voxel_index(&mut self, index: usize) {
		self.set_voxel_index(index, true);
	}

	/// Set a voxel to `false`
	#[inline]
	pub fn empty_voxel_ijk(&mut self, i: usize, j: usize, k: usize) {
		self.set_voxel_ijk(i, j, k, false);
	}

	/// Set a voxel to `false` using linear index
	#[inline]
	pub fn empty_voxel_index(&mut self, index: usize) {
		self.set_voxel_index(index, false);
	}

	/// Zero out the entire grid (sets all voxels to `false`)
	pub fn zero_grid(&mut self) {
		self.data.fill(false);
	}

	/// Invert the entire grid (flip all bits)
	pub fn invert(&mut self) {
		for mut bit in self.data.as_mut_bitslice().iter_mut() {
			*bit = !*bit; // Flip each bit manually
		}
	}

	/// Count the number of filled voxels
	pub fn count_filled(&self) -> usize {
		self.data.count_ones()
	}
}
