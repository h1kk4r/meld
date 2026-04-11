#!/usr/bin/env sh

# Remove local Meld installation artifacts.
#
# The default path only removes the binary. Config and cache are opt-in removals
# so an uninstall does not accidentally erase a tuned init.lua.

set -eu

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"
CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/meld}"
REMOVE_CONFIG=0
REMOVE_CACHE=0

usage() {
  cat <<EOF
Usage: scripts/uninstall.sh [options]

Options:
  --prefix PATH      Set the install prefix (default: \$HOME/.local)
  --bin-dir PATH     Remove meld from PATH
  --config-dir PATH  Remove init.lua from PATH/init.lua when asked
  --remove-config    Remove the installed init.lua
  --remove-cache     Remove ~/.cache/meld
  --help             Show this help
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --prefix)
      PREFIX=$2
      BIN_DIR="$PREFIX/bin"
      shift 2
      ;;
    --bin-dir)
      BIN_DIR=$2
      shift 2
      ;;
    --config-dir)
      CONFIG_DIR=$2
      shift 2
      ;;
    --remove-config)
      REMOVE_CONFIG=1
      shift
      ;;
    --remove-cache)
      REMOVE_CACHE=1
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

BINARY_PATH="$BIN_DIR/meld"
CONFIG_PATH="$CONFIG_DIR/init.lua"
CACHE_DIR="$HOME/.cache/meld"

if [ -f "$BINARY_PATH" ]; then
  rm -f "$BINARY_PATH"
  echo "removed binary: $BINARY_PATH"
else
  echo "binary not found: $BINARY_PATH"
fi

if [ "$REMOVE_CONFIG" -eq 1 ]; then
  if [ -f "$CONFIG_PATH" ]; then
    rm -f "$CONFIG_PATH"
    echo "removed config: $CONFIG_PATH"
  else
    echo "config not found: $CONFIG_PATH"
  fi
fi

if [ "$REMOVE_CACHE" -eq 1 ]; then
  if [ -d "$CACHE_DIR" ]; then
    rm -rf "$CACHE_DIR"
    echo "removed cache: $CACHE_DIR"
  else
    echo "cache not found: $CACHE_DIR"
  fi
fi
