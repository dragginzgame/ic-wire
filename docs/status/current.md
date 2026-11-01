# Current handoff

## Core 0.13.4 deployment in progress

The maintainer authorized Git delivery to origin/main and crates.io publication
of the pending Core patch, followed by further isolated optimization research.
Release only Core 0.13.4 (ic-wire/derive); Endpoint remains 0.3.0. The standard
runner must complete its full gate before version preparation. Source and
registry readback agree on Core 0.13.3 as the released base; 0.13.4 is absent.
Keep new optimization changes outside this admitted release source.

Align Build's minimum and the managed release-editor pin on published 0.73.5.
The incoming lock already selects Build and Host Artifacts/FS/Process 0.73.5
and is preserved. Host Rust sources match 0.73.4; Build changes only package
archive qualification, and the editor's four release source files match 0.73.3.
Explicit editor preparation, offline admission, nine diagnostics/passing control,
qualification-caller Clippy/Rust 1.91 and production graph isolation pass.
The disposable Make fixture and both real release-group adapter fixtures pass
with the selected editor, including refusal, interrupted resume and receipt checks.
Production codec/generator source, bytes and budgets are unchanged; no runtime
performance claim or real release is made. [Adoption evidence](../evidence/build-host-0735-adoption/README.md).

Adopt published Testkit 0.76.0 with defaults disabled for the two private
qualification packages. Its native Platform supplier now shares the existing
0.16.0 identity; the lock removes 0.15.3, with all other package versions
preserved. The 28 Testkit Rust source files are byte-identical to 0.75.2.
Focused caller Clippy, Rust 1.91 host compilation, Core/endpoint Wasm graph
isolation and the prepared installed endpoint admission/rollback test pass.
Own versions remain Core 0.13.3 / Endpoint 0.3.0. Production codec/generator
source, bytes, budgets and retries are unchanged by this dependency adoption;
no performance saving is claimed. [Adoption evidence](../evidence/testkit-076-adoption/README.md).
Earlier cost studies below freeze their recorded 0.75.2 graph; this is not a
full performance matrix rerun or a cross-lock regression comparison.

Shared 0.27.24 at 1211262f32ea193cc4edcd60d2750baf1f1d6425 is canonically
exported (119 payloads). Raw total module bytes and code-section bytes are
separate required Wasm metrics; gzip/transport bytes are not an optimization
objective. Consumer profiles/optimizer settings remain unchanged.

New private cost-smoke corpus has 20 controls: small scalars/blobs, 1 MiB blob,
empty/singleton/4096-unit lists, empty records/inner lists, optional units, an
ordinary i16 control, a record and four binary-key maps. Roundtrip/refusal tests
and strict corpus Clippy pass. The data-only cost gate checks paired identities,
exact frame/input coverage, three stable unique-caller samples, unchanged Candid
controls and physical raw/code Wasm. It permits no Wasm growth and IC costs at
most baseline × 1.05 + 64. Eighteen corruption/regression fixture checks and all
192 real Wire pairs pass. No automatic baseline update or server startup.

Keep the simpler `Budget::enter_many` direct allowance checks, preserving sticky
refusal and debit order. The 20-case fixture saves 26 total/code bytes at opt-z
and 14 total/code at opt-3. The 100-type fixture saves 69 total/code at opt-z and
127 total / 125 code at opt-3. Maximum IC regression: +0.4121% (map128 encode);
optional-unit decode improves 6.38%. Native unit encoding improves about 85%,
but unit decoding grows about 0.2 µs (29–31%); allocation/heap rows are unchanged.
These are complete fixture measurements, not application headroom claims.

Byte serialization outlining worsened opt-3 size and doubled map encode cost;
unit inlining had no effect; a more aggressive budget rewrite regressed opt-z
unit decoding 12.38%. Reject those and the zero-charge no-size-win candidate.
The opt-z large unit-list encode gap remains about 9.48× Candid. The original
copied-workspace attempt was cache-contaminated and is INVALID. Corrected runs
use one worktree with observed library/guest rebuilds. Exact source, independent
patches, all artifacts/samples and replay commands are in the
[cost investigation](../evidence/cost-experiments-0134/README.md).

Follow-up [parser size screen](../evidence/parser-screen-0134/README.md) rejects
three additional candidates against that accepted cleanup: byte-budget direct
subtraction is bit-identical, shared buffer/string readers add 121 total / 119
code bytes at opt-z (94/92 at opt-3), and outlined varint tails add 230/227 at
opt-z (1,062/1,058 at opt-3). All eight 100-type modules were rebuilt and retained.
No instruction/timing work was run for candidates that failed the size objective.
The private sampler now records parser source identity too; strict sampler lint
passes. These rejected patches add no production runtime changes.

Focused qualification passes: 82 runtime/native/borrowed tests, runtime/corpus
Clippy, Rust 1.91 host/Wasm, installed endpoint rollback/admission, 49 CLI checks
and 18 cheap gate fixtures. Source/fixture/compiler/build identities are stronger.
No new full matrix/CI run is claimed. Protocol bytes, acceptance, budgets, saved
retry/hash inputs and the one production Serde/Postcard codec are unchanged.

The batch below is admitted for this deployment. Own package versions remain
Core 0.13.3 / Endpoint 0.3.0 until the standard runner prepares Core 0.13.4 after
validation. The delivery below records the previous completed release.

Read-only GitHub inspection on 2026-11-01 finds one open issue: #40,
documentation-only CI selection. It is a consumer workflow optimization requiring
a reviewed prose allowlist, stable required checks and full native fallback;
it has not been implemented or closed by this codec investigation.

## Delivered Core 0.13.3 / Endpoint 0.3.0

Published and verified on 2026-11-01. Core commit
33306395a833fd29f8c8a91abf9b8d38123dd0f9 / core-v0.13.3 and Endpoint commit
3ddaeb834e0f8e3290f3146a119435a185b9bda9 / endpoint-v0.3.0 were atomically pushed.
Complete local release CI passed in 579 s / 617 s. All three registry archives
match upload bytes, checksums and clean release provenance. A registry-only
Rust 1.91 consumer passes six tests and Wasm compilation with one Core/derive/
Endpoint/Types identity; no Candid/external executor leaks into its Wasm graph.
[Delivery evidence and reproduction](../evidence/published-core-0133-endpoint-030/README.md).
No build/evidence cleanup or sibling edit occurred.

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

## Earlier delivered

Core 0.13.2: eff8c18d053506be5e0d3a8676f1e48ae9ce99d0, core-v0.13.2.
Endpoint 0.2.1: a6cfed5b367fff84997985a357ceeb6731ec5400, endpoint-v0.2.1.
[Prior delivery evidence](../evidence/published-core-0132/README.md) and hosted CI
retain that release's qualification; they do not qualify this pending delivery.
#36/#39 are closed and published; #37 is superseded. The
[stable boundary](../stable-identities.md) keeps codec/schema/Types in Core,
execution in Platform and endpoint admission/trapping replies in Endpoint.
