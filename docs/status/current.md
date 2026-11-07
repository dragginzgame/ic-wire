# Current handoff

## Completed: full comparison refresh

The full **36-cohort** Wire/Candid run measured Core **0.17.6** working source in
`target/evidence/codec-matrix-20261107/`, with frozen source/lock and separate
`target/codec-matrix-build-20261107/`. It uses PocketIC **16.1.0**, two native
runs of eleven samples, allocation/heap probes and opt-z/opt-3 Wasm. All **2,105
case entries / 8,420 primary comparisons**, **144 fresh modules**, repeated IC
counters and input/frame identities verify. Wire wins **7,295/7,924** non-research
comparisons and is smaller in both Wasm metrics in all **72** fixture/profile pairs.
The [current comparison](../candid-comparison.md) owns results, limitations and
ranked experiments. No production code change or delivery is part of this refresh.

A separate cleanup interrupted `bulk-control` during opt-3 guest compilation
with 14 cohorts complete. Frozen sources and retained measurements verify;
the restored prepared PocketIC executable has the identical hash. Recovery lives
in the run's `recovery-1/`: failed build, resume script, verifier and progress.
Completed probes were retained; only missing guests and subsequent cohorts reran.
The maintainer stopped cleanup and explicitly protected both matrix paths.
Only the separately approved ordinary `target/debug` cache was removed by this
session. Both protected matrix paths remain intact; no further cleanup is authorized.

## Compiler, suppliers and release boundary

Pending Core **0.17.7** and Endpoint **0.7.3** select published **Build 0.77.0**
(`9d87394`), **Platform 0.23.6** (`51f5fbb`), **Runtime/macros/guard 0.19.5**
(`c07b714`) and **Testkit 0.90.0** (`c03161b`). Types remains **0.18.2**, Host
**0.74.4** and smallvec **1.16.5**. Rust/Cargo **1.99.1** remains floor and pin.
Root requirements and Cargo.lock own the selected graph. Build's removed timing
getters have no Wire callers; the release editor uses the same 0.77.0 with its
installation-only `cli` feature. Prepare through `make install-tools`.

All seven native PocketIC constructors now pass a **300-second request budget**,
matching Testkit's former default. The new finite deadline covers the whole
exchange, including body reads and accepted-operation polling; startup retains
its independent 60/90-second allowance. Typed failure propagation, caller-owned
Tokio runtimes and explicit awaited teardown are preserved. No replay or fallback
API was added. This addresses [#48](https://github.com/dragginzgame/ic-wire/issues/48).
Testkit stays outside published packages. Codec bytes, identities, limits and
acceptance are unchanged; no codec performance claim follows from this migration.

Focused caller Clippy, installed Endpoint admission/transaction checks, all seven
expected compiler diagnostics and the passing control, offline editor admission,
pins/boundaries and local documentation links pass. Both groups' all-feature
archives also pass before version preparation. Full captured delivery gates are
still required; no release/publication has occurred for this batch. Logs are in
`target/testkit090-callers.log`, `target/platform0195-installed.log`,
`target/build077-diagnostics.log`, `target/suppliers-20261107-static.log` and the
`target/{core0177,endpoint073}-archives-preflight.log` files.
The retained complete comparison predates this adoption: **Build 0.76.0**,
**Platform 0.23.5**, **Testkit 0.89.4**, **Runtime 0.19.4**, Types **0.18.2**,
Host **0.74.4**, Candid **0.10.40** and Postcard **1.1.3**. Its results must not
be relabeled as measurements of the new supplier cohort. Linux qualification
does not establish native macOS execution.

## Delivered state and recovery

Core **0.17.6** is on origin/main and crates.io (`ic-wire-derive`, `ic-wire`),
tag **core-v0.17.6**, release commit **163d6aa**. Endpoint/macros remain **0.7.2**.
The [receipt](../../releases/receipts/published-core0176/delivery.json) owns exact
source/archive/graph identities. The complete 20-stage Linux gate, all-feature
archives and installed Endpoint admission/transaction checks passed. Registry
readback matched the uploaded archives. Normal main was reconciled to delivery.

Shared **0.31.0 / 7431394** captured **245ed7a** in schema-2 **batch-1i9z2IIG**.
The release retained Platform **0.23.4**, Types **0.18.2**, Runtime **0.19.4**,
Testkit **0.89.3**, Build **0.75.0**, Host **0.74.4**, Rust **1.99.1**. The later
Platform lock edit was excluded, preserved at **7fd813f** on
`deferred/core0176-lock`, and reapplied after reconciliation. Subsequent working
selections are above; none is retroactively qualified by this delivery.

The failed **batch-1afAufRB** stopped before version preparation because browser
setup was missing; it had no delivery effects and is superseded. Do not resume
it. Corrected capture installs locked browser tools and checks them offline on
recovery. Both journals remain under `.git/release-state/frozen/`. Raw delivery
logs remain in `target/core0176-delivery/`; the separate cleanup removed the
completed `target/release-frozen/batch-1i9z2IIG/` checkout and adoption scratch.
Do not treat journal presence as proof that its captured checkout still exists.
The completed schema-1 Core 0.17.5 delivery is historical; do not resume it with
the schema-2 runner or replay publication. See its
[receipt](../../releases/receipts/published-core0175/delivery.json).

## Current contracts and follow-up

Public `serde_packed::nonzero_i64` / `heap_u64` shipped in 0.17.6. They remove
encode copies while preserving domain validation/reconstruction; already-packed
DTOs remain smaller in the 1/10/100-type stress controls. See the
[matched adapter study](../candid-comparison.md#borrowed-adapter-investigation)
and [consumer guidance](../../crates/ic-wire/README.md).
Adopting from ordinary lists changes `[-1_i64]` from `01 01` to
`08 ff ff ff ff ff ff ff ff` (heap `[1]`: `01 01` to
`08 01 00 00 00 00 00 00 00`). Coordinate contracts/clients and requalify budgets;
record-field allowances can differ from Packed DTOs. Existing consumers stay
unchanged. Retain original retry/hash bytes: heap/custom-Serde normalization is
consumer-owned. Native big-endian execution remains unqualified.

Account's [0.16 cut](../../releases/core/0.16.md) remains `01 20 + 32 bytes` for
present subaccounts and `00` for absent ones. Method IDs/name defaults remain
production behavior; contract fingerprints/no-manual-ID macros remain research.
Keep one Serde/Postcard codec; Candid and Testkit stay outside published packages.
Default bulk advice requires supported host metadata; codec-only/custom layouts
are not universally linted. Never silently rewrite consumer Serde.

Next candidates are checked-copy bool decoding/public adapter qualification,
narrow bound-wrapper sharing and packed 16-byte ID research. None is implemented
by this refresh. Bool research cuts large encode instructions and fixture Wasm,
but decode reconstruction remains expensive. Domain adapters beat Candid while
costing more than already-packed DTOs in scale controls. Do not relabel these
stress-fixture differences as application savings. Generic
one-pass encoder rewrites, signed-i32 writer sharing and parser-tail sharing have
prior rejected tradeoffs; consult their existing studies before repeating them.

Future authorized delivery uses [Shared's captured groups](../development.md):
Core's 20 stages, qualified Endpoint-only Linux's 11, no stage reuse/final-candidate
opt-ins. Combined delivery still needs supplier-first archive qualification.
Older repository history was intentionally replaced: fresh-clone rather than
merge discarded refs; never re-upload existing registry versions to repair history.
