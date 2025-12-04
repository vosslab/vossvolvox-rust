#!/usr/bin/env bash
#
# Rust analogue of vossvolvox-cpp/test/test_volume.sh
# Downloads 2LYZ PDB, runs the Rust `volume` binary with filters, and checks summary values.

set -euo pipefail

cleanup_files=()
cleanup() {
  for f in "${cleanup_files[@]:-}"; do
    if [ -n "$f" ] && [ -f "$f" ]; then
      rm -f "$f"
    fi
  done
}
trap cleanup EXIT

compare_float() {
  local label="$1"
  local expected="$2"
  local actual="$3"
  local tolerance="${4:-1e-3}"
  python3 - "$label" "$expected" "$actual" "$tolerance" <<'PY'
import sys, math
label, expected, actual, tol = sys.argv[1:]
expected = float(expected)
actual = float(actual)
tol = float(tol)
if math.fabs(expected - actual) > tol:
    print(f"{label} mismatch: expected {expected}, got {actual}")
    sys.exit(1)
PY
}

overall_status=0

PDB_ID="2LYZ"
PDB_FILE="${PDB_ID}.pdb"
OUTPUT_MRC="${PDB_ID}-volume.mrc"
OUTPUT_PDB="${PDB_ID}-volume.pdb"

# Expected values derived from the C++ test suite
EXPECTED_VOLUME=18551.124
EXPECTED_SURFACE=4982.05
EXPECTED_PDB_LINES=4874
PDB_MD5_SANITIZED="9d50d5818de494c11c3f3ff32160dfa5"
VOLUME_TOLERANCE=0.01

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Locate/build the Rust binary
if [ -z "${CARGO_BIN_EXE_volume:-}" ]; then
  echo "Building Rust volume binary..."
  (cd "${ROOT_DIR}" && cargo build --bin volume >/dev/null)
  VOLUME_BIN="${ROOT_DIR}/target/debug/volume"
else
  VOLUME_BIN="${CARGO_BIN_EXE_volume}"
fi

# Step A/B: Download or reuse the PDB file
if [ -s "${PDB_FILE}" ]; then
  echo "Found existing ${PDB_FILE}; skipping download."
else
  if [ -s "${PDB_FILE}.gz" ]; then
    echo "Found existing ${PDB_FILE}.gz; reusing local copy."
  else
    echo "Downloading PDB file for ${PDB_ID}..."
    curl -s -L -o "${PDB_FILE}.gz" "https://files.rcsb.org/download/${PDB_ID}.pdb.gz"
  fi
  echo "Extracting PDB file..."
  gunzip -f "${PDB_FILE}.gz"
fi

PDB_LINES=$(wc -l < "${PDB_FILE}")
echo "Downloaded PDB file has ${PDB_LINES} lines."

# Step: Run the Rust volume binary
echo "Running Rust volume with probe radius 2.1 and grid spacing 0.9..."
VOLUME_LOG=$(mktemp)
cleanup_files+=("$VOLUME_LOG")
if ! "${VOLUME_BIN}" -i "${PDB_FILE}" -p 2.1 -g 0.9 --exclude-ions --exclude-water -m "${OUTPUT_MRC}" -o "${OUTPUT_PDB}"; then
  echo "volume failed; log output:" >&2
  cat "${VOLUME_LOG}" >&2
  exit 1
fi

SUMMARY_LINE=$(tail -n 1 "${VOLUME_LOG}" || true)
SUMMARY_NORMALIZED=$(echo "${SUMMARY_LINE}" | awk '{$1=$1}1')
read -r SUMMARY_PROBE SUMMARY_GRID SUMMARY_VOLUME SUMMARY_SURFACE SUMMARY_ATOMS SUMMARY_INPUT <<<"${SUMMARY_NORMALIZED}"
echo "Volume summary: volume=${SUMMARY_VOLUME} A^3, surface=${SUMMARY_SURFACE} A^2, atoms=${SUMMARY_ATOMS} (input ${SUMMARY_INPUT})."
if ! compare_float "Volume" "${EXPECTED_VOLUME}" "${SUMMARY_VOLUME}" "${VOLUME_TOLERANCE}"; then
  overall_status=1
fi
if ! compare_float "Surface area" "${EXPECTED_SURFACE}" "${SUMMARY_SURFACE}" "${VOLUME_TOLERANCE}"; then
  overall_status=1
fi

# Count lines in the output PDB (surface)
if [ -f "${OUTPUT_PDB}" ]; then
  OUTPUT_LINES=$(wc -l < "${OUTPUT_PDB}")
  echo "Surface PDB file has ${OUTPUT_LINES} lines."
  if [ "${OUTPUT_LINES}" -ne "${EXPECTED_PDB_LINES}" ]; then
    echo "Line count mismatch: expected ${EXPECTED_PDB_LINES}, got ${OUTPUT_LINES}" >&2
    overall_status=1
  fi

  # Sanitize PDB (strip whitespace differences) and md5 if available
  if command -v md5sum >/dev/null 2>&1; then
    SANITIZED_MD5=$(sed 's/[[:space:]]\+//g' "${OUTPUT_PDB}" | md5sum | awk '{print $1}')
    echo "Sanitized PDB md5: ${SANITIZED_MD5}"
    if [ -n "${PDB_MD5_SANITIZED}" ] && [ "${SANITIZED_MD5}" != "${PDB_MD5_SANITIZED}" ]; then
      echo "Sanitized MD5 mismatch: expected ${PDB_MD5_SANITIZED}, got ${SANITIZED_MD5}" >&2
      overall_status=1
    fi
  fi
fi

if [ "${overall_status}" -ne 0 ]; then
  echo "Test completed with failures."
  exit 1
fi

echo "Rust volume test completed successfully!"
