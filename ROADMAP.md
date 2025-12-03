# ROADMAP

## Guiding principles
- PDB-only workflow: parse PDB directly, apply radii and filters in Rust. No XYZR step.
- Output parity with the C++ reference: volumes, surfaces, and output hashes must match the `vossvolvox-cpp` test suite.
- Incremental delivery: land Volume parity first, then expand to the other tools.

## Milestones

### 1) Volume.exe parity
- Implement PDB parser + VDW radii table (united-atom default; hydrogen opt-in).
- Add filtering flags: exclude ions, water, ligands, hetatm, nucleic acids, amino acids.
- Port grid sizing/padding logic (`assignLimits`/`getIdealGrid` equivalents) to align dimensions (multiples of 4, probe padding).
- Implement accessible fill (sphere rasterization) and excluded contraction (neighbor check + precomputed offsets).
- Match volume/surface outputs for 2LYZ/1BL8 cases; match PDB/MRC origin/format expectations.
- Ship `src/bin/volume.rs` with flags matching `Volume.exe`.

### 2) VolumeNoCav.exe and VDW.exe
- Add cavity filling and “probe=0 accessible” paths on top of shared grid ops.
- Validate against test suite PDB hashes/line counts.

### 3) Channel/Cavities/Solvent
- Port connectivity/flood-fill and channel/cavity detection logic.
- Mirror output formatting and optional MRC/PDB/EZD outputs as in C++.
- Validate against suite expectations (`Channel.exe`, `Cavities.exe`, `Solvent.exe`, `AllChannel*`).

### 4) Outputs and tooling
- Ensure PDB writer matches sanitized MD5 expectations.
- Align MRC header origins to PDB-derived shifts.
- Consider EZD writer if needed for parity.

### 5) DX/UX
- Solidify CLI argument parsing and help text to mirror legacy flags.
- Add logging/progress bar cues similar to the C++ tools.

## Validation
- Use `vossvolvox-cpp/test/test_suite.yml` expected volumes, surfaces, and PDB hashes as the oracle.
- Where downloads are needed (2LYZ/1BL8), vendor fixtures or document how to obtain them offline.

## Stretch goals
- Performance tuning for large grids (parallel fill, optimized neighbor checks).
- Optional feature flags for debug outputs and profiling.
