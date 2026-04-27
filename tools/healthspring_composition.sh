#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# healthspring_composition.sh — NUCLEUS composition for health science validation
#
# Exercises healthSpring's three-tier validation (local → IPC → primal proof)
# against a live NUCLEUS, renders results in petalTongue, records provenance
# via DAG/ledger/braid.
#
# Usage:
#   COMPOSITION_NAME=healthspring FAMILY_ID=healthspring ./tools/healthspring_composition.sh
#
# Requires: NUCLEUS running (use tools/composition_nucleus.sh start)

set -euo pipefail

# ── 1. Configuration ──────────────────────────────────────────────────

COMPOSITION_NAME="${COMPOSITION_NAME:-healthspring}"
FAMILY_ID="${FAMILY_ID:-$COMPOSITION_NAME}"
REQUIRED_CAPS="visualization security"
OPTIONAL_CAPS="compute tensor dag ledger attribution storage"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/nucleus_composition_lib.sh"

# ── 2. Domain State ──────────────────────────────────────────────────

RUNNING=true
VALIDATION_PHASE="init"
VALIDATION_RESULTS=()
PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0
TOTAL_CHECKS=0

MATH_METHODS=("stats.mean" "stats.std_dev" "stats.variance" "stats.correlation")
MATH_DATA='[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]'
EXPECTED_MEAN=5.5
EXPECTED_STDDEV=3.0276503540974917

# Bessel's N-1 sample variance: std_dev^2
EXPECTED_VARIANCE=9.166666666666668
EXPECTED_CORRELATION=1.0

# ── 3. Hit Testing ────────────────────────────────────────────────────

# Three interactive buttons: [Run Validation] [View Results] [Quit]
hit_test_fn() {
    local px="$1" py="$2"
    px="${px%.*}"; py="${py%.*}"
    if (( px >= 50 && px < 200 && py >= 400 && py < 440 )); then
        echo 0  # Run Validation
    elif (( px >= 220 && px < 370 && py >= 400 && py < 440 )); then
        echo 1  # View Results
    elif (( px >= 390 && px < 460 && py >= 400 && py < 440 )); then
        echo 2  # Quit
    else
        echo -1
    fi
}

# ── 4. Validation Logic ──────────────────────────────────────────────

run_math_ipc_check() {
    local method="$1" expected="$2" label="$3"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    if ! cap_available tensor; then
        SKIP_COUNT=$((SKIP_COUNT + 1))
        VALIDATION_RESULTS+=("SKIP  $label (tensor offline)")
        return
    fi

    local params
    case "$method" in
        stats.mean|stats.std_dev|stats.variance)
            params="{\"data\":$MATH_DATA}"
            ;;
        stats.correlation)
            params="{\"x\":$MATH_DATA,\"y\":$MATH_DATA}"
            ;;
    esac

    local resp
    resp=$(send_rpc "$(cap_socket tensor)" "$method" "$params" 2>/dev/null || echo "")

    if [[ -z "$resp" ]]; then
        FAIL_COUNT=$((FAIL_COUNT + 1))
        VALIDATION_RESULTS+=("FAIL  $label (no response)")
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
        FAIL_COUNT=$((FAIL_COUNT + 1))
        VALIDATION_RESULTS+=("FAIL  $label (parse error: $(echo "$resp" | head -c 80))")
        return
    fi

    local diff
    diff=$(python3 -c "print(abs(float('$result') - float('$expected')))" 2>/dev/null || echo "999")

    local tol="1e-10"
    local pass
    pass=$(python3 -c "print('yes' if float('$diff') < float('$tol') else 'no')" 2>/dev/null || echo "no")

    if [[ "$pass" = "yes" ]]; then
        PASS_COUNT=$((PASS_COUNT + 1))
        VALIDATION_RESULTS+=("PASS  $label (ipc=$result, diff=${diff})")
        ok "$label: PASS (diff=$diff)"
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
        VALIDATION_RESULTS+=("FAIL  $label (ipc=$result, expected=$expected, diff=$diff)")
        err "$label: FAIL (diff=$diff)"
    fi
}

run_storage_check() {
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    if ! cap_available storage; then
        SKIP_COUNT=$((SKIP_COUNT + 1))
        VALIDATION_RESULTS+=("SKIP  storage round-trip (storage offline)")
        return
    fi

    local key="healthspring_composition_probe_$(date +%s)"
    local val='{"probe":true,"composition":"healthspring","ts":"'$(date -Iseconds)'"}'

    local store_resp
    store_resp=$(send_rpc "$(cap_socket storage)" "storage.store" \
        "{\"key\":\"$key\",\"value\":$val,\"family_id\":\"$FAMILY_ID\"}" 2>/dev/null || echo "")

    local retrieve_resp
    retrieve_resp=$(send_rpc "$(cap_socket storage)" "storage.retrieve" \
        "{\"key\":\"$key\",\"family_id\":\"$FAMILY_ID\"}" 2>/dev/null || echo "")

    if echo "$retrieve_resp" | grep -q '"probe":true'; then
        PASS_COUNT=$((PASS_COUNT + 1))
        VALIDATION_RESULTS+=("PASS  storage round-trip")
        ok "storage round-trip: PASS"
    else
        FAIL_COUNT=$((FAIL_COUNT + 1))
        VALIDATION_RESULTS+=("FAIL  storage round-trip ($(echo "$retrieve_resp" | head -c 80))")
        err "storage round-trip: FAIL"
    fi
}

