# Current handoff

## Released 0.7.23; pending 0.7.24 endpoints and upstream adoption

HEAD is `e90aed4f7dc0cd7ee31a9bcfb211044220ab0cec` / release 0.7.23.
Package versions remain 0.7.23; 0.7.24 is a changelog draft only. Leave staging,
commits, version changes, publication and releases to the maintainer. No hosted
workflow, publication or deployment was performed. No Canic/Platform source was
edited. Concurrent borrowed-input work below was preserved.

Current Platform minimum/lock is 0.8.3. Build/Testkit and the managed release editor
select 0.70.3, with Host Artifacts/FS/Process locked to the same family. Rust 1.91
remains the floor. The latest adoption preserves its incoming lockfile exactly;
it already contained these selections. Local Tooling 0.70.4 is an unpublished
draft and was not adopted. No package versions or sibling sources were changed.

Shared Tooling is 0.27.9 at `3c458c28cca132784e6e70fbaf219bdea1e4cf8f`:
reviewed impact preview, canonical refresh, 112 verified files including the new
validation-input checkpoint companion. Host/Cargo/IC executable catalog pins are
unchanged. No snapshot-owned file was hand-patched. Current adoption logs and its
incoming Make/manifest/lock/snapshot receipts live in `/tmp/wire-adoption-20261030/`.

Earlier endpoint and borrowing evidence below retains Platform 0.8.2 and Tooling
0.70.2 in its frozen graphs; it is not silently relabelled as new-version evidence.
The original endpoint-task lock remains at `target/wire-endpoint/incoming.Cargo.lock`.

## Current adoption qualification

Snapshot integrity, dependency pins/boundaries, all selected tool checks and the
0.70.3 editor's offline admission pass. Focused endpoint native/schema/diagnostic,
strict Clippy, Wasm and Rust 1.91 gates pass; standard PocketIC validates installed
sync/async/composite/borrowed/guarded calls and rollback. Disposable Make/hook and
release/resume fixtures pass without real-checkout release effects.

The [matched adoption matrix](../evidence/upstream-adoption-0724/README.md) repeats
all 17 endpoints / 110 cases under both profiles: 1,760 measured calls and 1,936
physical executions pass. Every instruction, page-charge and cycle-debit sample
matches the old graph exactly (0% change for nonzero values). Request/reply hashes
match too. Opt-z total Wasm grows 103,638 → 103,642 bytes (+4; 0.0039%); its code
section stays 89,112 bytes. Opt-3 stays 98,503 total / 84,974 code bytes. No protocol
change or performance saving is claimed. Raw candidate evidence:
`/tmp/wire-adoption-endpoint-matrix-20261030/`. Full CI, macOS and the broad Candid
matrix were not rerun; frozen historical studies retain their original scope.

## Typed endpoint integration

The optional `endpoint` feature supplies `ic_wire::endpoint::{query, update}`
and pre-copy admission / bounded decode / trap-on-encode helpers. Platform owns
exports, protected spawn and query/update executor contexts. The default codec
stays independent of Platform; the endpoint Wasm graph has no Canic, Candid or
PocketIC dependency. The private standalone consumer lives in `examples/endpoint`.
See the [API and ownership audit](../endpoints.md).

Bare attributes select stable exported-name IDs and per-direction defaults:
64 KiB frames, 8 MiB traversal, 65,536 nodes and depth 32. Overrides remain available.
Handlers support
one owned request or framed unit, sync/async queries, composite queries and updates.
The explicit `borrow` option supports one synchronous borrowed request/reply.
Optional `contract = NAME` binds schema projection and constants to the actual
handler signature. Existing native schema projection is reused; no registry or
Wasm metadata section is added. Canic keeps authentication, Fleet admission,
startup/dispatch, role bundles, metadata sections and final artifact admission.
Its later adoption replaces only generic Wire I/O and Wire-branch export glue.

Explicit declarations retain their protocol bytes, IDs, limits and acceptance.
For ID 7 / u8 42, the request remains
`49 57 07 00 00 00 00 00 00 00 00 2a`; reply direction is 01 at offset 10.
Omitting the ID for `echo` now selects `native::method_id("echo")` =
3346031142226386015, yielding `49 57 5f 5c cc 87 98 7d 6f 2e 00 2a`.
Removing existing overrides requires coordinated clients/schema regeneration.
No legacy lanes or automatic migration. Preserve exact retry/hash bytes.
Synchronous reply failure traps and rolls back mutation. After a real async await,
only the current callback rolls back; prior committed mutations remain.

Focused native/schema tests, eight compile-fail cases plus valid control, strict
core/derive/fixture Clippy, Wasm build, Rust 1.91 Wasm check and dependency graph
isolation pass. Installed PocketIC cases pass typed/unit calls, malformed and
wrong-identity frames, request/reply limits, sync/ready-async rollback, actual
async/composite self-calls and one-dispatch guarded composition. Raw logs are in
`target/wire-endpoint/`; diagnostics in `target/endpoint-check.CH8ejI/`.
`make check-endpoint` joins the core roster; `make check-endpoint-ic` explicitly
requires a prepared PocketIC server. Full CI/macOS were not run.
Documentation/dependency boundary checks and the disposable Make/formatting
adapter fixture also pass; no real-checkout release effects were invoked.

The defaults follow-up qualifies fixed ASCII/UTF-8 ID vectors, default-sized and
one-byte-over frames, independent overrides and reply-failure rollback. Latest
logs are in `target/endpoint-defaults/` (11 diagnostic cases plus valid control).
The [host expansion study](../evidence/endpoint-defaults/README.md) retains repeated
before/after/default samples and its concurrent-change limitations. ID derivation
and default selection are compile-time constants, not message work.

