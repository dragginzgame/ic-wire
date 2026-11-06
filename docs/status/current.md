# Current handoff

## Selected delivery: Core 0.17.4 / Endpoint 0.7.2

The maintainer authorized Git delivery and crates.io publication of this batch
from /home/adam/projects/ic-wire, main, to origin/main at dragginzgame/ic-wire.
Release Core first, then Endpoint, through the maintained group gates and
publication adapter. Expected tags: core-v0.17.4 and endpoint-v0.7.2. Receipts and
remote/registry verification own completion; this selection is not a release claim.

The maintainer accepted the audit actions, prioritizing lint advice over more
codec variants. Default generation now warns about ordinary bool, float,
128-bit integer and zero-sized lists as well as byte/integer/hash candidates.
Unsupported Packed types are never recommended. Exact-path acknowledgements,
stale-acknowledgement detection and the eight-entry display cap remain intact.
Codec-only consumers need a host schema-generation step to receive this advice.
No consumer Serde representation is rewritten.

Endpoint guards now qualify `Result::Ok`, preventing application `Ok` variants
from capturing generated code. Real owned/borrowed macro controls and receiver/
generic diagnostics replace the Core fixture's three mock adapter checks.
Mixed published/current Core, derive and Endpoint consumers run inside the
existing Endpoint stage, on host and Wasm, without downstream Cargo patches.
The qualification-only patches select candidates in disposable test manifests.
Compatible patches do not require republishing every dependent library; public
minor/type-identity changes and payload-layout changes still need consumer review.

Shared frozen fixtures protect representative payload/frame bytes, malformed
primitive refusal and actual host/Wasm allocation thresholds. Account tests
compare raw subaccounts so absent and explicit zero cannot be conflated.
The installed test-only endpoint runs these assertions on Wasm32; default
performance fixtures retain their existing export roster. Serde sequence
encoding visits references, so its charges can differ from owned decoding and
between 32-bit Wasm and 64-bit hosts. Tests freeze those existing rules explicitly.

The private `check-contract` CLI validates and conservatively compares named
schemas, method/payload declarations and optional frozen vectors. It never
updates baselines or adds a runtime fingerprint. The standard benchmark corpus
freezes 391 cases and their ordered-name digest. Fuzzing now covers maps/sets,
recursion, borrowed fields, zero-sized collections, variable limits and output
buffer APIs; normalized collections compare values rather than original bytes.
See [focused commands and selection](../development.md) for usage and limits.

[Matched host timings](../evidence/schema-generation.md#6-november-cost-advice-extension)
record existing controls at +0.2% to +5.9%. The 512-type fixture now retains
2,048 new warnings and takes 1.013 ms versus 0.848 ms. This is added host work,
not a codec or Wasm saving. Standalone schema validation keeps its existing
shared rendering traversal; no second validator or unmeasured sink refactor.

Protocol bytes, schemas, identities, budgets, primitive acceptance and rollback
are unchanged. Example: `[128_u16]` remains `01 80 01`; acknowledging a new
`/response/list/bool` warning changes diagnostics only. No migration or legacy
reader is introduced. Core and Endpoint changelogs have independent patch drafts
for the authorized delivery above.

## Qualification

Focused Core tests/doctests, the six-test 391-case Wire/Candid corpus, real
Core/Endpoint diagnostic fixtures, strict affected Clippy and default Cargo
warning/acknowledgement checks pass. Rust/TypeScript vectors, malformed-input
conformance, Principal/Account host/Wasm and simulated future supplier identity
checks pass. All three mixed-version lanes pass on host and Wasm. Installed
PocketIC admission, replies, synchronous/async rollback and the new Wasm
threshold checks pass. The expanded sanitizer smoke run completes 10,000 inputs
on the separately approved nightly, with leak checking disabled.

Formatting, dependency/Candid boundaries, documentation links, qualification
graphs and disposable Make routing pass. The roster remains 20 full stages and
11 eligible Endpoint-only Linux stages; installed execution and matched costs
remain explicit change-specific checks. No full CI, macOS execution, new full
Candid cost matrix, Toko build, IC-instruction or Wasm-size saving is claimed.
Raw qualification/paired sources live under ignored target/, not Git.

The subsequent Build 0.75/Testkit 0.88 adoption passes locked metadata and
production/Wasm isolation, strict affected native-driver Clippy, both actual
compiler-diagnostic suites, the 391-case corpus and installed Endpoint admission/
rollback checks on PocketIC 16.1.0. No caller migration was needed. Ten unused
transitive packages leave the lock; this is not a production Wasm-size claim.
The Build 0.75 release editor passes offline admission and the disposable Core
metadata/recovery fixture, including interrupted delivery and receipt refusal.
Earlier host timing results retain their original dependency identities.

## Compiler, suppliers and release boundary

Core/derive **0.17.3** is the last delivered Core release at **131f69d**;
Endpoint/macros remain **0.7.1**. Existing delivery receipts own publication
provenance. This audit preserves the pre-existing Platform facade **0.23.2**
lock update and the concurrent **cc 1.11.1** update. The paired host experiment
used the explicitly recorded earlier cc 1.11.0 lock; do not relabel its inputs.
Testkit **0.88.0** and Build **0.75.0** are adopted for native qualification.
The release metadata executable is separately prepared at Build **0.75.0**.
Types **0.18.1**, Runtime/macros/exact guard **0.19.2**, Host **0.74.4** and
Candid/derive **0.10.40** remain selected. Testkit's optional Wire feature stays off.
Rust **1.99.1** is the minimum and pinned compiler. Only the optional sanitizer
fuzzer uses `nightly-2026-11-05`; its script owns that exception.

Shared **0.30.6 / 45c0295** owns 141 exported payloads, independently verified
against the committed source. CBOR source/dependency/browser checks now run
through Wire's existing boundary and materialized-fixture workflows; there are
no CBOR exceptions or owned-format migrations. Candid comparison allowances
remain exact. Shared owns the package inventory, including aliases; lexical
checks do not prove dynamic imports, custom parsers or protocol provenance.
Snapshot integrity, actual boundary targets, generated-browser and materialized
Cargo checks, future-identity qualification, 54 generator/benchmark CLI checks,
and Make routing with forbidden-import refusal pass. Formatting, pins and
documentation links pass; no full CI or codec/performance rerun for this adoption.
Production Core stays SDK-free; optional schemas use Types, Endpoint alone selects Runtime execution,
and Candid/Testkit remain qualification-only. Stage reuse/final-candidate
execution are still disabled. The maintained full Candid matrix uses an older
supplier cohort; it is not a current-lock regression measurement.

## Retained contract and history cautions

The maintainer replaced repository history with one clean root. Consumers
should fresh-clone, not merge discarded refs. Registry archives are unchanged;
old receipt commit IDs are historical provenance, not refs to reconstruct.
Bulk research archives and obsolete Git-side state were retired; compact
[research summaries](../evidence/README.md) cannot verify removed raw artifacts.
Small [release receipts](../../releases/receipts/README.md) retain their own role.

Production retains explicit method IDs and name defaults. Fingerprints and
no-manual-ID macros remain research. The [0.16 Account cut](../../releases/core/0.16.md)
changed present subaccounts `01 + 32 bytes` to `01 20 + 32 bytes`; None stays `00`.
Consumers own retained-data adoption and collection/custom-Serde normalization.
Retain original retry bytes and encoder hash inputs; keep one codec and no
automatic migration. Ordinary-list hotspots remain; use measured field choices
and limits rather than per-type codec variants or projected stress-fixture gains.
