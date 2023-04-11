#!/bin/bash
ACTION="$1"
INDEX="$2"
PORT_BASE=7999

set -e

function help() {
    echo "Usage: $0 start|stop index|all"
    exit 1
}

echo "Building..."
cargo build &>/dev/null

if [ "$INDEX" == "all" ]; then
  "$0" "$ACTION" 1
  "$0" "$ACTION" 2
  "$0" "$ACTION" 3
  exit 0
fi

PORT="$(("$PORT_BASE" + "$INDEX"))"
if [ "$PORT" -eq "$PORT_BASE" ]; then
  # Index is NaN or empty
  help
fi

set -uo pipefail
PID_FILE="/var/run/user/$(id -u)/timestamping-$INDEX.pid"

cd "$(dirname "$(realpath "$0")")"

case "$ACTION" in
  start)
    if [ -e "$PID_FILE" ]; then
      echo "Already running"
      exit 0
    fi
    echo "Starting server with index $INDEX on http://localhost:$PORT"
    cargo run -- "$INDEX" "$PORT" &
    PID="$!"
    echo "$PID" > "$PID_FILE"
    ;;
  stop)
    if [ ! -e "$PID_FILE" ]; then
      echo "Not running"
        exit 0
    fi
    kill "$(cat "$PID_FILE")"
    rm "$PID_FILE"
    ;;
  *)
    help
    ;;
esac