run_liveness_checks() {
    for cap_name in security tensor storage; do
        TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
        if cap_available "$cap_name"; then
            local sock
            sock=$(cap_socket "$cap_name")
            local resp
            resp=$(send_rpc "$sock" "health.liveness" "{}" 2>/dev/null || echo "")
            if [[ -n "$resp" ]]; then
                PASS_COUNT=$((PASS_COUNT + 1))
                VALIDATION_RESULTS+=("PASS  ${cap_name}.liveness")
            else
                FAIL_COUNT=$((FAIL_COUNT + 1))
                VALIDATION_RESULTS+=("FAIL  ${cap_name}.liveness (no response)")
            fi
        else
            SKIP_COUNT=$((SKIP_COUNT + 1))
            VALIDATION_RESULTS+=("SKIP  ${cap_name}.liveness (offline)")
        fi
    done
}

run_full_validation() {
    VALIDATION_PHASE="running"
    VALIDATION_RESULTS=()
    PASS_COUNT=0; FAIL_COUNT=0; SKIP_COUNT=0; TOTAL_CHECKS=0

    log "── Tier 2: Liveness Probes ──"
    run_liveness_checks

    log "── Tier 2: barraCuda Math IPC ──"
    run_math_ipc_check "stats.mean" "$EXPECTED_MEAN" "stats.mean IPC parity"
    run_math_ipc_check "stats.std_dev" "$EXPECTED_STDDEV" "stats.std_dev IPC parity"
    run_math_ipc_check "stats.variance" "$EXPECTED_VARIANCE" "stats.variance IPC parity"
    run_math_ipc_check "stats.correlation" "$EXPECTED_CORRELATION" "stats.correlation self-parity"

    log "── Tier 2: Storage Round-Trip ──"
    run_storage_check

    VALIDATION_PHASE="complete"
    log "── Results: $PASS_COUNT pass, $FAIL_COUNT fail, $SKIP_COUNT skip / $TOTAL_CHECKS total ──"
}

# ── 5. Domain Hooks ──────────────────────────────────────────────────

domain_init() {
    dag_create_session "$COMPOSITION_NAME" "[]"
    ledger_create_spine
    domain_render "Ready — click [Run Validation] or press R"
}

domain_render() {
    local status="${1:-}"
    local title_node
    title_node=$(make_text_node "title" 230 40 "healthSpring NUCLEUS Composition" 24 0.95 0.95 1.0)

    local status_color_r=0.8 status_color_g=0.8 status_color_b=0.85
    if [[ "$VALIDATION_PHASE" = "complete" ]]; then
        if (( FAIL_COUNT == 0 )); then
            status_color_r=0.3; status_color_g=0.9; status_color_b=0.4
        else
            status_color_r=0.9; status_color_g=0.3; status_color_b=0.3
        fi
    fi

    local status_node
    status_node=$(make_text_node "status" 230 70 "$status" 14 $status_color_r $status_color_g $status_color_b)

    local result_nodes="" result_ids="" y=100 idx=0
    for line in "${VALIDATION_RESULTS[@]:-}"; do
        [[ -z "$line" ]] && continue
        local cr=0.7 cg=0.7 cb=0.7
        case "$line" in
            PASS*) cr=0.3; cg=0.85; cb=0.4 ;;
            FAIL*) cr=0.9; cg=0.3; cb=0.3 ;;
            SKIP*) cr=0.7; cg=0.7; cb=0.3 ;;
        esac
        local nid="r-${idx}"
        local node
        node=$(make_text_node "$nid" 230 $y "$line" 12 $cr $cg $cb)
        if [[ -n "$result_nodes" ]]; then
            result_nodes="${result_nodes},${node}"
            result_ids="${result_ids},\"${nid}\""
        else
            result_nodes="${node}"
            result_ids="\"${nid}\""
        fi
        y=$((y + 18))
        idx=$((idx + 1))
    done

    local summary_node=""
    local summary_id=""
    if [[ "$VALIDATION_PHASE" = "complete" ]]; then
        local sum_text="$PASS_COUNT/$TOTAL_CHECKS pass ($SKIP_COUNT skip, $FAIL_COUNT fail)"
        summary_node=$(make_text_node "summary" 230 $((y + 10)) "$sum_text" 16 $status_color_r $status_color_g $status_color_b)
        summary_id=",\"summary\""
    fi

    local btn_run
    btn_run=$(make_text_node "btn-run" 125 420 "[R] Run Validation" 13 0.3 0.7 0.9)
    local btn_view
    btn_view=$(make_text_node "btn-view" 295 420 "[V] View Results" 13 0.6 0.6 0.7)
    local btn_quit
    btn_quit=$(make_text_node "btn-quit" 425 420 "[Q] Quit" 13 0.7 0.4 0.4)

    local children="\"title\",\"status\""
    [[ -n "$result_ids" ]] && children="${children},${result_ids}"
    [[ -n "$summary_id" ]] && children="${children}${summary_id}"
    children="${children},\"btn-run\",\"btn-view\",\"btn-quit\""

    local all_nodes="${title_node},${status_node}"
    [[ -n "$result_nodes" ]] && all_nodes="${all_nodes},${result_nodes}"
    [[ -n "$summary_node" ]] && all_nodes="${all_nodes},${summary_node}"
    all_nodes="${all_nodes},${btn_run},${btn_view},${btn_quit}"

    local root
    root=$(printf '"root":{"id":"root","transform":{"a":1.0,"b":0.0,"c":0.0,"d":1.0,"tx":0.0,"ty":0.0},"primitives":[],"children":[%s],"visible":true,"opacity":1.0,"label":null,"data_source":null}' "$children")

    local scene="{\"nodes\":{${root},${all_nodes}},\"root_id\":\"root\"}"
    push_scene "${COMPOSITION_NAME}-main" "$scene"
}

