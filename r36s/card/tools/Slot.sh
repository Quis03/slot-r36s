#!/bin/bash
# ArkOS > Options > Tools > Slot: try slot without changing anything.
# Power off or Restart inside slot brings you back here.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
[ "$(id -u)" -eq 0 ] || exec sudo -n bash "$0" "$@"
exec sh "$ROOT/slot/System/r36s-launch.sh" test
