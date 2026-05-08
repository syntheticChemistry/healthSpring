#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Fetch MIT-BIH Arrhythmia Database subset from PhysioNet (WFDB records).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_ROOT="${HEALTHSPRING_DATA_ROOT:-$SCRIPT_DIR}"
COLD="${HEALTHSPRING_COLD_STORAGE:-}"

BASE_URL="https://physionet.org/files/mitdb/1.0.0/"
RECORDS=(100 101 102 103)
REL_OUT="mitbih"
OUT="${DATA_ROOT%/}/${REL_OUT}"
SUMS="${OUT}/blake3sums.txt"
FETCHED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

hash_file() {
  local f="$1"
  if command -v b3sum >/dev/null 2>&1; then
    b3sum --length 32 "$f" | awk '{print $1}'
  elif python3 -c "from blake3 import blake3" 2>/dev/null; then
    python3 - "$f" <<'PY'
import sys
from blake3 import blake3
path = sys.argv[1]
h = blake3()
with open(path, "rb") as fp:
    for chunk in iter(lambda: fp.read(1048576), b""):
        h.update(chunk)
print(h.hexdigest())
PY
  else
    echo "blake3: install b3sum (BLAKE3 reference CLI) or: pip install blake3" >&2
    exit 1
  fi
}

verify_sums() {
  [[ -f "$SUMS" ]] || return 1
  while read -r exp rel _; do
    [[ -z "${exp:-}" ]] && continue
    [[ "$exp" =~ ^# ]] && continue
    local p="${OUT}/${rel}"
    [[ -f "$p" ]] || return 1
    local got
    got="$(hash_file "$p")"
    [[ "$got" == "$exp" ]] || return 1
  done <"$SUMS"
}

write_sums() {
  mkdir -p "$OUT"
  : >"$SUMS"
  for rec in "${RECORDS[@]}"; do
    for ext in hea dat; do
      local fn="${rec}.${ext}"
      printf '%s  %s\n' "$(hash_file "${OUT}/${fn}")" "$fn" >>"$SUMS"
    done
  done
}

check_deps() {
  command -v curl >/dev/null 2>&1 || {
    echo "curl is required" >&2
    exit 1
  }
}

sync_from_cold() {
  [[ -n "$COLD" ]] || return 1
  local src="${COLD%/}/${REL_OUT}"
  [[ -d "$src" ]] || return 1
  mkdir -p "$OUT"
  cp -a "${src}/." "${OUT}/"
}

download_one() {
  local rec="$1"
  local ext="$2"
  local fn="${rec}.${ext}"
  local url="${BASE_URL}${fn}"
  curl --fail --silent --show-error --location \
    -o "${OUT}/${fn}.part" "$url"
  mv -f "${OUT}/${fn}.part" "${OUT}/${fn}"
}

main() {
  check_deps
  # Ensure blake3 is available
  if ! command -v b3sum >/dev/null 2>&1 && ! python3 -c "from blake3 import blake3" 2>/dev/null; then
    echo "blake3: install b3sum or: pip install blake3" >&2
    exit 1
  fi

  mkdir -p "$OUT"

  if verify_sums 2>/dev/null; then
    echo "# mitbih: existing files match ${SUMS}; skip download."
  else
    if sync_from_cold && verify_sums 2>/dev/null; then
      echo "# mitbih: synced from HEALTHSPRING_COLD_STORAGE; skip network."
    else
      for rec in "${RECORDS[@]}"; do
        download_one "$rec" hea
        download_one "$rec" dat
      done
      write_sums
    fi
  fi

  echo ""
  echo "# --- provenance (${FETCHED_AT}) ---"
  echo "# physionet_base_url = \"${BASE_URL}\""
  echo "# records = $(printf '%s ' "${RECORDS[@]}")"
  while read -r h rel _; do
    [[ "$h" =~ ^# ]] && continue
    [[ -z "$h" ]] && continue
    echo "# blake3_${rel//./_} = \"${h}\""
  done <"$SUMS"
  echo ""
  echo "# manifest.toml fragment:"
  echo "[datasets.mitbih.blake3]"
  while read -r h rel _; do
    [[ "$h" =~ ^# ]] && continue
    [[ -z "$h" ]] && continue
    printf '"%s" = "%s"\n' "$rel" "$h"
  done <"$SUMS"
}

main "$@"
