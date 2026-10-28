# Current handoff

## Released 0.7.18; pending 0.7.19 fixes and tooling adoption

HEAD is `95a4e30` (local 0.7.19 preparation). The working tree preserves the
benchmark-fixture repair and adds the requested upstream adoption. Own package
versions remain 0.7.18; staging, commits, releases and publication remain with
the maintainer. No remote delivery or full release validation is claimed.

## Selected tooling

Shared Tooling is 0.27.3 at `7ee5c83744aa822ecd070703dbf0a7d95076d5f6`, exported
from the clean committed source through its canonical impact/refresh workflow.
All 111 payloads match that independently selected source commit, including modes;
ordinary offline integrity checking also passes. The existing `.githooks` selection
remains active; its disposable consumer proof exercises the new formatter driver.

Root declarations, Cargo.lock and the managed release editor now select IC
Build/Testkit/host 0.69.5 and Platform 0.7.1, the latest published versions verified
at adoption. Only those six upstream package selections changed in Cargo.lock;
other dependency updates already present at entry were preserved. Tooling 0.69's
Platform 0.7 type boundary is aligned, with no consumer source migration needed.
The editor was explicitly prepared through `make install-tools` and is admitted
offline. Bootstrap remains before Cargo-tool setup. Snapshot-owned files were
not patched locally; optional sccache and scheduled maintenance were not activated.

These packages remain qualification/host dependencies. Published runtime crates
remain independent of platform SDKs. This adoption changes no runtime/generator
execution, protocol bytes, identities, admission rules, retry/hash inputs or
consumer endpoint layout. No performance saving is claimed for dependency adoption.

## Existing issue fixes in the pending patch

Both [#29](https://github.com/dragginzgame/ic-wire/issues/29) and
[#30](https://github.com/dragginzgame/ic-wire/issues/30) are implemented and have
local evidence recorded in their threads. Keep them open pending delivery and
required producer qualification; SDK rollout remains consumer-owned.

Wire metadata accepts container `serde(crate = "path")` without changing layouts.
Unsupported layout-changing attributes, duplicates and invalid placements still
reject. Direct/re-exported record/enum paths retain identical metadata tokens,
frames and generated modules.

Payload-only TS codecs/generation share existing schema validation, traversal,
scalar checks and bulk advice. Unit is empty; u64 7 and `(7u64,)` are `07`.
Existing framed IW/ID/direction APIs retain their contract. Rust caller-slice APIs
share Postcard admission and two-pass sizing; short buffers reject before writing,
other errors publish no successful slice, and output cannot outlive storage.
The [SDK recipe](../sdk-integration.md) covers typed facades, scalar conversions,
JavaScript delivery and SDK-owned buffer, transport, lifecycle and rollback policy.

Allocation admission still reserves the configured byte allowance and weights
visited value/buffer sizes eightfold. For the documented 22-byte response with
`bytes: 64`, allocation 832 encodes and 704 decodes; 512 rejects and 1,024 admits.
The public docs and regression tests clarify this existing rule, not a unit bug.
The retained release failure's `derive(ype)` typo is repaired to `CandidType` in
the private packed-manifest comparison fixture; production code is unaffected.

## Current focused validation

Linux evidence is retained under `target/tooling-adoption-0273/`:

- Snapshot provenance/integrity, managed tool admission, dependency pins and
  external-IC boundaries pass.
- The real grouped formatting selection and maintained documentation links pass.
- Actual Make routing, formatting hook and release-adapter recovery/delivery
  fixtures pass in disposable repositories; no real release effects ran.
- Strict Clippy passes for the benchmark targets (`wire,candid,ic`) and the
  diagnostic example. Nine compile-fail cases plus a passing control pass through
  IC Build 0.69.5; both standard Wire/Candid roundtrip tests pass.
- The diagnostic and admission examples compile on Rust 1.91.
- The current Wasm admission probe passes 144 valid/truncated/resource cases,
  with three samples each, using an explicitly prepared PocketIC 16.0.0 server
  and Testkit-owned lifecycle. Identities/raw results are in `admission-results/`.

Earlier runtime/TS conformance evidence remains in the issue threads and
[caller-storage study](../evidence/caller-storage-0719/README.md). Its original
0.68.12/0.6.21 performance measurements do not qualify the new dependency graph;
no full performance matrix was rerun. The frozen [0.7.16 runtime](../evidence/runtime-0716/README.md)
and [encoder](../evidence/encoder-followup-0716/README.md) studies remain historical.
Full CI, macOS and installed consumer qualification were not run in this batch.
The maintainer's complete release gate remains required for delivery.
