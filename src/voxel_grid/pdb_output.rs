use std::fs::File;
use std::io::{BufWriter, Write};

use crate::voxel_grid::grid::Grid3D;
use crate::voxel_grid::surface_area::classify_edge_point;

/// Write surface voxels to a PDB file.
/// A voxel is considered surface if any of its 6 face neighbors is empty or out of bounds.
pub fn write_surface_pdb(grid: &Grid3D, path: &str) -> std::io::Result<()> {
	let mut file = BufWriter::new(File::create(path)?);
	let mut serial = 1usize;
    for k in 0..grid.len_k {
        for j in 0..grid.len_j {
            for i in 0..grid.len_i {
                if !grid.get_voxel_ijk(i, j, k) {
                    continue;
                }
                let idx = i + j * grid.len_i + k * grid.len_i * grid.len_j;
                if classify_edge_point(grid, idx) == 0 {
                    continue;
                }
                let x = i as f32 * grid.grid_size + grid.x_shift;
                let y = j as f32 * grid.grid_size + grid.y_shift;
                let z = k as f32 * grid.grid_size + grid.z_shift;
                writeln!(
                    file,
                    "ATOM  {:5}  C   RES A   1    {:8.3} {:8.3} {:8.3}",
                    serial, x, y, z
                )?;
                serial += 1;
            }
        }
    }
    writeln!(file, "END")?;
    Ok(())
}
