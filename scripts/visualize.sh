#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
#
# One-command launcher: build healthSpring, generate scenario JSONs,
# and open them in petalTongue.
#
# Usage:
#   ./scripts/visualize.sh              # default: web mode
#   ./scripts/visualize.sh ui           # egui desktop
#   ./scripts/visualize.sh tui          # ratatui terminal
#   ./scripts/visualize.sh web          # axum web server
#   ./scripts/visualize.sh web 8080     # custom port
#   ./scripts/visualize.sh --ipc        # also push via IPC after launching

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PETALTONGUE_ROOT="$(cd "${PROJECT_ROOT}/../phase2/petalTongue" 2>/dev/null && pwd || true)"

MODE="${1:-web}"
PORT="${2:-13377}"
IPC=false

for arg in "$@"; do
    if [[ "$arg" == "--ipc" ]]; then
        IPC=true
    fi
done

info()  { echo -e "\033[1;36m[healthSpring]\033[0m $*"; }
ok()    { echo -e "\033[1;32m  ✓\033[0m $*"; }
err()   { echo -e "\033[1;31m  ✗\033[0m $*"; }
die()   { err "$*"; exit 1; }

# --- 1. Build healthSpring dump binaries ---------------------------------

info "Building healthSpring scenario dump binaries …"
cargo build --release \
    --manifest-path "${PROJECT_ROOT}/experiments/exp056_study_scenarios/Cargo.toml" \
    --bin dump_scenarios 2>&1 | tail -1
ok "dump_scenarios built"

cargo build --release \
    --manifest-path "${PROJECT_ROOT}/experiments/exp063_clinical_trt_scenarios/Cargo.toml" \
    --bin dump_clinical_scenarios 2>&1 | tail -1
ok "dump_clinical_scenarios built"

# --- 2. Generate fresh scenario JSONs ------------------------------------

info "Generating scenario JSONs → sandbox/scenarios/ …"
"${PROJECT_ROOT}/target/release/dump_scenarios"
"${PROJECT_ROOT}/target/release/dump_clinical_scenarios"
ok "All scenarios written to sandbox/scenarios/"

# Sync to petalTongue sandbox so showcase demos see fresh data
if [[ -d "${PETALTONGUE_ROOT}/sandbox/scenarios" ]]; then
    "${SCRIPT_DIR}/sync_scenarios.sh"
fi

SCENARIOS_DIR="${PROJECT_ROOT}/sandbox/scenarios"
SCENARIO_FILE="${SCENARIOS_DIR}/healthspring-diagnostic.json"
if [[ ! -f "$SCENARIO_FILE" ]]; then
    SCENARIO_FILE="${SCENARIOS_DIR}/healthspring-full-study.json"
fi

# --- 3. Locate petalTongue ----------------------------------------------

if [[ -z "${PETALTONGUE_ROOT}" ]] || [[ ! -d "${PETALTONGUE_ROOT}" ]]; then
    die "petalTongue not found at ../phase2/petalTongue. Set PETALTONGUE_ROOT."
fi

PETALTONGUE_BIN="${PETALTONGUE_ROOT}/target/release/petaltongue"
if [[ ! -x "$PETALTONGUE_BIN" ]]; then
    info "Building petalTongue (release) …"
    cargo build --release --manifest-path "${PETALTONGUE_ROOT}/Cargo.toml" 2>&1 | tail -1
    ok "petalTongue built"
fi

# --- 4. Launch petalTongue with scenario ---------------------------------

info "Launching petalTongue in '${MODE}' mode …"

case "$MODE" in
    ui)
        "$PETALTONGUE_BIN" ui --scenario "$SCENARIO_FILE"
        ;;
    tui)
        "$PETALTONGUE_BIN" tui --scenario "$SCENARIO_FILE"
        ;;
    web)
        info "  Web UI: http://127.0.0.1:${PORT}"
        "$PETALTONGUE_BIN" web --bind "127.0.0.1:${PORT}" --scenario "$SCENARIO_FILE" &
        PT_PID=$!

        for i in $(seq 1 15); do
            if curl -sf "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
                ok "petalTongue web server ready"
                break
            fi
            sleep 1
        done

        if ! kill -0 "$PT_PID" 2>/dev/null; then
            die "petalTongue exited unexpectedly"
        fi

        # --- 5. Optional IPC push ----------------------------------------
        if [[ "$IPC" == "true" ]]; then
            info "Pushing scenarios via IPC …"
            SOCKET_PATH="/tmp/petaltongue-${PT_PID}.sock"
            export PETALTONGUE_SOCKET="$SOCKET_PATH"

            EXP064="${PROJECT_ROOT}/target/release/exp064_ipc_push"
            if [[ -x "$EXP064" ]]; then
                "$EXP064" || info "IPC push completed (petalTongue may not have socket listener)"
            else
                info "exp064_ipc_push not built — skipping IPC push"
            fi
        fi

        info "Press Ctrl-C to stop"
        wait "$PT_PID"
        ;;
    *)
        die "Unknown mode: ${MODE}. Use ui, tui, or web."
        ;;
esac
