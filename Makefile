.DEFAULT_GOAL := help

override SHARED_TOOLING_ROOT := $(CURDIR)
SHARED_TOOLING_RUST := 1
export CARGO_TARGET_DIR := $(CURDIR)/target
export RUSTUP_AUTO_INSTALL := 0
LOCAL_TOOL_INSTALL_TARGETS += install-release-editor
LOCAL_TOOL_CHECK_TARGETS += release-editor-check
export RELEASE_GROUP
# Refuse ambiguous group effects before any setup, gate or release runner.
ifneq ($(filter-out release-editor-check,$(filter release-% publish publish-check version,$(MAKECMDGOALS))),)
ifneq ($(words $(RELEASE_GROUP)),1)
$(error Select RELEASE_GROUP=core or endpoint explicitly)
endif
ifeq ($(filter core endpoint,$(RELEASE_GROUP)),)
$(error Select RELEASE_GROUP=core or endpoint explicitly)
endif
endif
RELEASE_EDITOR_VERSION := 0.75.0
export CARGO_RELEASE_METADATA_MAX_BYTES := 8388608
RELEASE_EDITOR_COMMAND = bash "$(CURDIR)/scripts/dev/install-rust-tools.sh" --consumer "$(CURDIR)" \
    --package ic-build --version "$(RELEASE_EDITOR_VERSION)" --bin ic-build-release-metadata --profile debug --features cli
override CARGO_PUBLISH_PACKAGES := ic-wire-derive ic-wire ic-wire-endpoint-macros ic-wire-endpoint
# Publication is scoped to an explicitly selected group, including PACKAGE subsets.
ifneq ($(filter publish publish-check check-package,$(MAKECMDGOALS)),)
ifeq ($(RELEASE_GROUP),core)
override CARGO_PUBLISH_PACKAGES := ic-wire-derive ic-wire
else ifeq ($(RELEASE_GROUP),endpoint)
override CARGO_PUBLISH_PACKAGES := ic-wire-endpoint-macros ic-wire-endpoint
else
$(error Select RELEASE_GROUP=core or endpoint explicitly for publication/archive checks)
endif
endif
override CARGO_PUBLISH_REGISTRY := crates-io
override CARGO_PUBLISH_VERIFY_TARGETS := lint
FORMAT_RUST_ARGS := --rustfmt-source crates/ic-wire-qualification/benches/endpoint_generation/main.rs --rustfmt-edition 2024
# Reject the retired selector rather than silently publishing the default roster.
ifneq ($(filter publish publish-check,$(MAKECMDGOALS)),)
ifneq ($(origin PUBLISH_PACKAGES),undefined)
$(error PUBLISH_PACKAGES was replaced by PACKAGE; use PACKAGE=ic-wire for a selected attempt)
endif
endif
include make/consumer.mk

# Local compiler policy is admitted before Cargo; no compiler version lives here.
RUST_POLICY = bash "$(CURDIR)/scripts/ci/rust-policy.sh"

.PHONY: install-release-editor release-editor-check
install-release-editor:
	+@$(RELEASE_EDITOR_COMMAND)
release-editor-check:
	+@$(RELEASE_EDITOR_COMMAND) --check

.PHONY: help build check test lint check-wasm check-msrv check-doc-links check-pins check-tooling ci ci-core ci-typescript ci-benchmarks version
# One declaration in ci/release-validation.json; Shared admits the Cargo-group projection.
# macOS system Make predates .SHELLSTATUS; retain failure admission in shell output.
override IC_WIRE_CI_TARGETS := $(shell perl "$(CURDIR)/scripts/ci/wire-validation-roster.pl" || printf '%s\n' WIRE_ROSTER_REFUSED)
ifneq ($(filter WIRE_ROSTER_REFUSED,$(IC_WIRE_CI_TARGETS)),)
$(error Wire validation roster admission failed)
endif
.PHONY: check-package
check-package:
	+@$(MAKE) lint
	+@$(RUST_POLICY) cargo package --locked --allow-dirty $(foreach package,$(CARGO_PUBLISH_PACKAGES),-p $(package)) --all-features --target-dir target/publication-check

.PHONY: evaluate-codecs evaluate-codec-matrix evaluate-codec-instructions check-benchmarks
evaluate-codecs:
	+@$(RUST_POLICY) bash scripts/ci/evaluate-codecs.sh "$(CURDIR)/target/evidence/codec-comparison"

evaluate-codec-matrix:
	+@$(RUST_POLICY) bash scripts/ci/evaluate-codec-matrix.sh "$(CURDIR)/target/evidence/codec-matrix"

evaluate-codec-instructions:
	+@$(RUST_POLICY) cargo run --locked -p ic-wire-benchmarks --example codec-instructions --features wire,candid,ic -- "$(CURDIR)/target/evidence/codec-comparison"

