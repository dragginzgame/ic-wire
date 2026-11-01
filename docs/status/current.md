# Current handoff

## Core 0.13.4 delivered; writer experiments remain isolated

Core 0.13.4 is published and independently verified on 2026-11-01. Standard full
local release CI passed in 607 s from source `25a4cb9`; release commit
`f83755b30f76704105164e163b9334310e106e44` and annotated tag `core-v0.13.4` were
atomically pushed to origin/main. Both registry archives match admitted upload
bytes, checksums and clean release provenance. A registry-only Rust 1.91 consumer
passes six tests and Wasm compilation with one Core/derive/Endpoint/Types identity
and clean production dependency isolation. Endpoint remains **0.3.0**.
[Delivery evidence and reproduction](../evidence/published-core-0134/README.md).

Current suppliers: Platform/macros/guard 0.16.0, canonical Types 0.15.1,
Testkit 0.76.0, Build/Host 0.73.5 and Metrics 0.24.4. Build's minimum and managed
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

The maintainer requested more optimization after deployment. Isolated work is
under target/research-0135-writers, from released 0.13.4 with a common private
sampler contract-source hash. Owned slice output and Wasm-only output inlining
are being screened. No hint or writer change is applied to production; keep
research separate from the completed release. No subsequent release or
publication is authorized by ordinary continuation.

One open issue observed on 2026-11-01: #40, conservative documentation-only CI
selection. It needs an exact prose allowlist, stable checks and full native
fallback. This codec investigation has not implemented or closed it.

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
