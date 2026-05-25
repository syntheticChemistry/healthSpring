#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# gate_nucleus.sh — Deploy healthSpring's dual-tower NUCLEUS on assigned gate
#
# Launches the healthspring_enclave_proto_nucleate composition:
#   Tower A (Patient Data Enclave): beardog, songbird, nestgate, rhizocrypt,
#     loamspine, sweetgrass, biomeos
#   Tower B (Analytics/Inference): beardog, songbird, squirrel, nestgate,
#     rhizocrypt, sweetgrass
#   Ionic Bridge: primalspring_primal (bonding.propose / bonding.accept)
#
# Two FAMILY_IDs enforce the dual-tower trust boundary. Cross-family traffic
# goes through the ionic bridge only (storage.* denied across towers).
#
# Gate: ironGate (i9-14900K, RTX 5070, 96GB DDR5) — primary
# Co-tenants: primalSpring, ludoSpring, groundSpring
#
# Usage:
#   ./tools/gate_nucleus.sh start
#   ./tools/gate_nucleus.sh status
#   ./tools/gate_nucleus.sh stop
#   ./tools/gate_nucleus.sh validate
#
# Prerequisites:
#   - plasmidBin primals available (run fetch_primals.sh --check first)
#   - socat installed

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ECO_ROOT="$(cd "$PROJECT_ROOT/../.." && pwd)"

FAMILY_A="${FAMILY_A:-healthspring-tower-a}"
FAMILY_B="${FAMILY_B:-healthspring-tower-b}"
GATE_NAME="irongate"

PLASMID_BIN="${ECOPRIMALS_PLASMID_BIN:-$ECO_ROOT/infra/plasmidBin}"
BIN_DIR="$PLASMID_BIN/primals"

RUNTIME_DIR="/tmp/biomeos"
SOCKET_DIR_A="$RUNTIME_DIR/$FAMILY_A"
SOCKET_DIR_B="$RUNTIME_DIR/$FAMILY_B"
PID_DIR="/tmp/nucleus-${GATE_NAME}-pids"
LOG_DIR="/tmp/nucleus-${GATE_NAME}-logs"

SEED_A="${FAMILY_SEED_A:-$(head -c 32 /dev/urandom | xxd -p | tr -d '\n')}"
SEED_B="${FAMILY_SEED_B:-$(head -c 32 /dev/urandom | xxd -p | tr -d '\n')}"

HEALTH_TIMEOUT=20

log() { printf "[%s] %s %s\n" "$GATE_NAME" "$(date +%H:%M:%S)" "$*"; }
err() { printf "[%s] ERROR: %s\n" "$GATE_NAME" "$*" >&2; }
ok()  { printf "[%s] ✓ %s\n" "$GATE_NAME" "$*"; }

find_binary() {
    local name="$1"
    local triple_dir="$PLASMID_BIN/primals/x86_64-unknown-linux-musl"
    if [[ -x "$triple_dir/$name" ]]; then
        echo "$triple_dir/$name"
        return
    fi
    if [[ -x "$BIN_DIR/$name" ]]; then
        echo "$BIN_DIR/$name"
        return
    fi
    echo ""
}

save_pid() {
    mkdir -p "$PID_DIR"
    echo "$2" > "$PID_DIR/$1.pid"
}

check_health() {
    local sock="$1" timeout="${2:-$HEALTH_TIMEOUT}" elapsed=0
    local xdg_sock="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/biomeos/$(basename "$sock")"
    while [[ $elapsed -lt $timeout ]]; do
        for path in "$sock" "$xdg_sock"; do
            if [[ -S "$path" ]]; then
                local resp
                resp=$(echo '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":1}' | \
                    timeout 3 socat - "UNIX-CONNECT:$path" 2>/dev/null || true)
                if echo "$resp" | grep -qE '"(healthy|alive)"'; then
                    return 0
                fi
            fi
        done
        sleep 0.5
        elapsed=$((elapsed + 1))
    done
    return 1
}

