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
	  'install-hooks         Activate the shared formatting hook in this clone' \
	  'check-formatting-hooks Qualify the hook against Wire formatting inputs' \
	  'snapshot-check        Verify installed snapshot bytes offline' \
	  'deps-list             List workspace package versions' \
	  'deps-update           Run cargo update -v for maintained workspaces' \
	  '  Optional: DEPENDENCY_PACKAGE=name DEPENDENCY_PRECISE=version' \
	  'test                  Aggregate product tests (default features)' \
	  '  Focused codec: cargo test -p ic-wire --test encode_tests --locked' \
	  'test-compatibility    Aggregate qualification tests (default features)' \
	  'test-allocation       Run isolated native allocation checks' \
	  'test-host             Run focused host schema contracts' \
	  'lint-host             Lint host schema library and tests' \
	  'doc-host              Check host feature Rustdoc' \
	  'check-wasm-host       Run derived Platform schema on Wasm (Node required)' \
	  'test-typescript       Exchange Rust bytes with current TypeScript (Node required)' \
	  'lint-typescript       Lint the focused cross-language qualification harness' \
	  'check-typescript      Strictly typecheck generated clients (npm ci prerequisite)' \
	  'check-packages        Assemble archives and verify extracted-package consumers' \
	  'check-browser         Chromium smoke (CLIENT_DIR and BROWSER required)' \
	  'test-consumer         Standalone contract against the current codec' \
	  'check-consumer-wasm   Build/run the current Wasm consumer (Node required)' \
	  'measure-consumer-wasm Measure current opt-z/opt-3 consumer Wasm sizes' \
	  'measure-consumer-ic   Installed counters (explicit POCKET_IC_BIN required)' \
	  'lint-instructions    Lint the native driver without starting PocketIC' \
	  'doc-instructions     Check native driver Rustdoc without starting PocketIC' \
	  'test-derive           Check strict derive rejection and typed projection' \
	  'lint-derive           Lint the derive implementation and contract tests' \
	  'lint-compatibility    Aggregate qualification Clippy (default features)' \
	  'lint                  Aggregate product Clippy (default features)' \
	  'doc                   Aggregate product Rustdoc (default features)' \
	  'check-wasm            Aggregate product Wasm check (default features)' \
	  'fmt-check             Aggregate formatting across both workspaces'

snapshot-check:
	@bash "$(TOKO_TOOLING_ROOT)/scripts/ci/verify-toko-tooling-snapshot.sh" --consumer "$(CURDIR)"

test lint doc check-wasm: toko-tooling-check | toko-execution-check

test:
	+cargo test --workspace --locked

lint:
	+cargo clippy --workspace --all-targets --locked -- -D warnings

doc:
	+RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

check-wasm:
	+RUSTFLAGS="-D warnings" cargo check --workspace --lib --target wasm32-unknown-unknown --locked

qualification-tooling-check: toko-tooling-check
	+@root_selection="$$(perl "$(TOKO_TOOLING_ROOT)/scripts/dev/tooling-lock.pl" "$(CURDIR)")" && \
	  qualification_selection="$$(perl "$(TOKO_TOOLING_ROOT)/scripts/dev/tooling-lock.pl" "$(CURDIR)/qualification")" && \
	  test "$$root_selection" = "$$qualification_selection"

.PHONY: lint-instructions doc-instructions check-formatting-hooks

lint-instructions doc-instructions: qualification-tooling-check | toko-execution-check

lint-instructions:
	+cargo clippy --manifest-path qualification/Cargo.toml -p ic-wire-instructions --all-targets --locked -- -D warnings

doc-instructions:
	+RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path qualification/Cargo.toml -p ic-wire-instructions --no-deps --locked

check-formatting-hooks: qualification-tooling-check format-tools-check | toko-execution-check
	+bash scripts/ci/check-formatting-hooks.sh

test-compatibility test-allocation lint-compatibility: qualification-tooling-check | toko-execution-check

test-compatibility:
	+cargo test --manifest-path qualification/Cargo.toml -p ic-wire-compatibility -p ic-wire-consumer --locked

test-allocation:
	+cargo test --manifest-path qualification/Cargo.toml --test allocation --locked

lint-compatibility:
	+cargo clippy --manifest-path qualification/Cargo.toml -p ic-wire-compatibility -p ic-wire-consumer --all-targets --locked -- -D warnings

.PHONY: test-host lint-host doc-host check-wasm-host
test-host lint-host doc-host check-wasm-host: toko-tooling-check | toko-execution-check

test-host:
	+cargo test -p ic-wire --features typescript --test schema --locked

lint-host:
	+cargo clippy -p ic-wire --features typescript --lib --test schema --locked -- -D warnings

doc-host:
	+RUSTDOCFLAGS="-D warnings" cargo doc -p ic-wire --features typescript --no-deps --locked

check-wasm-host: qualification-tooling-check
	+RUSTFLAGS="-D warnings" cargo check -p ic-wire --features typescript --lib --target wasm32-unknown-unknown --locked
	+RUSTFLAGS="-D warnings" cargo build --manifest-path qualification/Cargo.toml -p ic-wire-consumer --features schema,wire/platform-types --example platform-schema --profile wire-size --target wasm32-unknown-unknown --locked
	node qualification/ic-wire-consumer/check-wasm.mjs qualification/target/wasm32-unknown-unknown/wire-size/examples/platform-schema.wasm

.PHONY: test-typescript lint-typescript check-typescript
test-typescript lint-typescript check-typescript: qualification-tooling-check | toko-execution-check

check-typescript:
	+IC_WIRE_CHECK_TYPESCRIPT=1 cargo test --manifest-path qualification/Cargo.toml --features typescript --test typescript --locked

test-typescript:
	+cargo test --manifest-path qualification/Cargo.toml --features typescript --test typescript --locked

lint-typescript:
	+cargo clippy --manifest-path qualification/Cargo.toml --features typescript --test typescript --locked -- -D warnings

.PHONY: test-consumer check-consumer-wasm
test-consumer check-consumer-wasm: qualification-tooling-check | toko-execution-check

.PHONY: measure-consumer-wasm
measure-consumer-wasm: qualification-tooling-check | toko-execution-check
	node qualification/ic-wire-consumer/measure-wasm.mjs

.PHONY: measure-consumer-ic
measure-consumer-ic: qualification-tooling-check | toko-execution-check
	bash qualification/ic-wire-instructions/run.sh

test-consumer:
	+cargo test --manifest-path qualification/Cargo.toml -p ic-wire-consumer --test contract --locked

check-consumer-wasm:
	+cargo build --manifest-path qualification/Cargo.toml -p ic-wire-consumer --bin wire-consumer --target wasm32-unknown-unknown --target-dir qualification/target/consumer --locked
	node qualification/ic-wire-consumer/check-wasm.mjs qualification/target/consumer/wasm32-unknown-unknown/debug/wire-consumer.wasm

.PHONY: test-derive lint-derive
test-derive lint-derive: toko-tooling-check | toko-execution-check

test-derive:
	+cargo test -p ic-wire-derive --lib --locked
	+cargo test -p ic-wire --features derive --test derive_tests --locked

lint-derive:
	+cargo clippy -p ic-wire-derive --lib --tests --locked -- -D warnings
	+cargo clippy -p ic-wire --features derive --lib --test derive_tests --locked -- -D warnings

.PHONY: check-packages check-browser
check-packages: qualification-tooling-check | toko-execution-check
	bash qualification/check-packages.sh

check-browser:
	bash qualification/browser/check.sh "$(CLIENT_DIR)" "$(BROWSER)"