check-benchmarks:
	+@$(MAKE) check-benchmark-cli
	+@$(RUST_POLICY) cargo test --locked -p ic-wire-benchmarks --example codec-report
	+@$(RUST_POLICY) cargo test --locked -p ic-wire-benchmarks --features wire,candid --example codec-wasm
	+@$(RUST_POLICY) cargo clippy --locked -p ic-wire-benchmarks --all-targets --features wire,candid,ic -- -D warnings
	+@for mode in allocations heap; do \
	    $(RUST_POLICY) cargo clippy --locked -p ic-wire-benchmarks --all-targets --features "wire,candid,$$mode" -- -D warnings || exit $$?; \
	done
	+@$(RUST_POLICY) cargo check --locked -p ic-wire-benchmarks --example codec-wasm --target wasm32-unknown-unknown --features wire,candid
	+@$(MAKE) check-consumer-benchmarks

.PHONY: check-benchmark-cli
check-benchmark-cli: check-codec-costs-tests
	+@$(RUST_POLICY) node scripts/ci/check-benchmark-cli.mjs

.PHONY: check-codec-costs check-codec-costs-tests
export CODEC_BASELINE_DIR CODEC_CANDIDATE_DIR
check-codec-costs-tests:
	+@node scripts/ci/check-codec-costs.test.mjs

# Data-only admission: measurement/server preparation remains explicit.
check-codec-costs:
	@test -n "$${CODEC_BASELINE_DIR:-}" && test -n "$${CODEC_CANDIDATE_DIR:-}" || { echo 'Set CODEC_BASELINE_DIR and CODEC_CANDIDATE_DIR to completed matched runs' >&2; exit 2; }
	+@node scripts/ci/check-codec-costs.mjs "$$CODEC_BASELINE_DIR" "$$CODEC_CANDIDATE_DIR"

.PHONY: check-consumer-benchmarks
check-consumer-benchmarks:
	+@for corpus in coverage-shapes research-bulk contract-shapes cost-smoke byte-keys byte-keys-control hotspots toko derive-scale-1 derive-scale-10 derive-scale packed-i16 packed-i16-baseline packed-scale packed-scale-shared,derive-scale-10 bulk bulk,bulk-baseline bulk-scale bulk-scale,bulk-baseline; do \
	    $(RUST_POLICY) cargo test --locked -p ic-wire-benchmarks --features "wire,candid,$$corpus" --example codec-wasm && \
	    $(RUST_POLICY) cargo clippy --locked -p ic-wire-benchmarks --all-targets --features "wire,candid,ic,$$corpus" -- -D warnings || exit $$?; \
	done
	+@for corpus in coverage-shapes research-bulk contract-shapes cost-smoke byte-keys byte-keys-control hotspots toko derive-scale packed-i16 packed-scale bulk bulk-scale; do \
	    $(RUST_POLICY) cargo check --locked -p ic-wire-benchmarks --example codec-wasm --target wasm32-unknown-unknown --features "wire,candid,$$corpus" || exit $$?; \
	done

.PHONY: evaluate-allocation
evaluate-allocation:
	+@$(RUST_POLICY) cargo run --locked --profile evaluation -p ic-wire-qualification --example allocation --features allocation-fixture -- target/evidence/allocation-comparison evaluation

.PHONY: fuzz-runtime

fuzz-runtime:
	+@bash scripts/ci/fuzz-runtime.sh

.PHONY: check-typed-diagnostics
.PHONY: check-release check-qualification check-compatible-consumers
.PHONY: check-typescript check-platform-types check-stable-identities

check-typescript:
	+@bash scripts/ci/check-typescript.sh

check-stable-identities:
	+@bash scripts/ci/check-stable-identities.sh

check-platform-types:
	+@bash scripts/ci/check-platform-types.sh
	+@bash scripts/ci/check-stable-identities.sh

check-qualification:
	+@bash scripts/ci/check-qualification-tools.sh
	+@$(RUST_POLICY) cargo test --locked -p ic-wire-qualification --lib --tests --example check-diagnostics --example check-contract --features allocation-fixture,schema-fixture
	+@$(RUST_POLICY) cargo clippy --locked -p ic-wire-qualification --all-targets --features allocation-fixture,schema-fixture -- -D warnings

check-compatible-consumers:
	+@bash scripts/ci/check-compatible-consumers.sh

check-typed-diagnostics:
	+@bash scripts/ci/check-diagnostics.sh

.PHONY: check-endpoint check-endpoint-ic evaluate-endpoints
check-endpoint:
	+@bash scripts/ci/check-endpoint.sh

# Explicit prepared PocketIC only; not part of the ordinary native/compile gate.
check-endpoint-ic:
	+@bash scripts/ci/check-endpoint-ic.sh

