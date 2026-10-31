#!/bin/sh
# AURORA testnet LAN assistant for macOS. Requires Python >=3.10.
set -eu
CDPATH= cd -- "$(dirname -- "$0")"
if ! command -v python3 >/dev/null 2>&1; then
  echo "Install Python 3.10+ from a trusted source. No package is downloaded automatically." >&2
  exit 1
fi
exec python3 scripts/lan_assistant_unix.py "$@"
