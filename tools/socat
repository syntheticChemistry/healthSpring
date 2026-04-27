#!/usr/bin/env bash
# socat shim — translates `socat - UNIX-CONNECT:path` to `nc -U path`
# Used by nucleus_composition_lib.sh on systems without socat installed.
for arg in "$@"; do
    if [[ "$arg" =~ ^UNIX-CONNECT:(.+)$ ]]; then
        sock="${BASH_REMATCH[1]}"
        exec nc -q 1 -U "$sock"
    fi
done
echo "socat shim: unrecognized args: $*" >&2
exit 1