# Explicit cost study; EVIDENCE_DIR must be a new absolute directory.
evaluate-endpoints:
	+@bash scripts/ci/evaluate-endpoint-matrix.sh "$(EVIDENCE_DIR)"

help:
	@printf '%s\n' \
		'IC Wire — bounded Postcard runtime' \
		'Setup: install-tools, install-hooks; offline admission: tools-check' \
		'Rust: build, check, test, lint, check-wasm, check-msrv, fmt, fmt-check' \
		'Evaluation: check-typed-diagnostics (isolated compile-fail checks)' \
		'Evaluation: check-typescript (Rust/TypeScript runtime conformance)' \
		'Qualification: check-platform-types (standalone Principal/Account, MSRV and feature isolation)' \
		'Evaluation: evaluate-codecs / evaluate-codec-matrix (timing, heap, Wasm, explicit PocketIC)' \
		'Qualification: check-codec-costs CODEC_BASELINE_DIR=... CODEC_CANDIDATE_DIR=... (completed evidence only)' \
		'Qualification: check-benchmarks (comparison corpus, lint and MSRV)' \
		'Evaluation: evaluate-allocation (borrowing, buffers and bounded/packed layouts)' \
		'Evaluation: fuzz-runtime (prepared nightly/C++/cache; FUZZ_RUNS=100000)' \
		'Qualification: check-qualification (allocation fixtures and compiler support)' \
		'Qualification: check-endpoint; check-endpoint-ic requires prepared POCKET_IC_BIN' \
		'Evaluation: evaluate-endpoints EVIDENCE_DIR=/absolute/new/path (instrumented POCKET_IC_BIN)' \
		'Registry: publish (uploads); publish-check (dry run); PACKAGE selects a subset; check-package (archives)' \
		'Focused tooling: shared-tooling-check, check-doc-links, check-pins, check-boundary, check-tooling' \
		'Workspace counts: cloc; workspace version: version' \
		'Complete CI gate: ci (explicit local invocation)' \
		'Hosted CI groups: ci-core, ci-typescript, ci-benchmarks (separate runners)' \
		'Releases: release-patch, release-minor, release-major, release-resume (Git delivery only)'

build:
	+@$(RUST_POLICY) cargo build --locked -p ic-wire

check:
	+@$(RUST_POLICY) cargo check --locked -p ic-wire --all-targets

test:
	+@$(RUST_POLICY) cargo test --locked -p ic-wire

lint:
	+@bash scripts/ci/check-lint.sh

check-wasm:
	+@$(RUST_POLICY) cargo check --locked -p ic-wire --lib --target wasm32-unknown-unknown

.PHONY: check-rust-policy
check-rust-policy: host-tools-check
	+@$(RUST_POLICY)

# Explicit convenience target; CI already owns check and check-wasm separately.
check-msrv: check check-wasm

check-doc-links:
	@bash scripts/ci/check-repository-documentation.sh

.PHONY: check-boundary check-candid-boundary check-browser-boundary
check-candid-boundary:
	@perl scripts/ci/check-candid-boundary.pl --cbor "$(CURDIR)" ci/candid-boundary.json crates examples scripts/benchmarks

check-browser-boundary:
	@node scripts/ci/check-ic-sdk-boundary.mjs examples/browser crates scripts/dev/inspect-wire.mjs

check-boundary: check-candid-boundary check-browser-boundary
	+@bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$(CURDIR)" --external-ic

.PHONY: check-ci-selection
check-ci-selection: host-tools-check
	+@node scripts/ci/check-ci-selection.mjs

check-pins: host-tools-check
	+@bash scripts/ci/check-dependency-pins.sh --consumer "$(CURDIR)" --cargo-inheritance --npm-root examples/browser --node-version "$$(cat examples/browser/.nvmrc)" --npm-version "$$(jq -r '.packageManager | ltrimstr("npm@")' examples/browser/package.json)"

.PHONY: check-release-validation validation-targets
validation-targets:
	@printf '%s\n' $(IC_WIRE_CI_TARGETS)

check-release-validation:
	+@node scripts/ci/check-release-validation.mjs

check-release:
	+@bash scripts/ci/test-local-release.sh core
	+@bash scripts/ci/test-local-release.sh endpoint

check-tooling:
	+@bash scripts/ci/test-local-make.sh

version: host-tools-check
	+@bash scripts/ci/ic-wire-release.sh version

ci:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast $(IC_WIRE_CI_TARGETS)

ci-core:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast $(filter-out check-benchmarks check-typescript,$(IC_WIRE_CI_TARGETS))

ci-typescript:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast tools-check check-rust-policy check-typescript

ci-benchmarks:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast tools-check check-rust-policy check-benchmarks

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
