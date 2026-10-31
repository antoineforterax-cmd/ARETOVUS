#!/bin/sh
# AURORA testnet local-LAN assistant for Linux; Python >=3.10 required.
set -eu
CDPATH= cd -- "$(dirname -- "$0")"
if ! command -v python3 >/dev/null 2>&1; then
  echo "Python 3.10+ required. The node itself does not require Python." >&2
  exit 1
fi
exec python3 scripts/lan_assistant_unix.py "$@"