start_primal() {
    local name="$1" family="$2" socket_dir="$3" seed="$4"
    shift 4
    local extra_env=("$@")

    mkdir -p "$socket_dir" "$LOG_DIR"
    local log_file="$LOG_DIR/${name}-${family}.log"
    local sock_path="$socket_dir/${name}-${family}.sock"

    log "Starting $name (family=$family)..."

    local start_script="$PLASMID_BIN/start_primal.sh"
    if [[ -x "$start_script" ]]; then
        local output
        output=$(env FAMILY_ID="$family" \
            FAMILY_SEED="$seed" \
            BEARDOG_FAMILY_SEED="$seed" \
            NODE_ID="$GATE_NAME" \
            "${extra_env[@]}" \
            "$start_script" "$name" \
                --socket "$sock_path" \
                --family-id "$family" \
                --log-file "$log_file" 2>&1) || true

        local pid
        pid=$(echo "$output" | grep -oP 'pid:\s+\K\d+' || echo "")
        [[ -n "$pid" ]] && save_pid "${name}-${family}" "$pid"
    else
        local bin
        bin=$(find_binary "$name")
        if [[ -z "$bin" ]]; then
            err "Binary not found: $name"
            return 1
        fi
        env FAMILY_ID="$family" \
            FAMILY_SEED="$seed" \
            BEARDOG_FAMILY_SEED="$seed" \
            NODE_ID="$GATE_NAME" \
            SOCKET_DIR="$socket_dir" \
            SOCKET_PATH="$sock_path" \
            "${extra_env[@]}" \
            "$bin" server --socket "$sock_path" --family-id "$family" \
            > "$log_file" 2>&1 &
        local pid=$!
        save_pid "${name}-${family}" "$pid"
    fi

    if check_health "$sock_path"; then
        ok "$name ($family) healthy"
        return 0
    else
        err "$name ($family) failed health check within ${HEALTH_TIMEOUT}s"
        return 1
    fi
}

do_start() {
    log "═══ Dual-Tower NUCLEUS Start ═══"
    log "Tower A family: $FAMILY_A (Patient Data Enclave)"
    log "Tower B family: $FAMILY_B (Analytics/Inference)"
    log "Seed A: ${SEED_A:0:16}..."
    log "Seed B: ${SEED_B:0:16}..."
    echo ""

    mkdir -p "$SOCKET_DIR_A" "$SOCKET_DIR_B" "$PID_DIR" "$LOG_DIR"
    echo "$SEED_A" > "$SOCKET_DIR_A/.family.seed"
    echo "$SEED_B" > "$SOCKET_DIR_B/.family.seed"

    local failed=0

    log "── Tower A (Patient Data Enclave) ──"
    start_primal beardog "$FAMILY_A" "$SOCKET_DIR_A" "$SEED_A" || ((failed++))
    start_primal songbird "$FAMILY_A" "$SOCKET_DIR_A" "$SEED_A" \
        "BEARDOG_SOCKET=$SOCKET_DIR_A/beardog-${FAMILY_A}.sock" || ((failed++))
    local SONGBIRD_A="$SOCKET_DIR_A/songbird-${FAMILY_A}.sock"
    start_primal nestgate "$FAMILY_A" "$SOCKET_DIR_A" "$SEED_A" \
        "DISCOVERY_ENDPOINT=$SONGBIRD_A" || ((failed++))
    start_primal rhizocrypt "$FAMILY_A" "$SOCKET_DIR_A" "$SEED_A" \
        "DISCOVERY_ENDPOINT=$SONGBIRD_A" || ((failed++))
    start_primal loamspine "$FAMILY_A" "$SOCKET_DIR_A" "$SEED_A" \
        "DISCOVERY_ENDPOINT=$SONGBIRD_A" \
        "BEARDOG_SOCKET=$SOCKET_DIR_A/beardog-${FAMILY_A}.sock" || ((failed++))
    start_primal sweetgrass "$FAMILY_A" "$SOCKET_DIR_A" "$SEED_A" \
        "DISCOVERY_ENDPOINT=$SONGBIRD_A" || ((failed++))
    start_primal biomeos "$FAMILY_A" "$SOCKET_DIR_A" "$SEED_A" \
        "DISCOVERY_ENDPOINT=$SONGBIRD_A" || ((failed++))

    echo ""
    log "── Tower B (Analytics/Inference) ──"
    start_primal beardog "$FAMILY_B" "$SOCKET_DIR_B" "$SEED_B" || ((failed++))
    start_primal songbird "$FAMILY_B" "$SOCKET_DIR_B" "$SEED_B" \
        "BEARDOG_SOCKET=$SOCKET_DIR_B/beardog-${FAMILY_B}.sock" || ((failed++))
    local SONGBIRD_B="$SOCKET_DIR_B/songbird-${FAMILY_B}.sock"
    start_primal squirrel "$FAMILY_B" "$SOCKET_DIR_B" "$SEED_B" \
        "DISCOVERY_ENDPOINT=$SONGBIRD_B" || ((failed++))
    start_primal nestgate "$FAMILY_B" "$SOCKET_DIR_B" "$SEED_B" \
        "DISCOVERY_ENDPOINT=$SONGBIRD_B" || ((failed++))
    start_primal rhizocrypt "$FAMILY_B" "$SOCKET_DIR_B" "$SEED_B" \
        "DISCOVERY_ENDPOINT=$SONGBIRD_B" || ((failed++))
    start_primal sweetgrass "$FAMILY_B" "$SOCKET_DIR_B" "$SEED_B" \
        "DISCOVERY_ENDPOINT=$SONGBIRD_B" || ((failed++))

    echo ""
    log "── Ionic Bridge (primalSpring coordination) ──"
    start_primal primalspring_primal "$FAMILY_A" "$SOCKET_DIR_A" "$SEED_A" \
        "FAMILY_B=$FAMILY_B" "SOCKET_DIR_B=$SOCKET_DIR_B" || ((failed++))

    echo ""
    if [[ $failed -eq 0 ]]; then
        ok "All 14 primal instances started. Dual-tower NUCLEUS live."
    else
        err "$failed primal(s) failed to start."
    fi

    log "Socket dirs:"
    log "  Tower A: $SOCKET_DIR_A"
    log "  Tower B: $SOCKET_DIR_B"
    log "PID dir: $PID_DIR"
    log "Logs: $LOG_DIR"

    return $failed
}

