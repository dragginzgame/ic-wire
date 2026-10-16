.DEFAULT_GOAL := help

override SHARED_TOOLING_ROOT := $(CURDIR)
SHARED_TOOLING_RUST := 1
export CARGO_TARGET_DIR := $(CURDIR)/target
export RUSTUP_AUTO_INSTALL := 0
include make/consumer.mk

.PHONY: help build check test lint check-wasm check-msrv check-doc-links check-pins check-tooling ci version release-policy-check
.PHONY: evaluate-codecs evaluate-journey check-typed-diagnostics evaluate-canisters
.PHONY: check-canister-evaluation
.PHONY: evaluate-browser check-browser

evaluate-browser:
	+@bash scripts/ci/evaluate-browser.sh

check-browser:
	+@env cargo run --quiet --locked -p ic-wire-canister-evaluation --example browser-fixture --features bindings -- examples/browser --generate-only
	+@env npm --prefix examples/browser run check
	+@env npm --prefix examples/browser run build

check-canister-evaluation:
	+@env cargo clippy --locked -p ic-wire-canister-evaluation --all-targets --all-features -- -D warnings
	+@env cargo check --locked -p ic-wire-canister-evaluation --lib --target wasm32-unknown-unknown
	+@env cargo check --locked -p ic-wire-canister-evaluation --lib --target wasm32-unknown-unknown --all-features
	+@env RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked -p ic-wire-canister-evaluation --all-targets --all-features
	+@env RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked -p ic-wire-canister-evaluation --lib --target wasm32-unknown-unknown --all-features

evaluate-canisters:
	+@bash scripts/ci/evaluate-canisters.sh

check-typed-diagnostics:
	+@bash scripts/ci/test-typed-diagnostics.sh

evaluate-codecs:
	+@env cargo run --quiet --locked -p ic-wire --example codec-evaluation

evaluate-journey:
	+@env cargo run --quiet --locked -p ic-wire --example typed-journey

help:
	@printf '%s\n' \
		'IC Wire — unpublished no_std investigation; no codec implemented' \
		'Setup: install-tools, install-hooks; offline admission: tools-check' \
		'Rust: build, check, test, lint, check-wasm, check-msrv, fmt, fmt-check' \
		'Evaluation: evaluate-codecs (JSON payload-size and round-trip report)' \
		'Evaluation: evaluate-journey (native typed request/reply prototype)' \
		'Evaluation: check-typed-diagnostics (isolated compile-fail checks)' \
		'Evaluation: evaluate-canisters (requires explicit POCKET_IC_BIN)' \
		'Evaluation: check-browser, evaluate-browser (prepared Node/npm/Chromium)' \
		'Evaluation: check-canister-evaluation (compile and lint; no emulator)' \
		'Focused tooling: shared-tooling-check, check-doc-links, check-pins, check-tooling' \
		'Workspace counts: cloc; workspace version: version' \
		'Complete CI gate: ci (explicit local invocation)' \
		'Releases: release-patch, release-minor, release-major, release-resume are disabled'

build:
	+@env cargo build --locked -p ic-wire

check:
	+@env cargo check --locked -p ic-wire --all-targets

test:
	+@env cargo test --locked -p ic-wire

lint:
	+@env cargo clippy --locked -p ic-wire --all-targets -- -D warnings

check-wasm:
	+@env cargo check --locked -p ic-wire --lib --target wasm32-unknown-unknown

check-msrv:
	+@env RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked -p ic-wire --lib
	+@env RUSTUP_TOOLCHAIN=1.91.0 cargo check --locked -p ic-wire --lib --target wasm32-unknown-unknown

check-doc-links:
	@perl scripts/ci/check-documentation-links.pl --root . AGENTS.md README.md CHANGELOG.md docs/design.md docs/developer-experience.md docs/evaluation.md docs/evidence.md docs/scaffold-validation.md docs/development.md docs/shared-codec-evaluation.md docs/codec-screen.md docs/canister-comparison.md docs/browser-interop.md docs/status/current.md

check-pins: host-tools-check
	+@bash scripts/ci/check-dependency-pins.sh --consumer "$(CURDIR)" --cargo-inheritance --npm-root examples/browser --node-version "$$(cat examples/browser/.nvmrc)" --npm-version "$$(jq -r '.packageManager | ltrimstr("npm@")' examples/browser/package.json)"

check-tooling:
	+@bash scripts/ci/test-local-make.sh

version: host-tools-check
	+@bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml

ci:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast tools-check shared-tooling-check check-pins check-doc-links check-tooling fmt-check check lint test check-typed-diagnostics check-canister-evaluation check-wasm check-msrv

# The investigation has no release destination or accepted release contract.
# Keep the common entrypoints, but reject before invoking the shared runner.
release-patch release-minor release-major release-resume: release-policy-check
release-policy-check:
	@printf '%s\n' 'IC Wire releases are disabled: this investigation remains unpublished; the maintainer must authorize and configure release delivery first.' >&2
	@exit 2
