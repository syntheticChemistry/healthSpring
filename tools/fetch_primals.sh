#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# fetch_primals.sh — Verify plasmidBin primal binaries for gate deployment
#
# Post-primordial: all primal binaries MUST come from plasmidBin.
# No target/release/ fallbacks. No cargo build for primals.
#
# Usage:
#   ./tools/fetch_primals.sh           # verify all primals present
#   ./tools/fetch_primals.sh --check   # same (explicit)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ECO_ROOT="$(cd "$PROJECT_ROOT/../.." && pwd)"

PLASMID_BIN="${ECOPRIMALS_PLASMID_BIN:-$ECO_ROOT/infra/plasmidBin}"
TRIPLE="x86_64-unknown-linux-musl"
BIN_DIR="$PLASMID_BIN/primals/$TRIPLE"

NUCLEUS_PRIMALS=(
    beardog songbird toadstool barracuda coralreef
    nestgate rhizocrypt loamspine sweetgrass
    biomeos squirrel skunkbat petaltongue
)

for arg in "$@"; do
    case "$arg" in
        --check) ;; # default behavior
        --help|-h)
            echo "Usage: $0 [--check]"
            echo ""
            echo "Verifies all NUCLEUS primals exist in plasmidBin."
            echo "Binary dir: $BIN_DIR"
            exit 0
            ;;
    esac
done

echo "━━━ healthSpring: Verify Primals (plasmidBin only) ━━━"
echo "Binary dir: $BIN_DIR"
echo ""

if [[ ! -d "$BIN_DIR" ]]; then
    echo "ERROR: plasmidBin triple dir not found: $BIN_DIR"
    echo "  Run: cd infra/plasmidBin && git pull"
    exit 1
fi

missing=0
present=0

for primal in "${NUCLEUS_PRIMALS[@]}"; do
    if [[ -x "$BIN_DIR/$primal" ]]; then
        present=$((present + 1))
        echo "  [OK] $primal"
    else
        missing=$((missing + 1))
        echo "  [!!] $primal — MISSING from plasmidBin"
    fi
done

echo ""
echo "Present: $present / ${#NUCLEUS_PRIMALS[@]}"

if [[ $missing -eq 0 ]]; then
    echo "All NUCLEUS primals available in plasmidBin. Gate ready."
    exit 0
else
    echo ""
    echo "Missing $missing primal(s). Pull plasmidBin:"
    echo "  cd infra/plasmidBin && git pull"
    exit 1
fi
