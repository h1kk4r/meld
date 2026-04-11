# Install Guide

This project is prepared for local release-style installation. The installer builds and installs the release binary only.

## Default Install

Run:

```bash
./scripts/install.sh
```

This does three things:

- builds `target/release/meld`
- installs the binary to `~/.local/bin/meld`
- installs `~/.config/meld/init.lua` only if it does not already exist

If `~/.local/bin` is not in your `PATH`, add it to your shell config:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

## Custom Paths

Install to a different prefix:

```bash
./scripts/install.sh --prefix /opt/meld
```

Install directly to a specific binary directory:

```bash
./scripts/install.sh --bin-dir /usr/local/bin
```

Install with a custom config directory:

```bash
./scripts/install.sh --config-dir "$HOME/.config/meld"
```

## Config Installation Rules

The installer never overwrites an existing config by default.

If `~/.config/meld/init.lua` already exists, the installer prints a message and leaves it untouched.

To intentionally replace the config:

```bash
./scripts/install-config.sh --force
```

Before replacement, the script creates a timestamped backup:

```text
init.lua.bak.YYYYMMDDHHMMSS
```

## Make Targets

Equivalent make targets:

```bash
make install
make install-config
make install-config-force
make uninstall
```

## Uninstall

Remove the binary:

```bash
./scripts/uninstall.sh
```

Remove the binary and config:

```bash
./scripts/uninstall.sh --remove-config
```

Remove the binary, config and cache:

```bash
./scripts/uninstall.sh --remove-config --remove-cache
```
