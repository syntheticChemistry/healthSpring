#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# fetch_primals.sh — Pull plasmidBin primal binaries for gate deployment
#
# Thin wrapper around primalSpring/tools/fetch_primals.sh or plasmidBin/fetch.sh.
# Ensures all 13 NUCLEUS primals + primalspring_primal are available locally.
#
# Usage:
#   ./tools/fetch_primals.sh           # fetch all primals
#   ./tools/fetch_primals.sh --force   # re-download even if present
#   ./tools/fetch_primals.sh --check   # just verify presence

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ECO_ROOT="$(cd "$PROJECT_ROOT/../.." && pwd)"

PLASMID_BIN="${ECOPRIMALS_PLASMID_BIN:-$ECO_ROOT/infra/plasmidBin}"
BIN_DIR="$PLASMID_BIN/primals"

NUCLEUS_PRIMALS=(
    beardog songbird toadstool barracuda coralreef
    nestgate rhizocrypt loamspine sweetgrass
    biomeos squirrel skunkbat petaltongue
)

CHECK_ONLY=false
FORCE=false

for arg in "$@"; do
    case "$arg" in
        --check) CHECK_ONLY=true ;;
        --force) FORCE=true ;;
        --help|-h)
            echo "Usage: $0 [--check|--force]"
            echo ""
            echo "  --check   Verify all primals are present without downloading"
            echo "  --force   Re-download even if present"
            exit 0
            ;;
    esac
done

echo "━━━ healthSpring: Fetch Primals (gate deployment) ━━━"
echo "Binary dir: $BIN_DIR"
echo ""

missing=0
present=0

for primal in "${NUCLEUS_PRIMALS[@]}"; do
    if [[ -x "$BIN_DIR/$primal" ]]; then
        present=$((present + 1))
        echo "  [OK] $primal"
    else
        missing=$((missing + 1))
        echo "  [!!] $primal — MISSING"
    fi
done

# Also check primalspring_primal (ionic bridge coordinator)
PS_BIN="$ECO_ROOT/springs/primalSpring/target/release/primalspring_primal"
if [[ -x "$PS_BIN" ]]; then
    present=$((present + 1))
    echo "  [OK] primalspring_primal (from primalSpring/target/release/)"
else
    missing=$((missing + 1))
    echo "  [!!] primalspring_primal — MISSING (build primalSpring or fetch)"
fi

echo ""
echo "Present: $present / $((present + missing))"

if [[ $missing -eq 0 ]]; then
    echo "All primals available. Gate ready for deployment."
    exit 0
fi

if $CHECK_ONLY; then
    echo ""
    echo "Missing $missing primal(s). Run without --check to fetch."
    exit 1
fi

# Try upstream fetch script
UPSTREAM_FETCH="$ECO_ROOT/springs/primalSpring/tools/fetch_primals.sh"
if [[ -x "$UPSTREAM_FETCH" ]]; then
    echo ""
    echo "Using primalSpring/tools/fetch_primals.sh..."
    ARGS=("--all")
    $FORCE && ARGS+=("--force")
    "$UPSTREAM_FETCH" "${ARGS[@]}"
else
    # Fallback: plasmidBin fetch.sh
    PLASMID_FETCH="$PLASMID_BIN/fetch.sh"
    if [[ -x "$PLASMID_FETCH" ]]; then
        echo "Using plasmidBin/fetch.sh..."
        "$PLASMID_FETCH"
    else
        echo "ERROR: No fetch script found. Options:"
        echo "  1. Run: springs/primalSpring/tools/fetch_primals.sh --all"
        echo "  2. Build primals from source"
        exit 1
    fi
fi
