#!/usr/bin/env sh

# Production installer for local builds.
#
# The script intentionally installs only the release binary. Keeping one
# production path avoids confusing users with stale local build artifacts in PATH.

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$PREFIX/bin}"
CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/meld}"
BUILD=1
INSTALL_CONFIG=1
FORCE_CONFIG=0

usage() {
  cat <<EOF
Usage: scripts/install.sh [options]

Options:
  --no-build         Reuse an existing build artifact
  --prefix PATH      Set the install prefix (default: \$HOME/.local)
  --bin-dir PATH     Install the binary directly into PATH
  --config-dir PATH  Install init.lua into PATH/init.lua
  --skip-config      Do not install examples/init.lua
  --force-config     Overwrite an existing init.lua
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
    --skip-config)
      INSTALL_CONFIG=0
      shift
      ;;
    --force-config)
      FORCE_CONFIG=1
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

echo "installed binary: $BIN_DIR/meld"

if [ "$INSTALL_CONFIG" -eq 1 ]; then
  if [ "$FORCE_CONFIG" -eq 1 ]; then
    "$SCRIPT_DIR/install-config.sh" --config-dir "$CONFIG_DIR" --force
  else
    "$SCRIPT_DIR/install-config.sh" --config-dir "$CONFIG_DIR"
  fi
fi

case ":$PATH:" in
  *":$BIN_DIR:"*)
    ;;
  *)
    echo "add this to your PATH if needed: $BIN_DIR"
    ;;
esac

echo "verify with: $BIN_DIR/meld --diagnostics"
