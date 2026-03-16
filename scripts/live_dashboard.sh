#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Live streaming dashboard: launch petalTongue and push healthSpring data
# incrementally via IPC (push_append / push_gauge_update) to simulate a
# real-time clinical monitoring session.
#
# Usage:
#   ./scripts/live_dashboard.sh              # web mode, default port
#   ./scripts/live_dashboard.sh ui           # egui desktop
#   ./scripts/live_dashboard.sh web 8080     # custom port

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PETALTONGUE_ROOT="${PETALTONGUE_ROOT:-$(cd "${PROJECT_ROOT}/../petalTongue" 2>/dev/null && pwd || cd "${PROJECT_ROOT}/../phase2/petalTongue" 2>/dev/null && pwd || cd "${PROJECT_ROOT}/../wateringHole/petaltongue" 2>/dev/null && pwd || true)}"

MODE="${1:-web}"
PORT="${2:-13378}"

info()  { echo -e "\033[1;36m[live-dashboard]\033[0m $*"; }
ok()    { echo -e "\033[1;32m  ✓\033[0m $*"; }
err()   { echo -e "\033[1;31m  ✗\033[0m $*"; }
die()   { err "$*"; exit 1; }

cleanup() {
    if [[ -n "${PT_PID:-}" ]] && kill -0 "$PT_PID" 2>/dev/null; then
        info "Stopping petalTongue (PID ${PT_PID}) …"
        kill "$PT_PID" 2>/dev/null || true
        wait "$PT_PID" 2>/dev/null || true
    fi
    if [[ -n "${STREAMER_PID:-}" ]] && kill -0 "$STREAMER_PID" 2>/dev/null; then
        kill "$STREAMER_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM

# --- 1. Locate petalTongue ----------------------------------------------

if [[ -z "${PETALTONGUE_ROOT}" ]] || [[ ! -d "${PETALTONGUE_ROOT}" ]]; then
    die "petalTongue not found. Set PETALTONGUE_ROOT env var."
fi

PETALTONGUE_BIN="${PETALTONGUE_ROOT}/target/release/petaltongue"
if [[ ! -x "$PETALTONGUE_BIN" ]]; then
    info "Building petalTongue (release) …"
    cargo build --release --manifest-path "${PETALTONGUE_ROOT}/Cargo.toml" 2>&1 | tail -1
    ok "petalTongue built"
fi

# --- 2. Build the streaming binary --------------------------------------

STREAMER_CRATE="${PROJECT_ROOT}/experiments/exp065_live_dashboard"
STREAMER_BIN="${PROJECT_ROOT}/target/release/exp065_live_dashboard"

if [[ -d "$STREAMER_CRATE" ]]; then
    info "Building live dashboard streamer …"
    cargo build --release \
        --manifest-path "${STREAMER_CRATE}/Cargo.toml" 2>&1 | tail -1
    ok "exp065_live_dashboard built"
else
    die "exp065_live_dashboard crate not found. Build it first."
fi

# --- 3. Generate base scenario ------------------------------------------

info "Generating base scenarios …"
DUMP_BIN="${PROJECT_ROOT}/target/release/dump_scenarios"
if [[ ! -x "$DUMP_BIN" ]]; then
    cargo build --release \
        --manifest-path "${PROJECT_ROOT}/experiments/exp056_study_scenarios/Cargo.toml" \
        --bin dump_scenarios 2>&1 | tail -1
fi
"$DUMP_BIN"
ok "Base scenarios ready"

SCENARIOS_DIR="${PROJECT_ROOT}/sandbox/scenarios"
SCENARIO_FILE="${SCENARIOS_DIR}/healthspring-biosignal.json"

# --- 4. Start petalTongue in background ---------------------------------

info "Starting petalTongue in '${MODE}' mode …"
PT_PID=""

case "$MODE" in
    ui)
        "$PETALTONGUE_BIN" ui --scenario "$SCENARIO_FILE" &
        PT_PID=$!
        sleep 2
        ;;
    tui)
        "$PETALTONGUE_BIN" tui --scenario "$SCENARIO_FILE" &
        PT_PID=$!
        sleep 2
        ;;
    web)
        "$PETALTONGUE_BIN" web --bind "127.0.0.1:${PORT}" --scenario "$SCENARIO_FILE" &
        PT_PID=$!

        for _ in $(seq 1 15); do
            if curl -sf "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
                ok "petalTongue web ready at http://127.0.0.1:${PORT}"
                break
            fi
            sleep 1
        done
        ;;
    *)
        die "Unknown mode: ${MODE}. Use ui, tui, or web."
        ;;
esac

if ! kill -0 "$PT_PID" 2>/dev/null; then
    die "petalTongue exited unexpectedly"
fi

# --- 5. Start streaming data to petalTongue ------------------------------

info "Starting live data stream …"
info "  Simulating: ECG streaming, HRV rolling window, PK infusion monitor"

"$STREAMER_BIN" &
STREAMER_PID=$!

info "Press Ctrl-C to stop"
wait "$STREAMER_PID" 2>/dev/null || true

info "Streaming session complete"
