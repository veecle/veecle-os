#!/usr/bin/env bash

set -meuo pipefail

for tool in jq uuidgen mktemp cargo
do
  if ! command -v $tool >/dev/null 2>&1
  then
    printf '\e[31mERROR\e[0m: `%s` is required on the PATH\n' $tool
    missing_tool=true
  fi
done
if [ -v missing_tool ]
then
  exit 1
fi

if [ $# -eq 0 ]
then
  echo >&2 "missing subset to run ('ping-pong' or 'useless')"
  exit 1
fi

subset="$1"
case "$subset"
in
  ping-pong | useless)
    true
  ;;
  * )
    echo >&2 "unknown subset '$subset', use 'ping-pong' or 'useless'"
    exit 1
  ;;
esac

# Cleanly kill background processes when exiting for any reason.
typeset -a pids
shutdown() {
  if [ -v pids ]
  then
    # Kill processes in reverse order, waiting for each to quit.
    for ((i=${#pids[@]}-1; i>=0; i--)); do
      if kill -0 "${pids[i]}" 2>/dev/null; then
        run kill "${pids[i]}"
        wait "${pids[i]}" 2>/dev/null || true
      fi
    done
    pids=()
  fi
}
trap shutdown EXIT

# Run from the root of the repository, this is in `veecle-os-examples/orchestrator-ipc/` so two directories up.
cd "$(dirname "$(dirname "$(realpath "$(dirname "${BASH_SOURCE[0]}")")")")"

# Helper functions.
log-command() {
  local command="$1"
  shift

  printf '     \e[36;1mRunning\e[0m `%q' "$command" >&2
  printf ' %q' "$@" >&2
  printf '`\n' >&2
}

run() {
  log-command "$@"
  "$@"
}

run-background() {
  log-command "$@"
  "$@" &
  pids+=($!)
}

get-bin-path() {
  jq -r --arg bin "$1" 'select(.reason == "compiler-artifact") | select (.target.name == $bin) | .executable'
}

build() {
  run cargo build --message-format=json --bin "$1" | get-bin-path "$1"
}

# Ensure any user environment won't affect the spawned processes.
unset VEECLE_ORCHESTRATOR_SOCKET

# Make some known ids/paths/addresses to use.

# Use a non-standard localhost IP to avoid conflicts with other local services.
EXAMPLE_IP=127.0.0.26

CONTROL1=$EXAMPLE_IP:7607
CONTROL2="$(mktemp -u -p "$XDG_RUNTIME_DIR" veecle-orchestrator.example.XXXXXX.socket)"

IPC1=$EXAMPLE_IP:2661
IPC2=$EXAMPLE_IP:2662
TELEMETRY_SOCKET=$EXAMPLE_IP:8329
UI_WEBSOCKET_PORT=42817

PING_ID=$(uuidgen -r)
PONG_ID=$(uuidgen -r)
TRACE_ID=$(uuidgen -r)

USELESS_MACHINE1_ID=$(uuidgen -r)
USELESS_MACHINE2_ID=$(uuidgen -r)

# Build the binaries that will be used
case "$subset"
in
  ping-pong)
    PING="$(cd veecle-os-examples/orchestrator-ipc && build ping)"
    PONG="$(cd veecle-os-examples/orchestrator-ipc && build pong)"
    TRACE="$(cd veecle-os-examples/orchestrator-ipc && build trace)"
  ;;

  useless)
    USELESS_MACHINE="$(cd veecle-os-examples/orchestrator-ipc && build useless-machine)"
  ;;
esac

ORCHESTRATOR="$(build veecle-orchestrator)"
CLI="$(build veecle-orchestrator-cli)"
UI_SERVER="$(build veecle-telemetry-server)"
UI_APP="$(build veecle-telemetry-ui)"

echo >&2
echo >&2 'Starting veecle-telemetry-server and orchestrators'
echo >&2

export VEECLE_ORCHESTRATOR_LOG=debug

run-background "$UI_SERVER" --bind "$EXAMPLE_IP" --port "$UI_WEBSOCKET_PORT" --telemetry-socket "$TELEMETRY_SOCKET"
run-background "$ORCHESTRATOR" --control-socket "$CONTROL1" --ipc-socket $IPC1 --telemetry-socket "$TELEMETRY_SOCKET"
run-background "$ORCHESTRATOR" --control-socket "$CONTROL2" --ipc-socket $IPC2 --telemetry-socket "$TELEMETRY_SOCKET"

sleep 1

echo >&2
echo >&2 'Configuring runtimes on orchestrators'
echo >&2

case "$subset"
in
  ping-pong)
    run "$CLI" >&2 --socket "$CONTROL1" runtime add "$PING" --id $PING_ID
    run "$CLI" >&2 --socket "$CONTROL2" runtime add "$PONG" --id $PONG_ID --copy
    run "$CLI" >&2 --socket "$CONTROL1" runtime add "$TRACE" --id $TRACE_ID

    mod=veecle_os_examples_common::actors::ping_pong

    run "$CLI" >&2 --socket "$CONTROL1" link add --type $mod::Ping --to $TRACE_ID
    run "$CLI" >&2 --socket "$CONTROL1" link add --type $mod::Ping --to $IPC2
    run "$CLI" >&2 --socket "$CONTROL2" link add --type $mod::Ping --to $PONG_ID
    run "$CLI" >&2 --socket "$CONTROL1" link add --type $mod::Pong --to $TRACE_ID
    run "$CLI" >&2 --socket "$CONTROL2" link add --type $mod::Pong --to $IPC1
    run "$CLI" >&2 --socket "$CONTROL1" link add --type $mod::Pong --to $PING_ID

  ;;

  useless)
    # Because it's registered without privileges on orchestrator1 it will fail to shut itself down.
    run "$CLI" >&2 --socket "$CONTROL1" runtime add "$USELESS_MACHINE" --id $USELESS_MACHINE1_ID
    run "$CLI" >&2 --socket "$CONTROL2" runtime add "$USELESS_MACHINE" --id $USELESS_MACHINE2_ID --privileged

    run "$CLI" >&2 --socket "$CONTROL2" runtime list
    run "$CLI" >&2 --socket "$CONTROL2" link list

    run "$CLI" >&2 --socket "$CONTROL2" clear

    run "$CLI" >&2 --socket "$CONTROL2" runtime list
    run "$CLI" >&2 --socket "$CONTROL2" link list

    run "$CLI" >&2 --socket "$CONTROL2" runtime add "$USELESS_MACHINE" --id $USELESS_MACHINE2_ID --privileged
  ;;
esac

run "$CLI" >&2 --socket "$CONTROL2" runtime list
run "$CLI" >&2 --socket "$CONTROL2" link list

echo >&2
echo >&2 'Configuration done, starting runtimes'
echo >&2

sleep 1

case "$subset"
in
  ping-pong)
    run "$CLI" >&2 --socket "$CONTROL1" runtime start $TRACE_ID
    run "$CLI" >&2 --socket "$CONTROL2" runtime start $PONG_ID
    run "$CLI" >&2 --socket "$CONTROL1" runtime start $PING_ID
  ;;

  useless)
    run "$CLI" >&2 --socket "$CONTROL1" runtime start $USELESS_MACHINE1_ID
    run "$CLI" >&2 --socket "$CONTROL2" runtime start $USELESS_MACHINE2_ID
  ;;
esac

echo >&2
echo >&2 'Starting veecle-telemetry-ui (will run until closed)'
echo >&2

run "$UI_APP" "ws://$EXAMPLE_IP:$UI_WEBSOCKET_PORT"

echo >&2
echo >&2 'veecle-telemetry-ui closed, stopping background processes'
echo >&2

shutdown

echo >&2
echo >&2 'Run successful, there should have been much log spam in the ui'
