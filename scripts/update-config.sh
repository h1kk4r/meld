#!/usr/bin/env sh

# Regenerate init.lua from the current effective config.
#
# This keeps known user values, pulls in new documented sections, and creates
# a timestamped backup unless --no-backup is passed.

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/meld}"
CONFIG_FILE=""
BIN_PATH=""
BACKUP=1

usage() {
  cat <<EOF
Usage: scripts/update-config.sh [options]

Options:
  --config-dir PATH  Update PATH/init.lua
  --config PATH      Update an explicit init.lua path
  --bin PATH         Use a specific meld binary
  --no-backup        Replace the config without creating a backup
  --help             Show this help
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --config-dir)
      CONFIG_DIR=$2
      shift 2
      ;;
    --config)
      CONFIG_FILE=$2
      shift 2
      ;;
    --bin)
      BIN_PATH=$2
      shift 2
      ;;
    --no-backup)
      BACKUP=0
      shift
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [ -z "$CONFIG_FILE" ]; then
  CONFIG_FILE="$CONFIG_DIR/init.lua"
fi

if [ -z "$BIN_PATH" ]; then
  if [ -x "$REPO_ROOT/target/release/meld" ]; then
    BIN_PATH="$REPO_ROOT/target/release/meld"
  else
    BIN_PATH=$(command -v meld 2>/dev/null || true)
  fi
fi

if [ -z "$BIN_PATH" ] || [ ! -x "$BIN_PATH" ]; then
  echo "meld binary not found; pass it with --bin or build meld first" >&2
  exit 1
fi

mkdir -p "$(dirname "$CONFIG_FILE")"

TMP_FILE=$(mktemp "${TMPDIR:-/tmp}/meld-config.XXXXXX")
trap 'rm -f "$TMP_FILE"' EXIT INT TERM

if [ -f "$CONFIG_FILE" ]; then
  "$BIN_PATH" --config "$CONFIG_FILE" --print-config > "$TMP_FILE"
else
  "$BIN_PATH" --print-config > "$TMP_FILE"
fi

if [ -f "$CONFIG_FILE" ] && [ "$BACKUP" -eq 1 ]; then
  BACKUP_FILE="$CONFIG_FILE.bak.$(date +%Y%m%d%H%M%S)"
  cp "$CONFIG_FILE" "$BACKUP_FILE"
  echo "backup created: $BACKUP_FILE"
fi

mv "$TMP_FILE" "$CONFIG_FILE"
trap - EXIT INT TERM

echo "updated config: $CONFIG_FILE"
echo "check it with: $BIN_PATH --config \"$CONFIG_FILE\" --diagnostics"
