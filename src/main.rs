use voxel_sphere::voxel_grid::grid;
use voxel_sphere::voxel_grid::info;


fn main() {
    let scale = 128 as usize;
    let len_i = 2*scale;
    let len_j = 2*scale;
    let len_k = 2*scale;
    let grid_size = 1.0; // 1 Angstrom per voxel

    info::print_citation();
    info::print_compile_info();

    let mut grid = grid::Grid3D::new(len_i, len_j, len_k, grid_size);
    grid.report_memory();

    // Add a sphere centered i grid
    //let radius = (scale as f64) * 0.6203504908994;
    let radius = 10.0;
    grid.add_sphere(scale, scale, scale, radius);
    grid.add_sphere(scale+10, scale+10, scale+10, radius);
    grid.add_sphere(scale-10, scale+10, scale-10, radius);

    grid.report_memory();

    // Print the number of filled voxels before inversion
    let filled_before = grid.count_filled();
    println!("Filled voxels: {}", filled_before);

    grid.write_to_mrc_file("sphere.mrc");
}
