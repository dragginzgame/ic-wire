# Independent Runtime adoption: source qualification

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

This prepares [Wire PR #41](https://github.com/dragginzgame/ic-wire/pull/41), the
Wire side of [Platform #131](https://github.com/dragginzgame/ic-platform/issues/131).
The integration is implemented in a disposable source tree, **not selected by the
main checkout**. Runtime and its matching macros/exact guard are unpublished as
of the Retired raw output (see retention notice). Keep the registry lock and published
Endpoint 0.3 supplier until those packages are available. No release, publication,
remote PR change or sibling checkout edit is performed by this batch.

## Prepared change

Decompress Retired raw output (see retention notice) before applying it to the source
revision recorded below; it applies without
changing the pending Core 0.13.5 output-writer optimization. It extends PR head
`e21017e2212c4c06a19e059e6cbe116162e02ec9` with current owner/migration documentation.
The patch selects `ic-platform-runtime` directly for Endpoint and its canister
fixture, makes the facade optional for native qualification, and updates graph
checks to distinguish runtime, schema and native client owners. The future-facade
fixture retains Core, derive, Types, Runtime, macros and guard package identities
while changing the facade and Endpoint package identities.

This is a pending **Endpoint 0.4** ownership/API cut: direct low-level imports
move from `ic_platform::{cdk, executor, ic0}` to
`ic_platform_runtime::{cdk, executor, ic0}`. Align Timers and other executor
suppliers first. A consumer combining the new Endpoint with the legacy embedded
Platform 0.16 executor is intentionally rejected by Cargo's singleton guard.
Endpoint handler syntax, codec bytes, budgets, malformed-input acceptance,
transaction/rollback semantics and retained retry/hash inputs are unchanged.
The metadata derive and schema implementations are untouched.

## Qualification inputs and limits

Wire baseline is `e6c05564241e3a07775c72ac03a825007293f97b` plus the pending
0.13.5 writer hint and sampler identity, held equal in both lanes. Platform source
is frozen at `3a72f6fe6a83e6896cb23f3c5e551f1bb89b6116`; no moving sibling path
is used by the final qualification. Prepared package versions are facade/native
client 0.16.0, Runtime/macros/exact guard 0.17.0 and Types 0.15.1. Other resolved
registry dependencies match, including Testkit 0.76.1, Build/Host 0.73.5 and
Postcard 1.1.3. Retired raw output (see retention notice) records the supplier
extraction, including replacement of the native external PocketIC client.

The normalized dependency manifests have semantically equal `Cargo.toml.orig`
inputs to the frozen source. Their implementations are replaced with that exact
frozen source, including the final guard documentation. Root-only qualification
patches are retained explicitly; no such patch belongs in a downstream manifest.
Retired raw output (see retention notice) records input file hashes.

Platform's announced delivery also includes Types 0.16. **This experiment holds
Types 0.15.1 fixed** to test the execution boundary. It is not qualification of
that future registry graph. If the supplying Runtime selects Types 0.16, align
Core's schema supplier and classify that separate Rust identity change before
integrating. Do not introduce a second canonical Types identity or a shim.

Raw patches and logs containing significant whitespace are stored byte-for-byte
in gzip archives; decompression preserves the original evidence.

## Focused checks

- Endpoint: three schema/borrowing/contract tests, method identity vector test,
  strict library/fixture Clippy, eleven expected diagnostic subjects plus their
  passing control, optimized Wasm and Rust 1.91 Wasm compilation pass.
- Standalone schema-only and combined Endpoint consumers: Rust 1.91 host/Wasm,
  strict Clippy, four/five Rust tests and Rust–TypeScript Principal/Account exact
  bytes and malformed-input refusals pass. Absent/zero/nonzero subaccounts remain
  distinct. Schema-only graphs exclude execution; Wasm excludes host schema JSON.
- Future-facade identity simulation passes. One Runtime/macros/exact guard and
  one Core/derive/Types identity remain; production Endpoint excludes the facade,
  Candid, PocketIC and external executors. The legacy-executor conflict is rejected
  specifically for the duplicate `links` owner, rather than a missing package.
- Installed `typed_calls_limits_and_transactions` passes with explicit prepared
  PocketIC 16.0.0, including admission, exact replies, refusal, rollback, async
  and framework dispatch. Its server SHA-256 is
  `69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.
- Integration patch application, Rust formatting and changed shell syntax pass.

These are Linux source checks using normalized supplier manifests. No registry-only, macOS or full release
CI result is claimed. Initial raw workspace-inheritance failures are retained
under `target/runtime-017-adoption`; final standalone checks use normalized,
self-contained suppliers rather than inheriting from a different Cargo workspace.

## Matched endpoint cost comparison

Both profiles pass **178 cases across all 17 exports**: 5,696 measured calls
across the two lanes, plus 712 excluded warmup calls. All 356 case/profile pairs
have identical instruction and observed cycle-debit means. In fact, **all 3,204
paired samples, including warmups, have identical instruction and cycle counts**.
Caller, exact request/reply hashes, frame sizes and outcomes also match. Maximum
instruction regression is 0%; no case exceeds the 5% comparison screen.

| Profile | Total Wasm before → after | Code section before → after |
| --- | ---: | ---: |
| opt-z | 100,373 → 100,285 (−88; −0.0877%) | 86,476 → 86,476 (0%) |
| opt-3 | 93,957 → 93,869 (−88; −0.0937%) | 81,012 → 81,012 (0%) |

These are whole **uncompressed** modules. The code section does not shrink;
this is evidence of no added execution/code overhead in this fixture, rather
than a new codec optimization claim. Both consumer-owned profiles are retained.

Representative opt-z instruction controls (eight equal samples on each side):

| Path | Instructions before → after | Observed cycles debited on each side |
| --- | ---: | ---: |
| Default unit query | 248,988 → 248,988 | 0 |
| Owned 32-byte blob query | 256,236 → 256,236 | 0 |
| 65,522-byte blob query | 3,884,447 → 3,884,447 | 0 |
| Update with 32-byte reply | 492,582 → 492,582 | 6,984,582 |

Retired raw output (see retention notice), Retired raw output (see retention notice)
and Retired raw output (see retention notice) retain controls,
all repetitions, raw physical traces, refusals and source/configuration metadata.
The [maintained measurement semantics](endpoint-matrix.md)
include full physical guest executions, callbacks and page charges; cycles are
observed balance debits, not instruction estimates. The instrumented 16.1.0 server
is frozen by Retired raw output (see retention notice). Native timing is not measured: this change moves
execution ownership and holds codec code fixed. No gzip metric is used.
