#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sync healthSpring scenario JSONs to petalTongue's sandbox/scenarios/.
# This makes all healthSpring scenarios available to petalTongue demos
# and the showcase without manual copying.
#
# Usage:
#   ./scripts/sync_scenarios.sh                    # auto-detect petalTongue
#   PETALTONGUE_ROOT=/path/to ./scripts/sync_scenarios.sh   # explicit path

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PETALTONGUE_ROOT="${PETALTONGUE_ROOT:-$(cd "${PROJECT_ROOT}/../phase2/petalTongue" 2>/dev/null && pwd || true)}"

info()  { echo -e "\033[1;36m[sync]\033[0m $*"; }
ok()    { echo -e "\033[1;32m  ✓\033[0m $*"; }
err()   { echo -e "\033[1;31m  ✗\033[0m $*"; }
die()   { err "$*"; exit 1; }

SRC="${PROJECT_ROOT}/sandbox/scenarios"
DST="${PETALTONGUE_ROOT}/sandbox/scenarios"

if [[ ! -d "$SRC" ]]; then
    die "Source directory not found: ${SRC}"
fi

if [[ -z "$PETALTONGUE_ROOT" ]] || [[ ! -d "$PETALTONGUE_ROOT" ]]; then
    die "petalTongue not found. Set PETALTONGUE_ROOT env var."
fi

if [[ ! -d "$DST" ]]; then
    die "petalTongue sandbox/scenarios/ not found at ${DST}"
fi

info "Syncing healthSpring scenarios → petalTongue"
info "  From: ${SRC}"
info "  To:   ${DST}"
echo

SYNCED=0
for src_file in "${SRC}"/healthspring-*.json "${SRC}"/clinical-trt-*.json; do
    [[ -f "$src_file" ]] || continue
    filename="$(basename "$src_file")"
    dst_file="${DST}/${filename}"

    if [[ -f "$dst_file" ]] && cmp -s "$src_file" "$dst_file"; then
        echo "  skip  ${filename} (unchanged)"
    else
        cp "$src_file" "$dst_file"
        ok "${filename}"
        SYNCED=$((SYNCED + 1))
    fi
done

echo
if [[ $SYNCED -eq 0 ]]; then
    info "All scenarios already in sync"
else
    info "Synced ${SYNCED} scenario(s)"
fi
