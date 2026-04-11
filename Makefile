PREFIX ?= $(HOME)/.local
BIN_DIR ?= $(PREFIX)/bin
CONFIG_DIR ?= $(HOME)/.config/meld

.PHONY: build release test run diagnostics install install-config install-config-force update update-config uninstall clean

build:
	cargo build --release

release:
	cargo build --release

test:
	cargo test

run:
	cargo run --release --quiet --

diagnostics:
	cargo run --release --quiet -- --diagnostics

install:
	./scripts/install.sh --prefix "$(PREFIX)" --config-dir "$(CONFIG_DIR)"

install-config:
	./scripts/install-config.sh --config-dir "$(CONFIG_DIR)"

install-config-force:
	./scripts/install-config.sh --config-dir "$(CONFIG_DIR)" --force

update:
	./scripts/update.sh --prefix "$(PREFIX)" --config-dir "$(CONFIG_DIR)"

update-config:
	./scripts/update-config.sh --config-dir "$(CONFIG_DIR)"

uninstall:
	./scripts/uninstall.sh --prefix "$(PREFIX)" --config-dir "$(CONFIG_DIR)"

clean:
	cargo clean
