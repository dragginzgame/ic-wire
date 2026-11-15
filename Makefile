.DEFAULT_GOAL := help

override TOKO_TOOLING_ROOT := $(CURDIR)/vendor/toko-tooling
include vendor/toko-tooling/make/consumer.mk
include vendor/toko-tooling/make/cargo-dependencies.mk

.PHONY: help snapshot-check test lint doc check-wasm
help:
	@printf '%s\n' \
	  'install-tools         Install the reviewed common host and Rust tools' \
	  'tools-check           Check installed tools offline' \
	  'snapshot-check        Verify installed snapshot bytes offline' \
	  'deps-list             List workspace package versions offline' \
	  'deps-update           Run cargo update -v for maintained workspaces' \
	  '  Optional: DEPENDENCY_PACKAGE=name DEPENDENCY_PRECISE=version' \
	  'test                  Run native contract tests' \
	  'lint                  Run Clippy with warnings denied' \
	  'doc                   Check Rustdoc with warnings denied' \
	  'check-wasm            Check the portable library on wasm32'

snapshot-check:
	@bash "$(TOKO_TOOLING_ROOT)/scripts/ci/verify-toko-tooling-snapshot.sh" --consumer "$(CURDIR)"

test lint doc check-wasm: toko-tooling-check | toko-execution-check

test:
	+cargo test --workspace --locked --offline

lint:
	+cargo clippy --workspace --all-targets --locked --offline -- -D warnings

doc:
	+RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked --offline

check-wasm:
	+RUSTFLAGS="-D warnings" cargo check --workspace --lib --target wasm32-unknown-unknown --locked --offline
