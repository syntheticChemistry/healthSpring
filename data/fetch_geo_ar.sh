#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Fetch GEO series matrix (placeholder GSE28680 — AR-related expression panel).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_ROOT="${HEALTHSPRING_DATA_ROOT:-$SCRIPT_DIR}"
COLD="${HEALTHSPRING_COLD_STORAGE:-}"

GSE="GSE28680"
FTP_URL="https://ftp.ncbi.nlm.nih.gov/geo/series/GSE28nnn/${GSE}/matrix/${GSE}_series_matrix.txt.gz"
REL_OUT="geo"
OUT="${DATA_ROOT%/}/${REL_OUT}"
OUT_FILE="${OUT}/${GSE}_series_matrix.txt.gz"
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
  [[ -f "$SUMS" && -f "$OUT_FILE" ]] || return 1
  local exp rel
  while read -r exp rel _; do
    [[ -z "${exp:-}" ]] && continue
    [[ "$exp" =~ ^# ]] && continue
    [[ "$rel" != "$(basename "$OUT_FILE")" ]] && continue
    local got
    got="$(hash_file "$OUT_FILE")"
    [[ "$got" == "$exp" ]] || return 1
    return 0
  done <"$SUMS"
  return 1
}

write_sums() {
  mkdir -p "$OUT"
  printf '%s  %s\n' "$(hash_file "$OUT_FILE")" "$(basename "$OUT_FILE")" >"$SUMS"
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

main() {
  check_deps
  if ! command -v b3sum >/dev/null 2>&1 && ! python3 -c "from blake3 import blake3" 2>/dev/null; then
    echo "blake3: install b3sum or: pip install blake3" >&2
    exit 1
  fi

  mkdir -p "$OUT"

  if verify_sums 2>/dev/null; then
    echo "# geo_ar: existing matrix matches ${SUMS}; skip download."
  elif sync_from_cold && verify_sums 2>/dev/null; then
    echo "# geo_ar: synced from HEALTHSPRING_COLD_STORAGE; skip network."
  else
    curl --fail --silent --show-error --location \
      -o "${OUT_FILE}.part" "$FTP_URL"
    mv -f "${OUT_FILE}.part" "$OUT_FILE"
    write_sums
  fi

  local h
  h="$(hash_file "$OUT_FILE")"
  echo ""
  echo "# --- provenance (${FETCHED_AT}) ---"
  echo "# geo_series = \"${GSE}\""
  echo "# ftp_url = \"${FTP_URL}\""
  echo "# output_relpath = \"${REL_OUT}/$(basename "$OUT_FILE")\""
  echo "# blake3 = \"${h}\""
  echo ""
  echo "# manifest.toml fragment:"
  echo "[datasets.geo_androgen_receptor.blake3]"
  printf '"%s_series_matrix.txt.gz" = "%s"\n' "$GSE" "$h"
}

main "$@"
