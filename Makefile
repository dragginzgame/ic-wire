.DEFAULT_GOAL := help

override TOKO_TOOLING_ROOT := $(CURDIR)/vendor/toko-tooling
FORMAT_RUST_ARGS := --extra-workspace qualification
include vendor/toko-tooling/make/consumer.mk
include vendor/toko-tooling/make/cargo-dependencies.mk

.PHONY: help snapshot-check test lint doc check-wasm qualification-tooling-check test-compatibility test-allocation lint-compatibility
help:
	@printf '%s\n' \
	  'install-tools         Install the reviewed common host and Rust tools' \
	  'tools-check           Check installed tools offline' \
	  'snapshot-check        Verify installed snapshot bytes offline' \
	  'deps-list             List workspace package versions offline' \
	  'deps-update           Run cargo update -v for maintained workspaces' \
	  '  Optional: DEPENDENCY_PACKAGE=name DEPENDENCY_PRECISE=version' \
	  'test                  Run native contract tests' \
	  'test-compatibility    Compare candidate with published ic-wire 0.22.4' \
	  'test-allocation       Run isolated native allocation checks' \
	  'test-host             Run focused host schema contracts' \
	  'lint-host             Lint host schema library and tests' \
	  'doc-host              Check host feature Rustdoc' \
	  'check-wasm-host       Check TypeScript feature forwarding on Wasm' \
	  'test-typescript       Exchange Rust bytes with published reference TS (Node required)' \
	  'lint-typescript       Lint the focused cross-language qualification harness' \
	  'lint-compatibility    Lint the independent qualification workspace' \
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

qualification-tooling-check: toko-tooling-check
	+@root_selection="$$(perl "$(TOKO_TOOLING_ROOT)/scripts/dev/tooling-lock.pl" "$(CURDIR)")" && \
	  qualification_selection="$$(perl "$(TOKO_TOOLING_ROOT)/scripts/dev/tooling-lock.pl" "$(CURDIR)/qualification")" && \
	  test "$$root_selection" = "$$qualification_selection"

test-compatibility test-allocation lint-compatibility: qualification-tooling-check | toko-execution-check

test-compatibility:
	+cargo test --manifest-path qualification/Cargo.toml --workspace --locked --offline

test-allocation:
	+cargo test --manifest-path qualification/Cargo.toml --test allocation --locked --offline

lint-compatibility:
	+cargo clippy --manifest-path qualification/Cargo.toml --workspace --all-targets --locked --offline -- -D warnings

.PHONY: test-host lint-host doc-host check-wasm-host
test-host lint-host doc-host check-wasm-host: toko-tooling-check | toko-execution-check

test-host:
	+cargo test -p ic-wire --features typescript --test schema --locked --offline

lint-host:
	+cargo clippy -p ic-wire --features typescript --lib --test schema --locked --offline -- -D warnings

doc-host:
	+RUSTDOCFLAGS="-D warnings" cargo doc -p ic-wire --features typescript --no-deps --locked --offline

check-wasm-host:
	+RUSTFLAGS="-D warnings" cargo check -p ic-wire --features typescript --lib --target wasm32-unknown-unknown --locked --offline

.PHONY: test-typescript lint-typescript
test-typescript lint-typescript: qualification-tooling-check | toko-execution-check

test-typescript:
	+cargo test --manifest-path qualification/Cargo.toml --features typescript --test typescript --locked --offline

lint-typescript:
	+cargo clippy --manifest-path qualification/Cargo.toml --features typescript --test typescript --locked --offline -- -D warnings
