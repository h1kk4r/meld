# Operations Guide

This document covers update and production checks.

## Update

Update binary and config:

```bash
make update
```

Equivalent script:

```bash
./scripts/update.sh
```

Update only the config:

```bash
make update-config
```

Equivalent script:

```bash
./scripts/update-config.sh
```

## Config Update Semantics

`update-config` is merge-like, but it is not a text merge.

It does this:

- loads your current `init.lua`
- parses supported Meld options
- renders a fresh documented `init.lua`
- writes your current known values into that fresh file
- creates a timestamped backup before replacing the file

It preserves:

- supported config values
- line order
- labels
- colors
- view modes
- Spotify format
- logo/image/block settings

It does not preserve arbitrary text:

- hand-written comments
- helper functions not represented by supported config fields
- custom Lua code that only exists for side effects

If you need large custom helpers, keep them in a separate file and load them from `init.lua`.

## Diagnostics

Show the config path that would be used:

```bash
meld --show-config-path
```

Show config and rendering diagnostics:

```bash
meld --diagnostics
```

Diagnostics include:

- config source
- config candidates
- requested lines
- requested system fields
- visual backend priority
- image path/status
- Spotify status

## Production Checks

Before distributing:

```bash
cargo fmt --check
cargo test
cargo build --release
./target/release/meld --config examples/init.lua --diagnostics
```

To verify the installer without rebuilding:

```bash
tmpdir=$(mktemp -d)
./scripts/install.sh --no-build --bin-dir "$tmpdir/bin" --config-dir "$tmpdir/config"
```

To verify an update flow:

```bash
tmpdir=$(mktemp -d)
./scripts/install.sh --no-build --bin-dir "$tmpdir/bin" --config-dir "$tmpdir/config"
./scripts/update.sh --no-build --bin-dir "$tmpdir/bin" --config-dir "$tmpdir/config"
```
