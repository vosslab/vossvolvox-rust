# ARCHITECTURE

## Overview
`voxel_sphere` is a Rust crate that aims to mirror the vossvolvox C++ toolkit. It operates on a bit-packed voxel grid (`Grid3D`) to compute molecular volumes, surfaces, and channels directly from PDB inputs. XYZR files are not part of the Rust pipeline.

## Components (current and planned)
- **voxel_grid::grid**: Core `Grid3D` structure (dimensions, shifts, bitvec storage).
- **voxel_grid::utils**: Index conversions, bit access, memory reporting (expand to cover C++ `assignLimits` logic).
- **voxel_grid::manip**: Sphere add/remove with precomputed offsets.
- **voxel_grid::mrc_output**: MRC writer; origin fields should align with PDB-derived shifts.
- **voxel_grid::surface_area**: Edge-count surface estimation (will be tuned to match C++ results).
- **(planned) pdb**: Minimal PDB parser + VDW radii table + filtering flags (exclude ions/water/ligands/hetatm/nucleic/amino; hydrogen opt-in).
- **(planned) rasterization**: Accessible volume fill (`r+probe` spheres) and exclusion contraction (`trun_ExcludeGrid_fast` analogue with precomputed offsets).
- **(planned) cli binaries**: `src/bin/volume.rs` first, matching `Volume.exe` flags and output formatting.

## Data flow (Volume clone)
1. Parse PDB → atoms with radii (filters applied).
2. Compute bounds, pad by `MAXVDW + probe + 2*grid` and probe-derived safety margin; align dims to multiples of 4; record origin shifts.
3. Allocate `Grid3D` with derived dims; fill accessible grid by rasterizing spheres.
4. If probe > 0, contract to excluded volume using neighbor checks + precomputed offsets; otherwise keep accessible grid.
5. Compute filled voxel count and surface area; emit summary and tabbed line.
6. Optional outputs: surface PDB, MRC (with proper origin), EZD if added later.

## External reference
- Behavior, padding rules, and expected outputs are defined by `vossvolvox-cpp/` (see `utils-main.cpp`, `volume.cpp`).
- Tests in `vossvolvox-cpp/test/test_suite.yml` define target volumes/surfaces and PDB hashes for validation (e.g., 2LYZ/1BL8 cases).
