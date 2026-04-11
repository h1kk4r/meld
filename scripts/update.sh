#!/usr/bin/env sh

# Production updater for local builds.
#
# By default this script only updates the release binary. Config regeneration is
# opt-in because even merge-like generated updates can disturb hand-edited Lua.

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"
CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/meld}"
BUILD=1
UPDATE_CONFIG=0
NO_BACKUP=0

usage() {
  cat <<EOF
Usage: scripts/update.sh [options]

Options:
  --no-build         Reuse an existing build artifact
  --prefix PATH      Set the install prefix (default: \$HOME/.local)
  --bin-dir PATH     Install the binary directly into PATH
  --config-dir PATH  Config directory used with --update-config
  --update-config    Also regenerate PATH/init.lua after updating the binary
  --skip-config      Deprecated no-op; config updates are opt-in now
  --no-backup        Rewrite init.lua without backup when --update-config is used
  --help             Show this help
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --no-build)
      BUILD=0
      shift
      ;;
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
    --update-config)
      UPDATE_CONFIG=1
      shift
      ;;
    --skip-config)
      UPDATE_CONFIG=0
      shift
      ;;
    --no-backup)
      NO_BACKUP=1
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

if [ "$BUILD" -eq 1 ] && ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required to build meld" >&2
  exit 1
fi

cd "$REPO_ROOT"

if [ "$BUILD" -eq 1 ]; then
  cargo build --release
fi

BINARY_PATH="$REPO_ROOT/target/release/meld"

if [ ! -x "$BINARY_PATH" ]; then
  echo "binary not found: $BINARY_PATH" >&2
  echo "build meld first or omit --no-build" >&2
  exit 1
fi

mkdir -p "$BIN_DIR"
install -m 755 "$BINARY_PATH" "$BIN_DIR/meld"
echo "updated binary: $BIN_DIR/meld"

if [ "$UPDATE_CONFIG" -eq 1 ]; then
  if [ "$NO_BACKUP" -eq 1 ]; then
    "$SCRIPT_DIR/update-config.sh" --config-dir "$CONFIG_DIR" --bin "$BINARY_PATH" --no-backup
  else
    "$SCRIPT_DIR/update-config.sh" --config-dir "$CONFIG_DIR" --bin "$BINARY_PATH"
  fi
else
  echo "config left unchanged: $CONFIG_DIR/init.lua"
  echo "update config explicitly with: $0 --update-config"
fi

echo "verify with: $BIN_DIR/meld --diagnostics"
