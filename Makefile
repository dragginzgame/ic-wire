.DEFAULT_GOAL := help

override SHARED_TOOLING_ROOT := $(CURDIR)
SHARED_TOOLING_RUST := 1
LOCAL_TOOL_INSTALL_TARGETS += install-ic-tools
LOCAL_TOOL_CHECK_TARGETS += ic-tools-check
export PATH := $(CURDIR)/.tools/ic/bin:$(PATH)
export CARGO_TARGET_DIR := $(CURDIR)/target
export RUSTUP_AUTO_INSTALL := 0
include make/consumer.mk

.PHONY: help build check test lint check-wasm check-msrv check-doc-links check-pins check-tooling ci version
.PHONY: evaluate-runtime evaluate-macros check-macro-diagnostics
.PHONY: publish check-package
publish:
	+@env cargo publish --locked -p ic-wire-derive -p ic-wire

check-package:
	+@env cargo package --locked --allow-dirty -p ic-wire-derive -p ic-wire --all-features --target-dir target/publication-check

.PHONY: fuzz-runtime

fuzz-runtime:
	+@bash scripts/ci/fuzz-runtime.sh

evaluate-runtime:
	+@bash scripts/ci/evaluate-runtime.sh

evaluate-macros:
	+@bash scripts/ci/evaluate-macros.sh

check-macro-diagnostics:
	+@bash scripts/ci/check-diagnostics.sh macro

.PHONY: check-typed-diagnostics
.PHONY: check-release check-canister-evaluation
.PHONY: check-typescript evaluate-browser check-browser

check-typescript:
	+@bash scripts/ci/check-typescript.sh

evaluate-browser:
	+@bash scripts/ci/evaluate-browser.sh

check-browser:
	+@env cargo run --quiet --locked -p ic-wire-canister-evaluation --example browser-fixture --features bindings -- examples/browser --generate-only
	+@env npm --prefix examples/browser run check
	+@env npm --prefix examples/browser run build

check-canister-evaluation:
	+@env cargo test --locked -p ic-wire-canister-evaluation --lib --features browser-fixture
	+@env cargo test --locked -p ic-wire-canister-evaluation --lib --features candid-codec,runtime-benchmark
	+@env cargo clippy --locked -p ic-wire-canister-evaluation --all-targets --all-features -- -D warnings
	+@env cargo check --locked -p ic-wire-canister-evaluation --lib --target wasm32-unknown-unknown
	+@env cargo check --locked -p ic-wire-canister-evaluation --lib --target wasm32-unknown-unknown --all-features
	+@env RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked -p ic-wire-canister-evaluation --all-targets --all-features
	+@env RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked -p ic-wire-canister-evaluation --lib --target wasm32-unknown-unknown --all-features

check-typed-diagnostics:
	+@bash scripts/ci/check-diagnostics.sh wire

help:
	@printf '%s\n' \
		'IC Wire — bounded Postcard runtime' \
		'Setup: install-tools, install-hooks; offline admission: tools-check' \
		'Rust: build, check, test, lint, check-wasm, check-msrv, fmt, fmt-check' \
		'Evaluation: check-typed-diagnostics (isolated compile-fail checks)' \
		'Evaluation: check-macro-diagnostics, evaluate-macros (local adapter proof)' \
		'Evaluation: evaluate-runtime (production runtime versus Candid)' \
		'Evaluation: check-typescript (Rust/TypeScript runtime conformance)' \
		'Evaluation: fuzz-runtime (prepared nightly/C++/cache; FUZZ_RUNS=100000)' \
		'Evaluation: check-browser, evaluate-browser (prepared Node/npm/Chromium)' \
		'Evaluation: check-canister-evaluation (compile and lint; no emulator)' \
		'Registry: publish (uploads); check-package (no upload; registry access)' \
		'Focused tooling: shared-tooling-check, check-doc-links, check-pins, check-tooling' \
		'Workspace counts: cloc; workspace version: version' \
		'Complete CI gate: ci (explicit local invocation)' \
		'Releases: release-patch, release-minor, release-major, release-resume (Git delivery only)'

build:
	+@env cargo build --locked -p ic-wire

check:
	+@env cargo check --locked -p ic-wire --all-targets

test:
	+@env cargo test --locked -p ic-wire

lint:
	+@env cargo clippy --locked -p ic-wire -p ic-wire-derive --all-targets -- -D warnings

check-wasm:
	+@env cargo check --locked -p ic-wire --lib --target wasm32-unknown-unknown

check-msrv:
	+@env RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked -p ic-wire --lib
	+@env RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked -p ic-wire --lib --target wasm32-unknown-unknown

check-doc-links:
	@perl scripts/ci/check-documentation-links.pl --root . AGENTS.md README.md crates/ic-wire/README.md crates/ic-wire-derive/README.md CHANGELOG.md docs/design.md docs/developer-experience.md docs/evaluation.md docs/evidence.md docs/scaffold-validation.md docs/development.md docs/shared-codec-evaluation.md docs/codec-screen.md docs/canister-comparison.md docs/nested-authority.md docs/consumer-adoption.md docs/browser-interop.md docs/peer-interop.md docs/command-replay.md docs/runtime.md docs/runtime-adoption.md docs/macro-adoption.md docs/runtime-comparison.md docs/consumer-fixtures.md docs/status/current.md

check-pins: host-tools-check
	+@bash scripts/ci/check-dependency-pins.sh --consumer "$(CURDIR)" --cargo-inheritance --npm-root examples/browser --node-version "$$(cat examples/browser/.nvmrc)" --npm-version "$$(jq -r '.packageManager | ltrimstr("npm@")' examples/browser/package.json)"

check-release:
	+@bash scripts/ci/test-local-release.sh

check-tooling:
	+@bash scripts/ci/test-local-make.sh

version: host-tools-check
	+@bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml

ci:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast tools-check shared-tooling-check check-pins check-doc-links check-tooling check-release fmt-check check lint test check-typed-diagnostics check-canister-evaluation check-wasm check-msrv

# Git release delivery is separate from package publication.
export RELEASE_DELIVERY ?= direct
export RELEASE_CACHE_PREPARE ?= 1
export RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_SOURCE RELEASE_COMMIT
export RELEASE_REMOTE RELEASE_BRANCH
.PHONY: release-version release-preflight release-verify release-prepare-version release-prepared-check release-files release-commit-check release-committed-check release-tagged-check release-push-check
release-version:
	@bash scripts/ci/ic-wire-release.sh version
release-preflight:
	+@bash scripts/ci/ic-wire-release.sh preflight
release-verify:
	+@bash scripts/ci/ic-wire-release.sh verify
release-prepare-version:
	+@bash scripts/ci/ic-wire-release.sh prepare
release-prepared-check:
	+@bash scripts/ci/ic-wire-release.sh prepared
release-files:
	@bash scripts/ci/ic-wire-release.sh files
release-commit-check:
	+@bash scripts/ci/ic-wire-release.sh commit
release-committed-check:
	+@bash scripts/ci/ic-wire-release.sh committed
release-tagged-check:
	+@bash scripts/ci/ic-wire-release.sh tagged
release-push-check:
	+@bash scripts/ci/ic-wire-release.sh push
