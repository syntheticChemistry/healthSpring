#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Compute dashboard: runs toadStool dispatch matrix (exp069), PCIe P2P
# validation (exp070), and mixed system pipeline (exp071) with results
# output suitable for petalTongue StreamSession integration.
#
# This script demonstrates the end-to-end compute validation and
# petalTongue wiring path.
#
# Usage:
#   ./scripts/compute_dashboard.sh          # run all compute validations
#   ./scripts/compute_dashboard.sh --json   # also dump dispatch scenario JSON

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
JSON_MODE="${1:-}"

info()  { echo -e "\033[1;36m[compute-dashboard]\033[0m $*"; }
ok()    { echo -e "\033[1;32m  ✓\033[0m $*"; }
err()   { echo -e "\033[1;31m  ✗\033[0m $*"; }

# --- 1. Build all compute validation binaries ---
info "Building compute validation binaries (release) …"
cargo build --release \
    --manifest-path "${PROJECT_ROOT}/Cargo.toml" 2>&1 | tail -1
ok "All binaries built"

# --- 2. Run toadStool dispatch matrix ---
info "Running toadStool dispatch matrix (exp069) …"
"${PROJECT_ROOT}/target/release/exp069_toadstool_dispatch_matrix"
echo

# --- 3. Run PCIe P2P bypass validation ---
info "Running PCIe P2P bypass validation (exp070) …"
"${PROJECT_ROOT}/target/release/exp070_pcie_p2p_bypass"
echo

# --- 4. Run mixed system pipeline ---
info "Running mixed system pipeline (exp071) …"
"${PROJECT_ROOT}/target/release/exp071_mixed_system_pipeline"
echo

# --- 5. Run GPU crossover analysis ---
info "Running GPU crossover analysis (exp068) …"
"${PROJECT_ROOT}/target/release/exp068_gpu_benchmark"
echo

# --- 6. Run Rust CPU benchmarks ---
info "Running Rust CPU benchmarks (exp066) …"
"${PROJECT_ROOT}/target/release/exp066_barracuda_cpu_bench"
echo

# --- 7. Run compute dashboard (toadStool × petalTongue) ---
info "Running compute dashboard (exp072) …"
"${PROJECT_ROOT}/target/release/exp072_compute_dashboard"
echo

# --- 8. Optionally dump dispatch scenario JSON for petalTongue ---
if [[ "$JSON_MODE" == "--json" ]]; then
    info "Generating dispatch scenario JSON for petalTongue …"
    SCENARIOS_DIR="${PROJECT_ROOT}/sandbox/scenarios"
    mkdir -p "$SCENARIOS_DIR"

    "${PROJECT_ROOT}/target/release/dump_scenarios"
    ok "Scenarios written to ${SCENARIOS_DIR}"
fi

info "Compute dashboard complete — all validation binaries green"
