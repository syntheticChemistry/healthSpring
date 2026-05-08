#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Validate that every dotted method string in healthSpring Rust source appears
# in config/capability_registry.toml and vice versa.
#
# Follows the primalSpring pattern (tools/check_method_strings.sh).

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REGISTRY="$REPO_ROOT/config/capability_registry.toml"

if [[ ! -f "$REGISTRY" ]]; then
    echo "FATAL: $REGISTRY not found" >&2
    exit 1
fi

registry_methods() {
    grep -E '^\s*"[a-z][a-z0-9_]*\.[a-z]' "$REGISTRY" \
        | sed 's/.*"\([^"]*\)".*/\1/' \
        | sort -u
}

test_fixture_methods() {
    awk '/^\[test_fixtures\]/,/^\[/' "$REGISTRY" \
        | grep -E '^\s*"[a-z]' \
        | sed 's/.*"\([^"]*\)".*/\1/' \
        | sort -u
}

false_positive_methods() {
    awk '/^\[false_positives\]/,/^\[/' "$REGISTRY" \
        | grep -E '^\s*"[a-z]' \
        | sed 's/.*"\([^"]*\)".*/\1/' \
        | sort -u
}

exclude_pattern() {
    { test_fixture_methods; false_positive_methods; } | paste -sd'|' -
}

source_methods() {
    local excl
    excl="$(exclude_pattern)"

    rg -o '"[a-z][a-z0-9_]*\.[a-z][a-z0-9_.]*"' \
        --glob '*.rs' \
        --glob '!target/**' \
        "$REPO_ROOT/ecoPrimal/src" \
        "$REPO_ROOT/ecoPrimal/tests" \
        "$REPO_ROOT/experiments" \
    | sed 's/.*"\([^"]*\)".*/\1/' \
    | grep -Ev "^($excl)$" \
    | sort -u
}

errors=0

echo "=== Registry → Source (methods in registry but not in Rust) ==="
while IFS= read -r m; do
    if ! source_methods | grep -qxF "$m"; then
        echo "  WARN: $m in registry but not found in source (may be consumed-only)"
    fi
done < <(registry_methods | grep -Ev "^($(exclude_pattern))$")

echo ""
echo "=== Source → Registry (methods in Rust but not in registry) ==="
while IFS= read -r m; do
    if ! registry_methods | grep -qxF "$m"; then
        echo "  ERROR: $m in source but missing from registry"
        errors=$((errors + 1))
    fi
done < <(source_methods)

if [[ $errors -gt 0 ]]; then
    echo ""
    echo "FAIL: $errors method(s) in source not in registry"
    exit 1
fi

echo ""
echo "PASS: all source method strings found in registry"
