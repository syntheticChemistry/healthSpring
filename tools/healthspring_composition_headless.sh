#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# healthspring_composition_headless.sh — Non-interactive NUCLEUS composition
#
# Runs healthSpring's full validation against a live NUCLEUS, records
# provenance, prints results, and exits. Designed for CI and headless
# environments where petalTongue interaction isn't available.
#
# Usage:
#   PATH="$(pwd)/tools:$PATH" COMPOSITION_NAME=healthspring ./tools/healthspring_composition_headless.sh

set -euo pipefail

COMPOSITION_NAME="${COMPOSITION_NAME:-healthspring}"
FAMILY_ID="${FAMILY_ID:-$COMPOSITION_NAME}"
REQUIRED_CAPS="visualization security"
OPTIONAL_CAPS="compute tensor dag ledger attribution storage"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/nucleus_composition_lib.sh"

# ── Domain State ──────────────────────────────────────────────────────

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0
TOTAL_CHECKS=0
declare -a VALIDATION_RESULTS=()
declare -a GAP_NOTES=()

MATH_DATA='[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]'
EXPECTED_MEAN=5.5
EXPECTED_STDDEV=3.0276503540974917
EXPECTED_VARIANCE=9.166666666666668
EXPECTED_CORRELATION=1.0

# ── Validation Helpers ────────────────────────────────────────────────

record_pass() {
    local label="$1"
    PASS_COUNT=$((PASS_COUNT + 1))
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    VALIDATION_RESULTS+=("PASS  $label")
    ok "PASS  $label"
}

record_fail() {
    local label="$1" detail="${2:-}"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    VALIDATION_RESULTS+=("FAIL  $label${detail:+ ($detail)}")
    err "FAIL  $label${detail:+ ($detail)}"
}

record_skip() {
    local label="$1" reason="${2:-offline}"
    SKIP_COUNT=$((SKIP_COUNT + 1))
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    VALIDATION_RESULTS+=("SKIP  $label ($reason)")
    warn "SKIP  $label ($reason)"
}

record_gap() {
    GAP_NOTES+=("$1")
    warn "GAP: $1"
}

