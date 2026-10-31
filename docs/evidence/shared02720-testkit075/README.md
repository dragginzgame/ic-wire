# Shared Tooling 0.27.20 / Testkit 0.75 adoption

2026-10-31. This follow-up preserves the uncommitted Platform 0.15.2 / Types
0.15.1 / Build-Host 0.73.1 work and advances Testkit 0.74.0 to published 0.75.0.
Own versions stay Core 0.13.0 / Endpoint 0.2.0; pending patch notes remain Core
0.13.1 / Endpoint 0.2.1. No staging, commits, releases, publication or sibling
checkout edits were performed.

## Shared snapshot and local callers

The canonical exporter selects clean committed Shared Tooling **0.27.20** at
`267ba25810546bd9cb622d0fb43ccc0588622b67`, independently verified against remote
HEAD. An isolated source clone keeps sibling worktrees untouched.
[Impact](impact.log), [refresh](refresh.log) and [integrity](integrity.log) retain
the 119-payload selection; only five owned payloads change. No owned file was
patched locally. The original preview refused an equivalent-but-differently-
spelled remote URL; the retained source then uses the snapshot's exact canonical
URL before the successful preview/refresh.

The new shared admission checks all versioned root local dependencies against
full current member versions, including unselected groups. Wire already used
those full versions; existing root declarations now carry release-group labels.
No duplicate version catalog, unused dependency or exact-pinning policy is added.
The [local Make and two real release-group fixtures](adapters.log) qualify routing,
publication selection, metadata custody, failures, receipt tampering and resume
in disposable repositories; their expensive validation/uploads are substituted.
These are consumer adapter checks, not real releases or publication.

Shared's new cleanup rule requires normal `cargo clean` to retain its ordinary
whole-workspace scope. Wire needs no cleanup implementation change. No cleanup
was run in this working checkout. Essential retained evidence lives under docs;
transient inputs under target remain removable by explicit cleanup.

[Offline tool admission](tools-check.log) passes. The host/Cargo catalogs and
Build 0.73.1 executable selection are unchanged; no additional installation was
needed for this snapshot. Dependency pins/boundaries and formatting pass.

## Testkit and production graph

[Source comparison](testkit-source-review.json) finds all 28 Testkit src files
unchanged between 0.74 and 0.75. The published supplier now requires Wire 0.13 for
its optional typed helpers. The workspace keeps that feature disabled and retains
Metrics 0.23.2 without native metadata, as required by Testkit's compatible
arithmetic-only dependency range. No new direct Metrics dependency is introduced.

[Installed endpoint test](installed.log) passes with Testkit 0.75 and the same
explicitly prepared PocketIC server recorded in the
[preceding qualification](../platform0152-testkit074/README.md). Limits,
malformed-input refusal, response traps and transaction/rollback oracles pass.
[Benchmark examples](benchmark-msrv.log) compile on Rust 1.91 with wire,candid,ic.

The [normalized deployed Wasm graph](normalized-endpoint-wasm-graph.txt) is equal
to the incoming selection after normalizing local checkout paths. The baseline
was reconstructed with frozen incoming Cargo.toml/Cargo.lock and committed source;
a first comparison could not use an absent historical target directory and was
not accepted as evidence. Core, Types, Endpoint and every production normal/build
supplier remain unchanged by this follow-up. Therefore the prior Wasm/code-size
and Rust/TypeScript/MSRV evidence remain applicable; no fresh runtime cost or
performance benchmark is claimed. The prepared server still lacks the instruction
trace API, so no new instruction/cycle matrix was run.

Protocol unchanged: codec/schema bytes, field order, budgets, malformed acceptance,
retained retry/hash inputs and endpoint semantics. Account absent `01 04 00`
and explicit-zero `01 04 01` plus 32 zero bytes remain distinct. No full local CI
or new hosted/macOS matrix was run.

## Published typed/schema consumer

The [frozen manifest](Cargo.toml.snapshot), [lock](Cargo.lock.snapshot),
[source](consumer.rs.snapshot), [log](published-consumer.log) and
[resolved identities](consumer-identities.json) use only published suppliers:
Wire/Derive 0.13.0, Endpoint 0.2.0, Platform 0.15.2, canonical Types 0.15.1,
Testkit 0.75.0 with wire enabled, and Metrics 0.24.0 with native-wire enabled.
Testkit also selects arithmetic-only Metrics 0.23.2. These are two Metrics versions,
not two Core/schema identities; exactly one registry Wire/Derive/Types resolves.

On Rust 1.91, a real consumer DTO derives Serialize/Deserialize/Wire with Metrics,
Principal and Account fields. Its schema and all four Metrics schemas compile and
execute; Testkit's public Limits/Error values compose with Core and Endpoint
Contract, and query_wire/update_wire generic signatures accept the canonical owned
types. No service starts for this identity test. No Cargo patch, wrapper, local
Wire supplier or source-aligned producer manifest is used. This supersedes the
previous source-only integration proof without claiming downstream application
adoption or executing a network call through the typed helper.

With the locked registry cache and Rust 1.91 prepared:

```sh
bash docs/evidence/shared02720-testkit075/qualify-published.sh /tmp/wire-tooling075-repeat
make shared-tooling-check check-pins check-boundary fmt-check tools-check
make check-tooling check-release
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic make check-endpoint-ic
cargo +1.91.0 check --locked -p ic-wire-benchmarks --examples --features wire,candid,ic
```

The runners keep temporary builds isolated and install nothing. Full retained
working logs and isolated sources are under `target/adoptions/shared-tooling-latest/`;
that directory is not expected to survive explicit Cargo cleanup.
