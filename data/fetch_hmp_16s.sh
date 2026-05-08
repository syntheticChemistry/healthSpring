#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Fetch SRA run-info table for HMP BioProject PRJNA48479 via NCBI E-utilities
# (esearch history server + efetch rettype=runinfo).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_ROOT="${HEALTHSPRING_DATA_ROOT:-$SCRIPT_DIR}"
COLD="${HEALTHSPRING_COLD_STORAGE:-}"

BIOPROJECT="PRJNA48479"
ES_URL="https://eutils.ncbi.nlm.nih.gov/entrez/eutils"
REL_OUT="hmp"
OUT="${DATA_ROOT%/}/${REL_OUT}"
CSV_OUT="${OUT}/sra_runinfo_${BIOPROJECT}.csv"
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
  [[ -f "$SUMS" && -f "$CSV_OUT" ]] || return 1
  local exp rel
  while read -r exp rel _; do
    [[ -z "${exp:-}" ]] && continue
    [[ "$exp" =~ ^# ]] && continue
    [[ "$rel" != "$(basename "$CSV_OUT")" ]] && continue
    local got
    got="$(hash_file "$CSV_OUT")"
    [[ "$got" == "$exp" ]] || return 1
    return 0
  done <"$SUMS"
  return 1
}

write_sums() {
  mkdir -p "$OUT"
  printf '%s  %s\n' "$(hash_file "$CSV_OUT")" "$(basename "$CSV_OUT")" >"$SUMS"
}

check_deps() {
  command -v curl >/dev/null 2>&1 || {
    echo "curl is required" >&2
    exit 1
  }
  command -v python3 >/dev/null 2>&1 || {
    echo "python3 is required (WebEnv URL encoding)" >&2
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

efetch_runinfo() {
  local xml
  xml="$(curl --fail --silent --show-error \
    "${ES_URL}/esearch.fcgi?db=sra&term=${BIOPROJECT}&retmax=0&usehistory=y")"

  local query_key webenv
  query_key="$(printf '%s\n' "$xml" | sed -n 's/.*<QueryKey>\([^<]*\)<\/QueryKey>.*/\1/p')"
  webenv="$(printf '%s\n' "$xml" | sed -n 's/.*<WebEnv>\([^<]*\)<\/WebEnv>.*/\1/p')"
  if [[ -z "$query_key" || -z "$webenv" ]]; then
    echo "hmp: failed to parse esearch history (QueryKey/WebEnv)" >&2
    exit 1
  fi

  local enc_we
  enc_we="$(python3 -c "import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1], safe=''))" "$webenv")"

  curl --fail --silent --show-error \
    "${ES_URL}/efetch.fcgi?db=sra&query_key=${query_key}&WebEnv=${enc_we}&rettype=runinfo&retmode=text" \
    -o "${CSV_OUT}.part"
  mv -f "${CSV_OUT}.part" "$CSV_OUT"
}

main() {
  check_deps
  if ! command -v b3sum >/dev/null 2>&1 && ! python3 -c "from blake3 import blake3" 2>/dev/null; then
    echo "blake3: install b3sum or: pip install blake3" >&2
    exit 1
  fi

  mkdir -p "$OUT"

  if verify_sums 2>/dev/null; then
    echo "# hmp: existing CSV matches ${SUMS}; skip download."
  elif sync_from_cold && verify_sums 2>/dev/null; then
    echo "# hmp: synced from HEALTHSPRING_COLD_STORAGE; skip network."
  else
    efetch_runinfo
    write_sums
  fi

  local h
  h="$(hash_file "$CSV_OUT")"
  echo ""
  echo "# --- provenance (${FETCHED_AT}) ---"
  echo "# bioproject = \"${BIOPROJECT}\""
  echo "# ncbi_esearch = \"${ES_URL}/esearch.fcgi?db=sra&term=${BIOPROJECT}&usehistory=y\""
  echo "# ncbi_efetch_runinfo = \"${ES_URL}/efetch.fcgi?db=sra&rettype=runinfo\""
  echo "# output_relpath = \"${REL_OUT}/$(basename "$CSV_OUT")\""
  echo "# blake3 = \"${h}\""
  echo ""
  echo "# manifest.toml fragment:"
  echo "[datasets.hmp_16s.blake3]"
  printf '"sra_runinfo_%s.csv" = "%s"\n' "$BIOPROJECT" "$h"
}

main "$@"
