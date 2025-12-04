#!/usr/bin/env bash
#
# Rust analogue of vossvolvox-cpp/test/test_pdb_to_xyzr.sh
# Compares Rust pdb_to_xyzr output to the C++ binary (if available) and the python/shell converters (if present).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
RESULT_DIR="${SCRIPT_DIR}/pdb_to_xyzr_results"
mkdir -p "${RESULT_DIR}"

PDB_ID="${1:-2LYZ}"
OUTPUT_DIR="${RESULT_DIR}/${PDB_ID}"
mkdir -p "${OUTPUT_DIR}"

XYZR_DIR="${REPO_DIR}/xyzr"
BIN_DIR_CPP="${REPO_DIR}/vossvolvox-cpp/bin"

download_pdb() {
  local pdb="$1"
  local pdb_file="${OUTPUT_DIR}/${pdb}.pdb"
  if [ -f "${pdb_file}" ]; then
    echo "Reusing cached ${pdb_file}" >&2
    return 0
  fi
  echo "Downloading ${pdb}..." >&2
  local gz_file="${pdb_file}.gz"
  curl -s -L -o "${gz_file}" "https://files.rcsb.org/download/${pdb}.pdb.gz"
  if [ ! -f "${gz_file}" ]; then
    echo "Download failed for ${pdb}" >&2
    return 1
  fi
  echo "Unpacking ${gz_file}" >&2
  gunzip -f "${gz_file}"
  if [ ! -f "${pdb_file}" ]; then
    echo "Decompression failed for ${pdb}" >&2
    return 1
  fi
}

prepare_input() {
  local pdb="$1"
  local pdb_file="${OUTPUT_DIR}/${pdb}.pdb"
  if ! download_pdb "${pdb}"; then
    return 1
  fi
  echo "${pdb_file}"
}

INPUT_PDB=$(prepare_input "${PDB_ID}") || {
  echo "Failed to prepare input for ${PDB_ID}" >&2
  exit 1
}

# Locate/build Rust binary
if [ -z "${CARGO_BIN_EXE_pdb_to_xyzr:-}" ]; then
  echo "Building Rust pdb_to_xyzr binary..." >&2
  (cd "${REPO_DIR}" && cargo build --bin pdb_to_xyzr >/dev/null)
  PDB_TO_XYZR_RUST="${REPO_DIR}/target/debug/pdb_to_xyzr"
else
  PDB_TO_XYZR_RUST="${CARGO_BIN_EXE_pdb_to_xyzr}"
fi

# C++ binary if present
PDB_TO_XYZR_CPP="${BIN_DIR_CPP}/pdb_to_xyzr.exe"

IMPLEMENTATIONS=()
IMPLEMENTATIONS+=("rust::${PDB_TO_XYZR_RUST}")

if [ -x "${PDB_TO_XYZR_CPP}" ]; then
  IMPLEMENTATIONS+=("cpp::${PDB_TO_XYZR_CPP}")
fi
if [ -f "${XYZR_DIR}/pdb_to_xyzr.py" ]; then
  IMPLEMENTATIONS+=("python::python3 ${XYZR_DIR}/pdb_to_xyzr.py")
fi
if [ -f "${XYZR_DIR}/pdb_to_xyzr.sh" ]; then
  IMPLEMENTATIONS+=("sh::${XYZR_DIR}/pdb_to_xyzr.sh")
fi

KEYS=()
DURATIONS=()
MD5S=()
OUTPUTS=()
LINE_COUNTS=()

hash_cmd() {
  local file="$1"
  if command -v md5sum >/dev/null 2>&1; then
    md5sum "${file}" | awk '{print $1}'
  else
    md5 -q "${file}"
  fi
}

for entry in "${IMPLEMENTATIONS[@]}"; do
  key="${entry%%::*}"
  cmd="${entry#*::}"
  output="${OUTPUT_DIR}/${key}.xyzr"
  echo "Running ${key} converter -> ${output}" >&2
  start_ns=$(date +%s%N)
  if ! eval "${cmd} \"${INPUT_PDB}\" > \"${output}\""; then
    echo "Failed: ${cmd}" >&2
    continue
  fi
  end_ns=$(date +%s%N)
  elapsed_ms=$(( (end_ns - start_ns)/1000000 ))
  md5=$(hash_cmd "${output}")
  line_count=$(wc -l < "${output}")
  KEYS+=("${key}")
  DURATIONS+=("${elapsed_ms}")
  MD5S+=("${md5}")
  OUTPUTS+=("${output}")
  LINE_COUNTS+=("${line_count}")
done

echo ""
printf "%-8s %-8s %-10s %s\n" "Impl" "Lines" "Duration" "MD5"
printf "%-8s %-8s %-10s %s\n" "----" "-----" "--------" "---"
for ((idx=0; idx<${#KEYS[@]}; idx++)); do
  printf "%-8s %-8s %-10s %s\n" "${KEYS[$idx]}" "${LINE_COUNTS[$idx]}" "${DURATIONS[$idx]}ms" "${MD5S[$idx]}"
done

diff_lines() {
  python3 - "$1" "$2" <<'PY'
import sys
from itertools import zip_longest
ref = open(sys.argv[1]).read().splitlines()
other = open(sys.argv[2]).read().splitlines()
diff = sum(1 for a, b in zip_longest(ref, other, fillvalue=None) if a != b)
print(diff)
PY
}

baseline_key="rust"
baseline_file=""
for ((idx=0; idx<${#KEYS[@]}; idx++)); do
  if [ "${KEYS[$idx]}" = "${baseline_key}" ]; then
    baseline_file="${OUTPUTS[$idx]}"
    break
  fi
done

if [ -n "${baseline_file}" ]; then
  echo ""
  echo "Line differences vs ${baseline_key}.xyzr:"
  printf "%-8s %s\n" "Impl" "DifferingLines"
  printf "%-8s %s\n" "----" "--------------"
  for ((idx=0; idx<${#KEYS[@]}; idx++)); do
    key="${KEYS[$idx]}"
    file="${OUTPUTS[$idx]}"
    if [ "${key}" = "${baseline_key}" ]; then
      continue
    fi
    diff_count=$(diff_lines "${baseline_file}" "${file}")
    printf "%-8s %s\n" "${key}" "${diff_count}"
  done
else
  echo ""
  echo "Baseline (${baseline_key}) output missing; skipping line comparison."
fi
