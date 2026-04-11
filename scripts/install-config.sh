#!/usr/bin/env sh

# Install the documented starter init.lua.
#
# Existing configs are preserved by default; use --force when you explicitly
# want a fresh template and a timestamped backup of the old file.

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/meld}"
SOURCE_FILE="$REPO_ROOT/examples/init.lua"
FORCE=0

usage() {
  cat <<EOF
Usage: scripts/install-config.sh [options]

Options:
  --config-dir PATH  Install init.lua into PATH/init.lua
  --source PATH      Copy a different init.lua source file
  --force            Overwrite an existing init.lua (creates a backup first)
  --help             Show this help
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --config-dir)
      CONFIG_DIR=$2
      shift 2
      ;;
    --source)
      SOURCE_FILE=$2
      shift 2
      ;;
    --force)
      FORCE=1
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

TARGET_FILE="$CONFIG_DIR/init.lua"

if [ ! -f "$SOURCE_FILE" ]; then
  echo "source config not found: $SOURCE_FILE" >&2
  exit 1
fi

mkdir -p "$CONFIG_DIR"

if [ -f "$TARGET_FILE" ] && [ "$FORCE" -ne 1 ]; then
  echo "config already exists: $TARGET_FILE"
  echo "use --force to replace it"
  exit 0
fi

if [ -f "$TARGET_FILE" ] && [ "$FORCE" -eq 1 ]; then
  BACKUP_FILE="$TARGET_FILE.bak.$(date +%Y%m%d%H%M%S)"
  cp "$TARGET_FILE" "$BACKUP_FILE"
  echo "backup created: $BACKUP_FILE"
fi

cp "$SOURCE_FILE" "$TARGET_FILE"
echo "installed config: $TARGET_FILE"
echo "check it with: meld --show-config-path"