The [matched hook study](../evidence/endpoints/README.md) compares equivalent
inline Platform hooks with extracted hooks, not a complete Canic application.
Eleven samples per input/profile preserve unit/empty/32-byte/64-KiB controls.
Opt-z sampled IC cost increases 0.027–0.083%; total Wasm grows 608 bytes (1.91%)
and code section 492 bytes (1.75%). Opt-3 costs/sizes match. Native timings and
all regressions are recorded separately; concurrent host builds prevent speedup
claims. Host macro-generation time was not isolated. Protocol unchanged.

## Retained qualification boundaries

Released Tooling adoption evidence remains under `target/tooling-adoption-0723/`,
`target/platform-adoption-0722/` and the recorded `target/ic-adoption-*` studies.
Those results retain their own dependency graphs. Earlier hosted 0.7.21 evidence
is historical, not qualification of this working tree. No full codec matrix or
native macOS result is claimed for this batch.

The [SDK recipe](../sdk-integration.md) and
[#29 acceptance](https://github.com/dragginzgame/ic-wire/issues/29#issuecomment-6475585612)
retain consumer-owned auth, transport, retries, persistence and deployment.
Registry 0.7.21 payload/schema/TypeScript qualification remains in
`target/issue29-0721/`; it was not installed endpoint qualification.
Hosted core/TypeScript/benchmark groups retain their supported host matrix and
final scaffold gates; this batch adds the focused endpoint compile gate.

The [scoped-output study](../evidence/with-encoded-0721/README.md),
[schema-validator study](../evidence/schema-validator-0721/README.md),
[warning study](../evidence/generator-warnings-0721/README.md),
[caller-storage](../evidence/caller-storage-0719/README.md),
[runtime](../evidence/runtime-0716/README.md) and
[encoder](../evidence/encoder-followup-0716/README.md) preserve their frozen inputs.
Platform Candid comparisons use tree-depth-7 under the 16-level cap; historical
depth-8 rows cannot be paired. Depth 8 remains Wire-success/typed-Candid-refusal.
Installed SDK/framework adoption remains consumer-owned.

## Borrowed-input qualification

The [borrowed-input study](../evidence/input-borrowing-0724/README.md) isolates
released 0.7.23 from concurrent endpoint work. All 384 physical updates pass;
1 MiB prefix/rejection costs fall about 41%, full scans 15–21%, total Wasm
226–365 bytes. Both existing profiles retain small-input controls, exact bytes,
full validation and recorded page charges. No runtime/protocol change or installed
SDK qualification is claimed by that initial raw-export study.

The subsequent [generated-endpoint study](../evidence/borrowed-endpoints-0724/README.md)
qualifies `borrow` through actual Platform exports: another 384 physical updates
pass. At 1 MiB, prefix/business rejection instructions fall 46.8%, full scans
15.4–22.8%; total Wasm falls 344 bytes (opt-z) / 201 bytes (opt-3), code falls
342 / 201 bytes. Empty opt-3 calls increase 66 instructions (0.014%); all other
measured cohorts improve. Existing owned Wasm is byte-identical before/after in
both profiles. No protocol change, lazy validation or async borrowing.

Focused endpoint checks pass native custody/schema tests, 11 compile-fail cases
plus valid borrowed controls, Clippy, Rust 1.91 and Wasm/dependency checks.
Installed checks pass borrowed reply encoding, pre-handler malformed/trailing/
wrong-ID/oversized refusal, reply-failure rollback and existing sync/async cases.
Logs: `/tmp/wire-endpoint-borrow-{check,installed}.log`; diagnostics:
`target/endpoint-check.hPZNMZ/`. Evidence receipts retain frozen source and raw
traces. Full CI, macOS and actual Canic adoption were not run; no push/release.
Shared staging/releases remain untouched.


## Complete endpoint cost matrix

`make evaluate-endpoints EVIDENCE_DIR=/absolute/new/path` now freezes source and
measures every compiled fixture export with a prepared instrumented PocketIC.
The [matrix](../evidence/endpoint-matrix-0724/README.md) covers all 17 endpoints and
110 cases under both existing profiles: 1,760 measured logical calls / 1,936
physical executions pass, plus 220 excluded warmups. Success/refusal and state
oracles cover owned/borrowed, ready/suspending async, composite self-calls, unit,
defaults/overrides, guarded dispatch, malformed input, node limits and reply traps.
Each export has a valid control plus wrong-ID/trailing/oversized/truncated cases.
Coverage is checked against the actual Wasm exports; new unbenchmarked exports
fail the run. Full physical IC instructions, page charges, observed cycle debits,
frame sizes and whole-artifact uncompressed/code sizes remain separate metrics.

Query result caching is excluded with distinct callers and pre-sample simulator
round publication; missing physical records fail, rather than imply zero cost.
Query debit is zero on this simulator, not zero instruction work. Async costs
include self-call/callback/cleanup; state observation stays outside the interval.
This is the first complete matrix baseline, not a runtime optimization claim.
The runtime, endpoint generator, protocol and consumer code are unchanged by this
batch. The only lock change adds existing sha2 to private fixture host dev deps.

Focused host Clippy, script syntax, full installed matrix, raw receipt auditing
and cross-profile call-oracle/source identity checks pass. Full CI/macOS were not
run. Raw immutable run: `/tmp/wire-endpoint-matrix-final-20261030/`; failed harness
query-trace attempts are retained separately and contribute no accepted samples.
All versions/staging/commits/releases remain with the maintainer.