do_status() {
    log "═══ Gate Status ═══"
    local total=0 healthy=0

    for pid_file in "$PID_DIR"/*.pid; do
        [[ -f "$pid_file" ]] || continue
        local name
        name=$(basename "$pid_file" .pid)
        local pid
        pid=$(cat "$pid_file")
        total=$((total + 1))

        if kill -0 "$pid" 2>/dev/null; then
            ok "$name (pid $pid) — running"
            healthy=$((healthy + 1))
        else
            err "$name (pid $pid) — DEAD"
        fi
    done

    if [[ $total -eq 0 ]]; then
        log "No NUCLEUS running on this gate."
    else
        log "$healthy/$total primals healthy."
    fi
}

do_stop() {
    log "═══ Gate Stop ═══"

    for pid_file in "$PID_DIR"/*.pid; do
        [[ -f "$pid_file" ]] || continue
        local name
        name=$(basename "$pid_file" .pid)
        local pid
        pid=$(cat "$pid_file")

        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null && ok "Stopped $name (pid $pid)"
        fi
        rm -f "$pid_file"
    done

    rm -rf "$SOCKET_DIR_A" "$SOCKET_DIR_B"
    log "All primals stopped. Sockets cleaned."
}

do_validate() {
    log "═══ Gate Validation ═══"
    log "Running healthspring validate against live dual-tower NUCLEUS..."
    echo ""

    export BIOMEOS_SOCKET_DIR="$SOCKET_DIR_A"
    export PRIMALSPRING_SOCKET="$SOCKET_DIR_A/primalspring_primal-${FAMILY_A}.sock"
    export BEARDOG_SOCKET="$SOCKET_DIR_A/beardog-${FAMILY_A}.sock"
    export NESTGATE_SOCKET="$SOCKET_DIR_A/nestgate-${FAMILY_A}.sock"
    export SQUIRREL_SOCKET="$SOCKET_DIR_B/squirrel-${FAMILY_B}.sock"

    local unibin="$PROJECT_ROOT/target/release/healthspring_unibin"
    if [[ ! -x "$unibin" ]]; then
        err "healthspring_unibin not found. Build: cargo build --release --bin healthspring_unibin"
        return 1
    fi

    "$unibin" validate --format json 2>&1 | tee "$LOG_DIR/validation-$(date +%Y%m%d_%H%M%S).json"
    local rc=${PIPESTATUS[0]}

    echo ""
    if [[ $rc -eq 0 ]]; then
        ok "Validation PASSED against live NUCLEUS."
    else
        err "Validation FAILED (exit $rc). Check logs."
    fi
    return $rc
}

case "${1:-help}" in
    start)    do_start ;;
    stop)     do_stop ;;
    status)   do_status ;;
    validate) do_validate ;;
    *)        echo "Usage: $0 {start|stop|status|validate}"; exit 1 ;;
esac
