# Meld

`meld` is a Lua-configurable terminal fetch/status tool for macOS-first workflows.

It collects system, Git, terminal, battery, display, package and Spotify data, then renders it as a configurable terminal status block with optional ASCII art or WezTerm inline images.

## Features

- Lua configuration through `~/.config/meld/init.lua`
- Configurable line order, labels, separators, text case and colors
- Optional text or shell command output before/after the rendered block
- macOS ASCII logo preset with custom text-logo support
- WezTerm image backend with ASCII fallback and auto-height mode
- Spotify line support and optional current-track cover art
- ANSI color blocks
- Diagnostics for config path, requested fields and visual backend selection
- Release-only install and update scripts
- Config update flow that keeps supported user values while refreshing generated documentation

## Quick Start

Install the release binary and bootstrap a config:

```bash
./scripts/install.sh
```

Make sure `~/.local/bin` is in your `PATH`, then run:

```bash
meld
meld --diagnostics
```

The default config path is:

```text
~/.config/meld/init.lua
```

## Commands

Common project commands:

```bash
make build
make test
make install
make update
make update-with-config
make update-config
make diagnostics
make uninstall
```

Common binary commands:

```bash
meld
meld --config /path/to/init.lua
meld --show-config-path
meld --diagnostics
meld --print-config
```

## Install

`make install` and `./scripts/install.sh` both build a release binary and install it to:

```text
~/.local/bin/meld
```

They also bootstrap `~/.config/meld/init.lua` from [examples/init.lua](examples/init.lua) when no config exists yet.

Existing configs are preserved by default. To intentionally replace an existing config with a fresh template:

```bash
./scripts/install-config.sh --force
make install-config-force
```

A timestamped backup is created before replacement.

## Update

`make update` and `./scripts/update.sh` rebuild the release binary and reinstall it.

They do not rewrite `~/.config/meld/init.lua` by default.

To update the binary and regenerate the config in one explicit step:

```bash
./scripts/update.sh --update-config
make update-with-config
```

To update only the config:

```bash
make update-config
```

The config update is not a blind copy. `scripts/update-config.sh` asks the current `meld` binary to render a fresh documented config through `meld --print-config`, using values from your existing `init.lua`.

This means:

- known user values are preserved
- new documented sections are added
- new comments and examples are refreshed
- a backup is created before the file is rewritten

Important limitation: custom hand-written comments and arbitrary Lua helper code are not preserved as text. The output file is regenerated from the parsed effective config. If you keep large custom Lua helpers, store them in a separate file and load them from `init.lua`.

## Configuration

The config is a Lua script. The usual shape is:

```lua
local config = meld

config.layout = {
  align = true,
  separator = ":",
}

config.order = {
  "directory",
  "git",
  "spotify",
  "os",
  "kernel",
  "shell",
  "terminal",
}
```

Only fields listed in `config.order` are shown. Comment a field out to hide it.

See [docs/CONFIG.md](docs/CONFIG.md) for the full configuration guide.

## Coloring

For regular info lines:

```lua
config.colors = {
  label = "blue",
  separator = "bright_black",
  value = "white",
}
```

For individual symbols in custom ASCII art:

```lua
local esc = string.char(27)

config.logo.text = table.concat({
  esc .. "[34m██" .. esc .. "[0m",
  esc .. "[36m██" .. esc .. "[0m",
}, "\n")
```

ANSI escape sequences are ignored when calculating visible width, so alignment is preserved.

## Visual Backend Priority

Visual backends are selected in this order:

1. `spotify-cover`
2. `image`
3. `logo-text`
4. `logo-preset`
5. `none`

If a higher-priority backend is enabled but unavailable, `meld` falls back to the next backend.

## Documentation

- [docs/INSTALL.md](docs/INSTALL.md) covers installation and removal.
- [docs/CONFIG.md](docs/CONFIG.md) covers `init.lua`.
- [docs/OPERATIONS.md](docs/OPERATIONS.md) covers update, diagnostics and production checks.

## Release Checklist

Before tagging or distributing:

```bash
cargo fmt --check
cargo test
cargo build --release
./target/release/meld --config examples/init.lua --diagnostics
tmpdir=$(mktemp -d)
./scripts/install.sh --no-build --bin-dir "$tmpdir/bin" --config-dir "$tmpdir/config"
```

For WezTerm image changes, also verify rendering in a real WezTerm session.

## Uninstall

Remove only the binary:

```bash
./scripts/uninstall.sh
```

Remove the binary, config and cache:

```bash
./scripts/uninstall.sh --remove-config --remove-cache
```
