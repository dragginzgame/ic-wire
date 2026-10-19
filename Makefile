.DEFAULT_GOAL := help

override SHARED_TOOLING_ROOT := $(CURDIR)
SHARED_TOOLING_RUST := 1
export CARGO_TARGET_DIR := $(CURDIR)/target
export RUSTUP_AUTO_INSTALL := 0
include make/consumer.mk

.PHONY: help build check test lint check-wasm check-msrv check-doc-links check-pins check-tooling ci version
.PHONY: publish check-package
publish:
	+@env cargo publish --locked -p ic-wire-derive -p ic-wire

check-package:
	+@env cargo package --locked --allow-dirty -p ic-wire-derive -p ic-wire --all-features --target-dir target/publication-check

.PHONY: evaluate-performance
evaluate-performance:
	+@env cargo run --locked --profile evaluation -p ic-wire-canister-evaluation --example performance --features performance-postcard,performance-bitcode,performance-wincode -- target/native-performance

.PHONY: evaluate-messagepack
evaluate-messagepack:
	+@env cargo run --locked --profile evaluation -p ic-wire-canister-evaluation --example messagepack --features comparison-postcard,comparison-messagepack,comparison-wire -- target/messagepack-comparison evaluation

.PHONY: fuzz-runtime

fuzz-runtime:
	+@bash scripts/ci/fuzz-runtime.sh

.PHONY: check-typed-diagnostics
.PHONY: check-release check-evaluation
.PHONY: check-typescript

check-typescript:
	+@bash scripts/ci/check-typescript.sh

check-evaluation:
	+@env cargo test --locked -p ic-wire-canister-evaluation --lib --features performance-wire,performance-postcard,performance-bitcode,performance-wincode,comparison-postcard,comparison-messagepack,comparison-wire
	+@env cargo clippy --locked -p ic-wire-canister-evaluation --all-targets --all-features -- -D warnings
	+@env RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked -p ic-wire-canister-evaluation --all-targets --all-features

check-typed-diagnostics:
	+@bash scripts/ci/check-diagnostics.sh

help:
	@printf '%s\n' \
		'IC Wire — bounded Postcard runtime' \
		'Setup: install-tools, install-hooks; offline admission: tools-check' \
		'Rust: build, check, test, lint, check-wasm, check-msrv, fmt, fmt-check' \
		'Evaluation: check-typed-diagnostics (isolated compile-fail checks)' \
		'Evaluation: check-typescript (Rust/TypeScript runtime conformance)' \
		'Evaluation: evaluate-performance (native timing only; fresh target/native-performance output)' \
		'Evaluation: evaluate-messagepack (native comparison; fresh target/messagepack-comparison output)' \
		'Evaluation: fuzz-runtime (prepared nightly/C++/cache; FUZZ_RUNS=100000)' \
		'Evaluation: check-evaluation (native codec controls and compiler support)' \
		'Registry: publish (uploads); check-package (no upload; registry access)' \
		'Focused tooling: shared-tooling-check, check-doc-links, check-pins, check-boundary, check-tooling' \
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
	@perl scripts/ci/check-documentation-links.pl --root . AGENTS.md README.md crates/ic-wire/README.md crates/ic-wire-derive/README.md CHANGELOG.md docs/design.md docs/evaluation.md docs/scaffold-validation.md docs/development.md docs/runtime.md docs/performance.md docs/messagepack-comparison.md docs/runtime-optimization.md docs/consumer-fixtures.md docs/status/current.md

.PHONY: check-boundary
check-boundary:
	+@bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$(CURDIR)"

check-pins: host-tools-check
	+@bash scripts/ci/check-dependency-pins.sh --consumer "$(CURDIR)" --cargo-inheritance --npm-root examples/browser --node-version "$$(cat examples/browser/.nvmrc)" --npm-version "$$(jq -r '.packageManager | ltrimstr("npm@")' examples/browser/package.json)"

check-release:
	+@bash scripts/ci/test-local-release.sh

check-tooling:
	+@bash scripts/ci/test-local-make.sh

version: host-tools-check
	+@bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml

ci:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast tools-check shared-tooling-check check-pins check-boundary check-doc-links check-tooling check-release fmt-check check lint test check-typed-diagnostics check-evaluation check-wasm check-msrv

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
