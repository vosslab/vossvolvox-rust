# AGENT NOTES

Mission: recreate the vossvolvox toolkit in Rust with output parity to the C++ reference. Always consume PDB directly; never introduce XYZR in the Rust path. Use the C++ code in `vossvolvox-cpp/` and its test suite expectations as the oracle for behavior and formatting.

## Priorities
- **Parity first:** match volumes, surfaces, and output hashes for the published test cases (see `vossvolvox-cpp/test/test_suite.yml`).
- **PDB-only:** implement radii lookup and filters so the Rust pipeline starts from PDB and produces the same grids the C++ code gets after XYZR conversion.
- **Grid fidelity:** mirror `assignLimits`, padding, and grid alignment (multiples of 4; probe padding) so indices and voxel counts align with the reference.

## Near-term tasks
- Build shared Rust utilities: PDB parser + VDW radii table, grid sizing helpers, sphere rasterization, exclusion contraction, surface area calculation, volume formatting, PDB/MRC writers with matching origins.
- Implement `src/bin/volume.rs` mirroring `Volume.exe` flags/output: `-i pdb`, `-p probe`, `-g grid`, filters, `-o pdb`, optional `-m mrc`; print compile info, citation, summary, and tabbed result line.
- Validate against the 2LYZ/1BL8 test cases; adjust algorithms until outputs match the reference values/hashes.

## Practices
- Keep files ASCII; prefer `bitvec`-backed `Grid3D` as the main storage.
- Avoid destructive git commands; respect existing user changes.
- When adding frontends/CLIs, match the C++ logging tone (progress bars, summary blocks).

## Reference
- C++ source of truth: `vossvolvox-cpp/src/utils-main.cpp`, `volume.cpp`, related outputs.
- Test expectations: `vossvolvox-cpp/test/test_suite.yml` (volumes, surfaces, PDB line counts/hashes).
See Python coding style in docs/PYTHON_STYLE.md.
## Coding Style
See Markdown style in docs/MARKDOWN_STYLE.md.
When making edits, document them in docs/CHANGELOG.md.
See repo style in docs/REPO_STYLE.md.
Agents may run programs in the tests folder, including smoke tests and pyflakes/mypy runner scripts.
