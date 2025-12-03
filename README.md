# vossvolvox-rust

Rust reimplementation of the vossvolvox toolkit, targeting output parity with the legacy C++ utilities while using PDB input directly (no XYZR step). The core data structure is a bit-packed `Grid3D` voxel grid; the goal is to reproduce tools such as `Volume.exe`, `VolumeNoCav.exe`, `VDW.exe`, and channel/cavity finders.

## Status
- Skeleton voxel grid plus MRC writer exist; the C++ reference lives in `vossvolvox-cpp/`.
- PDB-only workflow is planned; XYZR is intentionally out of scope.
- Next milestone: a `volume` binary that matches `Volume.exe` outputs on the published test suite (e.g., 2LYZ/1BL8 cases).

## Layout
- `src/`: Rust library code (voxel grid, utils, outputs).
- `src/bin/`: will host CLI tools (e.g., `volume`).
- `vossvolvox-cpp/`: reference implementation and test suite expectations.

## Getting started
```
cargo build
```

Once the Volume clone lands, you’ll be able to run:
```
cargo run --bin volume -- --help
```

## Roadmap (abridged)
- Implement PDB parser + VDW radii table with filtering flags (exclude ions/water/ligands/hetatm/nucleic/amino, hydrogen opt-in).
- Port grid sizing/padding logic to mirror `utils-main.cpp` (`assignLimits`, `getIdealGrid` behavior).
- Port rasterization and exclusion contraction; match volume and surface area outputs for the test suite.
- Wire PDB/MRC/edge PDB outputs to match hashes/line counts in `vossvolvox-cpp/test/test_suite.yml`.
