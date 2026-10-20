# Current handoff

## Current batch

Released 0.5.1 (`cdd47c3`) is the baseline. Pending 0.5.2 initializes the finite
encoder header directly in its bounded starting allocation, avoiding the cold
general growth routine for this fixed-size write. Payload growth and the 0.5.1
allocation policy remain unchanged. No new public API, helper or codec mode.

## Evidence

Matched 50-case probes preserve all frame hashes, lengths and output capacities.
Eighteen finite encode cases improve; 32 controls and all decode samples remain
unchanged. Tiny Relay encodes save 1.4–2.6% instructions under opt-z and 5.8–11.5%
under opt-3. Linked Wasm shrinks 140/896 bytes and gzip shrinks 102/304 bytes for
opt-z/opt-3. No measured encode case regresses. Native timing and installed
consumer latency were not measured.
See [methodology and results](../evaluation.md#direct-header-initialization).
Sources, locks and raw samples are retained at `target/header-initialization/`.

## Qualification

All 38 runtime/native/borrowed tests and 10 doctests pass, together with strict
all-target/all-feature runtime/derive Clippy and Rust 1.91 native/Wasm checks.
Formatting and documentation links pass. Full CI, native macOS and installed
consumer tests were not run. No sibling source was modified.

## Retained decisions

The 0.5.1 output-growth optimization remains. Per-type codec preferences were
reverted because their measured savings did not justify the implementation and
API cost; [experimental evidence](../evaluation.md#public-api-code-generation-adapter)
is retained. No preference attribute or alternate production codec is supported.
Staging, commits, versions, tags, pushes and publication remain maintainer-owned.

## Shared Tooling adoption

Shared Tooling 0.25.8 (`0a2d2ee3b5e0ae77755c2915ac28836d4d5ba359`) replaces
0.24.2 through the canonical exporter with the existing entrypoint selection.
IC Wire selects IC Build 0.23.0's published release metadata editor through
shared setup/offline admission, with an 8 MiB per-document bound. Release CI now
runs the existing lint gate immediately after tool admission; package/publication
recipes also stop on lint failure. Evaluation and TypeScript conformance CLIs use the existing locked Clap
version through development dependencies only. Runtime code and the
pending header optimization were preserved. Native macOS and full CI remain
unrun; release/publication execution remains maintainer-owned.

Adoption checks pass: 104 snapshot files, offline tool/editor admission, disposable
Make and real-editor release fixtures, evaluation tests/Clippy/MSRV, 16 expected
compiler diagnostics, five CLI argument checks, and TypeScript conformance
(90 vectors, 17,524 mutations, 1 MiB boundaries). Maintained inputs contain no
symlinks; package license copies match their canonical owner.

## Further optimization investigation

Extending the Wasm same-pointer comparison shortcut to native builds was tested
and reverted: two paired runs improved 1 MiB borrowed blob decode by 5–6%, but
regressed large borrowed packed manifests by 12–13%. Bytes and capacities matched.
[Full evidence](../evaluation.md#native-borrowed-pointer-equality-experiment)
is retained; no additional runtime change was kept. The header optimization and
Shared Tooling adoption remain intact.