domain_on_key() {
    local key="$1"
    case "$key" in
        Q|q|Escape)
            log "quit requested"
            RUNNING=false
            ;;
        R|r)
            domain_render "Running validation..."
            run_full_validation

            dag_append_event "$COMPOSITION_NAME" "validation_run" "$VALIDATION_PHASE" \
                "[{\"key\":\"pass\",\"value\":\"$PASS_COUNT\"},{\"key\":\"fail\",\"value\":\"$FAIL_COUNT\"},{\"key\":\"skip\",\"value\":\"$SKIP_COUNT\"}]" \
                "keyboard" "0"

            braid_record "validation_run" "application/x-healthspring-validation" \
                "$PASS_COUNT/$TOTAL_CHECKS" \
                "{\"pass\":$PASS_COUNT,\"fail\":$FAIL_COUNT,\"skip\":$SKIP_COUNT,\"total\":$TOTAL_CHECKS}" \
                "keyboard" "0"

            if cap_available ledger && [[ -n "$SPINE_ID" ]]; then
                ledger_append_entry "validation-$COMPOSITION_NAME" \
                    "{\"pass\":$PASS_COUNT,\"fail\":$FAIL_COUNT,\"skip\":$SKIP_COUNT,\"total\":$TOTAL_CHECKS,\"ts\":\"$(date -Iseconds)\"}"
            fi

            local result_label="$PASS_COUNT/$TOTAL_CHECKS pass"
            (( FAIL_COUNT > 0 )) && result_label="$result_label, $FAIL_COUNT FAIL"
            (( SKIP_COUNT > 0 )) && result_label="$result_label, $SKIP_COUNT skip"
            domain_render "$result_label"
            ;;
        V|v)
            domain_render "Results (${#VALIDATION_RESULTS[@]} checks)"
            ;;
        *)
            log "unhandled key: $key"
            ;;
    esac
}

domain_on_click() {
    local cell="$1"
    case "$cell" in
        0) domain_on_key "R" ;;  # Run Validation
        1) domain_on_key "V" ;;  # View Results
        2) domain_on_key "Q" ;;  # Quit
    esac
}

domain_on_tick() {
    check_proprioception
}

# ── 6. Main Loop ─────────────────────────────────────────────────────

main() {
    discover_capabilities || { err "Required primals not found"; exit 1; }

    composition_startup "healthSpring NUCLEUS" "Science Validation Composition"

    subscribe_interactions "click"
    subscribe_sensor_stream

    domain_init

    while $RUNNING; do
        local sensor_batch
        sensor_batch=$(poll_sensor_stream)
        process_sensor_batch "$sensor_batch"

        ACCUMULATED_HOVER_MOVES=$((ACCUMULATED_HOVER_MOVES + SENSOR_HOVER_MOVES))

        if $SENSOR_HOVER_CHANGED; then
            : # no hover rendering needed
        fi

        if [[ -n "$SENSOR_KEY" ]]; then
            domain_on_key "$SENSOR_KEY"
        elif [[ "$SENSOR_CLICK_CELL" -ge 0 ]]; then
            domain_on_click "$SENSOR_CLICK_CELL"
        else
            domain_on_tick
            sleep "$POLL_INTERVAL"
        fi
    done

    if cap_available ledger && [[ -n "$SPINE_ID" ]]; then
        ledger_seal_spine
    fi

    composition_summary
    composition_teardown "${COMPOSITION_NAME}-main"
}

main
