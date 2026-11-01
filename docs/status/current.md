# Current handoff

## Core 0.13.5 deployment authorized

The user requested push/deploy on 2026-11-01. Deliver the compatible Core 0.13.5
writer optimization and retained qualification through the standard complete
release gate, direct origin/main Git release and crates.io publication. Endpoint
remains 0.3.0; Runtime/macros/guard are still unpublished. Preserve all evidence,
verify registry archives and installed consumers, and leave the Runtime proposal
isolated until its real supplier graph is qualified.

Core 0.13.4 is published and independently verified on 2026-11-01. Standard full
local release CI passed in 607 s from source `25a4cb9`; release commit
`f83755b30f76704105164e163b9334310e106e44` and annotated tag `core-v0.13.4` were
atomically pushed to origin/main. Both registry archives match admitted upload
bytes, checksums and clean release provenance. A registry-only Rust 1.91 consumer
passes six tests and Wasm compilation with one Core/derive/Endpoint/Types identity
and clean production dependency isolation. Endpoint remains **0.3.0**.
[Delivery evidence and reproduction](../evidence/published-core-0134/README.md).

Current suppliers: Platform/macros/guard 0.16.0, canonical Types 0.15.1,
locked Testkit 0.76.1 (declared minimum 0.76.0), Build/Host 0.73.5 and Metrics 0.24.4. Build's minimum and managed
release editor are 0.73.5; library defaults remain disabled, CLI setup explicit.
Shared 0.27.24 is canonically exported at
1211262f32ea193cc4edcd60d2750baf1f1d6425 (119 payloads). No standalone VERSION,
new codec, compatibility lane or sibling checkout edit was introduced.
Wasm objectives are total **uncompressed** module bytes and separate code-section
bytes. Consumer profiles remain consumer-owned; omit gzip comparisons.

The delivered admission cleanup saves 69 total/code bytes at opt-z and 127 total /
125 code at opt-3 in the 100-type fixture. Maximum smoke IC regression is 0.4121%;
native unit decoding costs about 0.2 µs more while encoding improves. All paired
allocation/heap observations match. The [cost study](../evidence/cost-experiments-0134/README.md)
retains full samples, small controls, rejected candidates and the invalid original
cache-contaminated attempt. The [parser follow-up](../evidence/parser-screen-0134/README.md)
rejects three further rewrites. Those reports keep their exact earlier lockfiles;
the adoption/release checks do not reinterpret them as a new performance matrix.

The 20-case cost-smoke corpus and data-only gate are maintained. The gate checks
matching build/input identities, stable fresh-caller IC samples and unchanged
Candid controls; no raw/code Wasm growth and instructions ≤ baseline × 1.05 + 64.
Eighteen cheap corruption fixtures run in ordinary CI; measurements require an
explicit prepared server. Further tradeoffs require native/IC results separately.

Select one Wasm-only `Output::try_push` inlining hint for compatible Core 0.13.5. The matched 100-type opt-3 fixture shrinks from
398,548 total / 364,120 code bytes to 396,039 / 361,615 (−2,509/−2,505); both opt-z
metrics are identical. The 20-case opt-3 fixture saves 11 total/code bytes.
All 192 IC pairs pass with unchanged Candid artifacts/counters; the maximum
increase is 1.83284% on optional-unit encoding. Map encoding and every decode/
refusal counter are unchanged. Native observations include a ~12 ns scalar
increase and variable directions elsewhere; no native speed win is claimed.
Allocation/heap observations are identical. Library/sampler Clippy and Rust 1.91
Wasm checks pass. No new full CI is claimed for this follow-up.

The [writer investigation](../evidence/writer-inlining-0135/README.md) retains
exact source, all 24 raw modules, native/IC samples, paired controls and rejected
alternatives. Broad inlining regressed map encode by 9.45%; owned slice output
grew opt-z; finalization hints had no effect. The private sampler now records
contract/output source identity. Bytes, budgets, errors and exact retry/hash
inputs remain unchanged; consumers need no migration. This is separate from delivered 0.13.4; the user has now authorized Core 0.13.5
release/publication. Full working outputs remain target/research-0135-writers.

One open issue observed on 2026-11-01: #40, conservative documentation-only CI
selection. It needs an exact prose allowlist, stable checks and full native
fallback. This codec investigation has not implemented or closed it.

## Prepared Endpoint Runtime adoption (#41)

The user requested the cross-crate implementation in [Wire PR #41](https://github.com/dragginzgame/ic-wire/pull/41),
for [Platform #131](https://github.com/dragginzgame/ic-platform/issues/131).
The 15-file integration patch and source qualification are [retained here](../evidence/runtime-owner-017/README.md).
They select independent Runtime directly, keep the facade in qualification clients,
and prove Core/derive/Types and execution identities survive a fake facade break.
Endpoint/schema/diagnostic/strict Clippy, Rust 1.91 host/Wasm, Rust–TypeScript and
installed transaction checks pass. Legacy embedded execution is rejected by links.
The full 17-export / 178-case comparison passes 5,696 measured calls: all 3,204
paired samples (warmups included) have identical IC instruction and cycle counts.
Total Wasm falls 88 bytes at opt-z and opt-3; code sections are unchanged.

Main keeps its published dependencies; no proposed code, supplier patch or source-only
lock is installed here. Prepared providers are frozen at Platform 3a72f6f, Runtime
0.17.0 / Types 0.15.1. Runtime/macros/guard publication and registry qualification
block integration, merge and release. The announced supplier release also includes
Types 0.16: recheck its actual graph, align Core schemas if required, and coordinate
Timers before application adoption. Preserve the pending 0.13.5 writer work.
No merge, staging, commit, release, publication or sibling edit was performed.

## Earlier delivered context

Core 0.13.3 / Endpoint 0.3.0 have [separate delivery evidence](../evidence/published-core-0133-endpoint-030/README.md).
Core's ByteArray re-export is an explicit representation choice: `[0_u8,255]`
changes `00 ff` to `02 00 ff`; coordinate contracts and regenerate clients when
adopting it. Existing arrays retain bytes/admission. Endpoint 0.3 requires aligned
Platform 0.16 execution suppliers and canonical Types 0.15.1.

The [full Candid matrix](../evidence/candid-comparison-0133/README.md) retains
22 cohorts / 1,243 entries / 4,972 primary comparisons, including ordinary-vector
losses and explicit Packed controls. It is not a universal win or a measurement
on today's supplier graph. Protocol bytes, budgets, acceptance and exact retained
retry/hash inputs remain unchanged in Core 0.13.4.

Core 0.13.2 / Endpoint 0.2.1 retain [prior evidence](../evidence/published-core-0132/README.md).
#36/#39 are closed and published; #37 is superseded. The
[stable boundary](../stable-identities.md) keeps codec/schema/Types in Core,
execution in Platform and endpoint admission/trapping replies in Endpoint.
