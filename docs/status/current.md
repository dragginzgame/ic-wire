# Current handoff

## Core 0.13.5 delivered; Endpoint remains 0.3.0

The maintainer requested push/deploy on 2026-11-01. Source `422b3f5` passed the
standard complete local CI gate in **567 s**. The runner prepared only Core,
created release commit `a8195d42683f90810a9afd615e4f27e3e5a3c744` and annotated
`core-v0.13.5` (object `6e6aff91a74f8436a1541eebac112b4aa9a13247`), then atomically
pushed origin/main and that tag. Core/derive 0.13.5 are published and independently
verified against registry checksums, prepared/uploaded archives and clean source.
[Delivery evidence, receipts and reproduction](../evidence/published-core-0135/README.md).

A registry-only Rust 1.91 consumer passes six tests and Wasm compilation with one
Core/derive 0.13.5, Endpoint 0.3.0 and canonical Types identity. Production Wasm
excludes Candid, simulators, serde_json and host tooling. No source patch is needed.
The provenance helper fetched the exact release source from the canonical repo.
All build outputs, failed preparations and delivery/publication evidence remain.

Published 0.13.5 suppliers were Platform/macros/guard 0.16.0 and Types 0.15.1.
This pending integration selects Runtime/macros/guard 0.17.1 and DTO Types 0.16,
with native qualification on facade 0.16 / locked Testkit 0.76.1
(declared minimum 0.76.0), Build/Host 0.73.5 and Metrics 0.24.4. Shared 0.27.24 is
exported at `1211262f32ea193cc4edcd60d2750baf1f1d6425` (119 payloads).
Library defaults remain disabled; managed CLI setup is explicit. No sibling
checkout was edited. Future ordinary continuation does not authorize another release.

## Delivered writer optimization

Core 0.13.5 keeps one Wasm-only `Output::try_push` inlining hint. In the matched
100-type opt-3 fixture, total/code bytes fall 398,548/364,120 → 396,039/361,615
(−2,509/−2,505); both opt-z metrics are identical. The 20-case opt-3 fixture saves
11 total/code bytes. All 192 IC pairs pass with unchanged Candid artifacts/counters;
maximum increase is 1.83284% on optional-unit encoding. Every decode/refusal
counter and map encoding are unchanged. Native observations include a ~12 ns
scalar increase and variable directions; no native speed win is claimed.
Allocation/heap observations match. Bytes, budgets, errors and retry/hash inputs
are unchanged; no consumer migration. The sampler records contract/output identity.

[Writer evidence](../evidence/writer-inlining-0135/README.md) retains all 24 raw
modules, repeated native/IC samples, controls and rejected candidates. Broad hints
regressed map encode by 9.45%; owned slices grew opt-z; finalization had no effect.
Raw work remains `target/research-0135-writers`. Objectives are total uncompressed
Wasm and separate code-section bytes; consumer profiles remain consumer-owned.
The complete release gate checks corpora but is not a fresh full performance matrix.

## Registry Runtime / Types adoption in PR #41

Suppliers are now published: Runtime/macros/guard 0.17.1, Types 0.16 and facade
0.17. The PR selects Runtime directly and adopts canonical Types 0.16 schemas.
Pending ledgers classify Core 0.14 and Endpoint 0.4; own package versions are still
0.13.5/0.3 during implementation and no new publication is performed.
The [registry qualification](../evidence/registry-runtime-0171/README.md) records
standalone schemas, exact Rust/browser bytes/refusals, MSRV host/Wasm, strict
Clippy, compiler diagnostics, one DTO/runtime owner and legacy-guard refusal.
Installed transaction/admission/rollback/async/dispatch checks pass.
Production Wasm excludes the facade and host/native client dependencies. Native
qualification deliberately keeps the matching facade 0.16 / Testkit client; its
old Types do not implement or supply new DTO schemas. The crypto-only future
facade fixture preserves Core/derive/DTO and execution identities.
The matched 17-export / 178-case comparison passes 5,696 measured calls: all
3,204 paired samples have identical IC instructions and cycle debits. Each profile
adds 40 total Wasm bytes with unchanged code sections. Source, raw samples and
artifacts are retained in the registry report. Source-only historical evidence is
not a registry acceptance claim. Required hosted checks/reviews remain independent.

## Other open work and earlier evidence

#40 remains open: conservative documentation-only CI selection, with an exact
prose allowlist, stable checks and full native fallback. It is not implemented.
The [full Candid matrix](../evidence/candid-comparison-0133/README.md) retains
22 cohorts / 1,243 entries / 4,972 comparisons including ordinary-vector losses;
it describes its frozen supplier graph, not every current consumer workload.
Core 0.13.4 has [separate delivery evidence](../evidence/published-core-0134/README.md).
Core 0.13.3 / Endpoint 0.3 have [earlier evidence](../evidence/published-core-0133-endpoint-030/README.md).
ByteArray remains an explicit choice: `[0_u8,255]` changes `00 ff` → `02 00 ff`;
coordinate contracts when adopting it. Existing arrays retain bytes/admission.
#36/#39 are published/closed and #37 superseded. The [stable boundary](../stable-identities.md)
keeps codec/schema/Types in Core and endpoint admission/trapping replies in Endpoint.