run_math_check() {
    local method="$1" expected="$2" label="$3"
    if ! cap_available tensor; then
        record_skip "$label" "tensor offline"
        return
    fi

    local params
    case "$method" in
        stats.mean|stats.std_dev|stats.variance)
            params="{\"data\":$MATH_DATA}" ;;
        stats.correlation)
            params="{\"x\":$MATH_DATA,\"y\":$MATH_DATA}" ;;
    esac

    local resp
    resp=$(send_rpc "$(cap_socket tensor)" "$method" "$params" 2>/dev/null || echo "")

    if [[ -z "$resp" ]]; then
        record_fail "$label" "no response"
        return
    fi

    local result
    result=$(echo "$resp" | python3 -c '
import json,sys
try:
    d=json.load(sys.stdin)
    r=d.get("result",d)
    if isinstance(r,dict): print(r.get("result",r.get("value","")))
    else: print(r)
except: print("")
' 2>/dev/null || echo "")

    if [[ -z "$result" ]]; then
        record_fail "$label" "parse error: $(echo "$resp" | head -c 80)"
        return
    fi

    local pass_check
    pass_check=$(python3 -c "
r=float('$result'); e=float('$expected')
print('yes' if abs(r-e) < 1e-6 else 'no')
print(abs(r-e))
" 2>/dev/null || echo -e "no\n999")

    local verdict diff
    verdict=$(echo "$pass_check" | head -1)
    diff=$(echo "$pass_check" | tail -1)

    if [[ "$verdict" = "yes" ]]; then
        record_pass "$label (ipc=$result, diff=$diff)"
    else
        record_fail "$label" "ipc=$result, expected=$expected, diff=$diff"
    fi
}

# ── Main ──────────────────────────────────────────────────────────────

main() {
    log "============================================"
    log "  healthSpring Composition Validation"
    log "  (headless/non-interactive mode)"
    log "============================================"

    discover_capabilities || { err "Required primals not found"; exit 1; }

    # Push a startup scene to exercise petalTongue
    composition_startup "healthSpring NUCLEUS" "Headless Validation"

    # ── DAG Session ──
    dag_create_session "$COMPOSITION_NAME" "[]"

    # ── Ledger Spine ──
    ledger_create_spine

    # ── Tier 1: Capability Discovery ──
    log "── Tier 1: Capability Discovery Validation ──"

    for cap_name in visualization security compute tensor dag ledger attribution storage; do
        if cap_available "$cap_name"; then
            record_pass "capability.discover: $cap_name"
        else
            if [[ "$cap_name" = "visualization" || "$cap_name" = "security" ]]; then
                record_fail "capability.discover: $cap_name (required)"
            else
                record_skip "capability.discover: $cap_name"
            fi
        fi
    done

    # ── Tier 2: Liveness Probes ──
    log "── Tier 2: Liveness Probes ──"

    for cap_name in visualization security compute tensor; do
        if ! cap_available "$cap_name"; then
            record_skip "${cap_name}.liveness"
            continue
        fi
        local sock resp
        sock=$(cap_socket "$cap_name")
        resp=$(send_rpc "$sock" "health.liveness" "{}" 2>/dev/null || echo "")
        if [[ -n "$resp" ]]; then
            record_pass "${cap_name}.liveness (responded)"
        else
            record_fail "${cap_name}.liveness" "socket exists, no response"
            record_gap "${cap_name}: health.liveness returns empty response"
        fi
    done

    # ── Tier 2: barraCuda Math IPC Parity ──
    log "── Tier 2: barraCuda Math IPC ──"

    run_math_check "stats.mean" "$EXPECTED_MEAN" "stats.mean IPC parity"
    run_math_check "stats.std_dev" "$EXPECTED_STDDEV" "stats.std_dev IPC parity"
    run_math_check "stats.variance" "$EXPECTED_VARIANCE" "stats.variance IPC parity"
    run_math_check "stats.correlation" "$EXPECTED_CORRELATION" "stats.correlation self-parity"

    # ── Tier 2: petalTongue Scene Push ──
    log "── Tier 2: petalTongue Visualization ──"

    if cap_available visualization; then
        local title_node
        title_node=$(make_text_node "val-title" 230 100 "healthSpring Validation" 24 0.95 0.95 1.0)
        local result_text="Checks: $TOTAL_CHECKS total"
        local result_node
        result_node=$(make_text_node "val-result" 230 140 "$result_text" 16 0.8 0.8 0.85)
        local root
        root=$(printf '"root":{"id":"root","transform":{"a":1.0,"b":0.0,"c":0.0,"d":1.0,"tx":0.0,"ty":0.0},"primitives":[],"children":["val-title","val-result"],"visible":true,"opacity":1.0,"label":null,"data_source":null}')
        local scene="{\"nodes\":{${root},${title_node},${result_node}},\"root_id\":\"root\"}"
        push_scene "${COMPOSITION_NAME}-validation" "$scene"
        record_pass "petalTongue.scene.push"
    else
        record_skip "petalTongue.scene.push"
    fi

    # ── Tier 2: BearDog Crypto Sign ──
    log "── Tier 2: BearDog Crypto ──"

    if cap_available security; then
        local sig_resp
        sig_resp=$(send_rpc "$(cap_socket security)" "crypto.sign" \
            "{\"message\":\"$(echo -n "healthspring-validation-$(date +%s)" | base64)\"}" 2>/dev/null || echo "")
        if echo "$sig_resp" | grep -q '"signature"'; then
            local sig
            sig=$(echo "$sig_resp" | grep -o '"signature":"[^"]*"' | head -1 | cut -d'"' -f4 || true)
            record_pass "beardog.crypto.sign (sig=${sig:0:16}...)"
        else
            record_fail "beardog.crypto.sign" "$(echo "$sig_resp" | head -c 80)"
        fi
    else
        record_skip "beardog.crypto.sign"
    fi

    # ── Tier 2: ToadStool Compute Capabilities ──
    log "── Tier 2: ToadStool Compute ──"

    if cap_available compute; then
        local compute_resp
        compute_resp=$(send_rpc "$(cap_socket compute)" "compute.capabilities" "{}" 2>/dev/null || echo "")
        if echo "$compute_resp" | grep -q '"compute_units"'; then
            local cores
            cores=$(echo "$compute_resp" | python3 -c '
import json,sys
try:
    d=json.load(sys.stdin)
    r=d.get("result",d)
    print(r.get("available_resources",{}).get("total_cpu_cores","?"))
except: print("?")
' 2>/dev/null || echo "?")
            record_pass "toadstool.compute.capabilities (${cores} cores)"
        else
            record_fail "toadstool.compute.capabilities" "$(echo "$compute_resp" | head -c 80)"
        fi
    else
        record_skip "toadstool.compute.capabilities"
    fi

    # ── Tier 2: Storage Round-Trip ──
    log "── Tier 2: Storage Round-Trip ──"

    if cap_available storage; then
        local key="healthspring_composition_probe_$(date +%s)"
        local val='{"probe":true,"composition":"healthspring"}'
        local store_resp
        store_resp=$(send_rpc "$(cap_socket storage)" "storage.store" \
            "{\"key\":\"$key\",\"value\":$val,\"family_id\":\"$FAMILY_ID\"}" 2>/dev/null || echo "")
        local retrieve_resp
        retrieve_resp=$(send_rpc "$(cap_socket storage)" "storage.retrieve" \
            "{\"key\":\"$key\",\"family_id\":\"$FAMILY_ID\"}" 2>/dev/null || echo "")
        if echo "$retrieve_resp" | grep -q '"probe":true'; then
            record_pass "storage round-trip"
        else
            record_fail "storage round-trip" "store=$(echo "$store_resp" | head -c 40) ret=$(echo "$retrieve_resp" | head -c 40)"
        fi
    else
        record_skip "storage round-trip" "storage offline"
    fi

    # ── Tier 2: Provenance Trio (DAG + Ledger + Braid) ──
    log "── Tier 2: Provenance Trio ──"

    if cap_available dag; then
        if [[ -n "$DAG_SESSION" ]]; then
            record_pass "rhizocrypt.dag.session.create"
            dag_append_event "$COMPOSITION_NAME" "validation" "running" \
                "[{\"key\":\"phase\",\"value\":\"headless\"}]" "automated" "0"
            if [[ -n "$CURRENT_VERTEX" ]]; then
                record_pass "rhizocrypt.dag.event.append"
            else
                record_fail "rhizocrypt.dag.event.append" "no vertex returned"
                record_gap "rhizoCrypt: dag.event.append returns empty on UDS"
            fi
        else
            record_fail "rhizocrypt.dag.session.create" "empty response"
            record_gap "rhizoCrypt: dag.session.create returns empty on UDS (PG-45 / PG-06)"
        fi
    else
        record_skip "rhizocrypt.dag" "dag offline"
    fi

    if cap_available ledger; then
        if [[ -n "$SPINE_ID" ]]; then
            record_pass "loamspine.spine.create"
        else
            record_fail "loamspine.spine.create" "empty response"
            record_gap "loamSpine: spine.create returns empty on UDS"
        fi
    else
        record_skip "loamspine.ledger" "ledger offline"
    fi

    if cap_available attribution; then
        braid_record "validation" "application/x-healthspring" "headless" \
            "{\"mode\":\"headless\",\"total\":$TOTAL_CHECKS}" "automated" "0"
        if [[ -n "$LAST_BRAID_ID" ]]; then
            record_pass "sweetgrass.braid.create"
        else
            record_fail "sweetgrass.braid.create" "empty response"
            record_gap "sweetGrass: braid.create returns empty on UDS"
        fi
    else
        record_skip "sweetgrass.braid" "attribution offline"
    fi

    # ── Tier 2: Proprioception ──
    log "── Tier 2: Proprioception ──"

    if cap_available visualization; then
        local proprio
        proprio=$(poll_proprioception)
        if echo "$proprio" | grep -q '"frame_rate"'; then
            local fps
            fps=$(echo "$proprio" | grep -oP '"frame_rate"\s*:\s*\K[0-9.]+' | head -1 || echo "?")
            record_pass "petalTongue.proprioception (fps=$fps)"
        else
            record_fail "petalTongue.proprioception" "no frame_rate in response"
            record_gap "petalTongue: proprioception.get returns no frame_rate in headless/server mode"
        fi
    else
        record_skip "petalTongue.proprioception"
    fi

    # ── Seal Ledger ──
    if cap_available ledger && [[ -n "$SPINE_ID" ]]; then
        ledger_seal_spine
    fi

    # ── Summary ──
    log "============================================"
    log "  VALIDATION SUMMARY"
    log "============================================"
    for line in "${VALIDATION_RESULTS[@]}"; do
        log "  $line"
    done
    log "--------------------------------------------"
    log "  PASS: $PASS_COUNT  FAIL: $FAIL_COUNT  SKIP: $SKIP_COUNT  TOTAL: $TOTAL_CHECKS"
    log "============================================"

    if (( ${#GAP_NOTES[@]} > 0 )); then
        log ""
        log "── Discovered Gaps ──"
        for gap in "${GAP_NOTES[@]}"; do
            log "  • $gap"
        done
    fi

    # ── Composition Summary (from lib) ──
    composition_summary

    # ── Teardown ──
    dismiss_scene "${COMPOSITION_NAME}-validation"
    composition_teardown "${COMPOSITION_NAME}-splash"

    if (( FAIL_COUNT > 0 )); then
        exit 1
    fi
}

main "$@"
