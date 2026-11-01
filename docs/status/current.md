# Current handoff

## Authorized delivery: Core 0.13.3 / Endpoint 0.3.0

The maintainer requested deployment on 2026-11-01. Source preparation is in
progress; Git releases and crates.io publication are authorized for both groups,
Core first. Full release CI and independent registry verification must complete
before recording delivery. Own manifests are still Core 0.13.2 / Endpoint 0.2.1
until the release adapters prepare their respective versions.

Core adds the standard `ic_wire::ByteArray<N>` re-export and exact-width host
metadata, including standard fixed-array byte adapters and options. Existing
arrays retain bytes/admission. Explicit ByteArray adoption changes `[0_u8,255]`
from `00 ff` to `02 00 ff` and uses bulk admission; coordinate contracts,
regenerate clients and qualify limits/retry inputs. One Serde/Postcard codec,
Types 0.15.1, schema trait and Core/derive identities remain.

The [matched byte-key study](../evidence/byte-array-0133/README.md) freezes one
source/lock for both representations. At 128 keys, opt-3 encode instructions
fall 288,996.25 → 62,121.25 (−78.50%), decode 699,614 → 221,623 (−68.32%).
Both beat Candid. Frame bytes rise 6,349 → 6,477; total Wasm rises 72,482 →
74,008 (+1,526, 2.11%). Opt-z adds 1,990 Wasm bytes. Empty opt-3 decode rises
375 → 393 instructions (+4.80%). Keep this explicit speed/size tradeoff visible.
Three native tests, both corpus controls, strict Clippy, Rust 1.91 host/Wasm,
standalone derives and alloc-only isolation pass; TS covers 176 vectors and
38,556 differential mutations. All 160 paired IC rows have stable counters.

The [full Candid matrix](../evidence/candid-comparison-0133/README.md) retains
22 cohorts / 1,243 entries / 4,972 primary comparisons, including 333 standard
cases with 35 appended workloads. Wire wins 4,462; Candid wins 510. All 88 raw
modules and exact sources are retained. Wire has smaller raw/code Wasm in all
44 cohort/profile pairs; 100 ordinary types at opt-3 are 398,675 versus
2,106,847 total bytes. Ordinary-vector losses remain, including 51.09× opt-3
encode; explicit generic Packed wins all 436 comparisons. No universal win.
Frozen Platform 0.15.3 / Metrics 0.24.3 differs from later checkout selections;
new adoption is not a reinterpretation or full rerun of those measurements.

Endpoint 0.3 adopts published Platform/macros/guard 0.16.0; align application
runtime suppliers, including timers. Testkit 0.75.2 retains native-only Platform
0.15.3; production Wasm selects only the 0.16 runtime/guard. Canonical Types
stays 0.15.1, with unchanged Principal/Account Rust/TS bytes and malformed rules.
[Adoption evidence](../evidence/platform-016-adoption/README.md) includes installed
admission, async/composite calls, reply/transaction rollback, standalone
schema-only/combined graphs, Rust 1.91, stable identities and 333-case codec tests.
The diagnostic recipe supplies matching CARGO_CRATE_NAME for direct rustc.
Selected runtime/Candid source trees are byte-identical to 0.15.3; no performance
saving is claimed. Existing Wire bytes, budgets, exports and retry/hash inputs
are unchanged by the supplier update. No sibling checkout was edited.

Build/Host 0.73.3 and Testkit 0.75.2 requirements are adopted; the incoming lock
already selected Build/Host 0.73.4 and Metrics 0.24.4 and is preserved. The release
editor remains explicitly 0.73.3. Shared snapshot 0.27.23 at
619d194875369e0f72b45477b66adb430acbf6b7 supplies 119 canonically exported payloads.
[Earlier tooling qualification](../evidence/upstream-20261101/README.md) retains
its original graph and disposable release/receipt fixtures. Cargo groups remain
independent; no standalone VERSION, compatibility lane or alternate codec added.

## Previously delivered

Core 0.13.2: eff8c18d053506be5e0d3a8676f1e48ae9ce99d0, core-v0.13.2.
Endpoint 0.2.1: a6cfed5b367fff84997985a357ceeb6731ec5400, endpoint-v0.2.1.
[Prior delivery evidence](../evidence/published-core-0132/README.md) and hosted CI
retain that release's qualification; they do not qualify this pending delivery.
#36/#39 are closed and published; #37 is superseded. The
[stable boundary](../stable-identities.md) keeps codec/schema/Types in Core,
execution in Platform and endpoint admission/trapping replies in Endpoint.
