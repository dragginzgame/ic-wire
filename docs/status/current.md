# Current handoff

## Release-group adapter contribution

The isolated Shared #328 contribution makes the local release policy consume
an explicitly selected group, its ledger/file inventory and distinct receipts.
No production group, package, version or vendored payload is changed. Group use
requires a reviewed Shared snapshot and a published supplying IC Build editor.
The real adapter passes coordinated and endpoint-group preparation/delivery/
resume fixtures with candidate Shared/editor inputs in a disposable checkout;
the expensive CI/tool setup gates are substitutes. ShellCheck passes. These
checks are Linux evidence, not native macOS or registry delivery. Shared owns
[the mechanism](https://github.com/dragginzgame/shared-tooling/issues/328);
[#36](https://github.com/dragginzgame/ic-wire/issues/36) retains boundary ownership.

## Released 0.10.0; pending 0.11.0

Released baseline is `c9d4e7b`. PR [#35](https://github.com/dragginzgame/ic-wire/pull/35)
merged as `2a6d186` and is fast-forwarded locally. Both optional integrations now
select published Platform 0.12.0, including its macros/runtime guard. Pending
Tooling 0.72.1 adoption and Tripwire research were preserved. Own package versions
remain 0.10.0; leave staging, commits, versioning, releases and publication to the
maintainer. Shared Tooling remains the reviewed 0.27.11 snapshot at
`62edfbd43087a718bf0283e9040676a49211976d`.

The supplying Principal/Account Rust identities change, so notes target Wire's
next minor, 0.11.0. Consumers using `platform-types` align their direct Platform
dependency to 0.12; endpoint consumers align all runtime suppliers. Native typed
clients exposing Wire Limits/Error also need the supplying Wire minor.
[#34](https://github.com/dragginzgame/ic-wire/issues/34) remains open for publication;
[Tooling #274](https://github.com/dragginzgame/ic-tooling/issues/274) tracks the native
client pairing. Core-only users need no migration. No codec bytes, budgets,
accepted-input rules, handler syntax, identities or retry/hash inputs change.

## Issue follow-up

[#31](https://github.com/dragginzgame/ic-wire/issues/31) is fixed locally: the five
benchmark/report/generator CLIs use the existing workspace Clap dependency.
Typed native paths, profile/primary choices, ranges/defaults, exclusive report
modes and Cargo's `--bench` are declared before effects. Measurement code and
create-only evidence remain unchanged. Clap is host-only in the unpublished
benchmark package. No codec/protocol change or performance saving is claimed.

`make check-benchmark-cli` passes 49 focused checks, including a real Cargo bench
caller stopped at existing output, report reuse refusal and the disposable
generator probe. Five report tests, focused strict Clippy and Rust 1.91 checks
pass. The Wasm benchmark graph excludes Clap. No full performance matrix was run
for CLI mechanics. Logs: `/tmp/wire-issues-31-34/`. Leave #31 open until delivery.

CI follow-up: the CLI batch omitted `ic-wire-benchmarks` from the consumer-owned
Clap dependency allowlist. Add that package to `ci/cargo-dependency-boundaries.json`;
keep the shared checker unchanged and Clap host-only/dev-only. This is policy
alignment, with no runtime or protocol change. `make check-boundary` passes;
the Wasm benchmark graph excludes Clap. Documentation links and diff checks pass.
No full CI or codec benchmark was rerun for this metadata fix.

[#32](https://github.com/dragginzgame/ic-wire/issues/32) is closed after qualifying
registry-only Wire 0.10.0 / Platform 0.11.1 together: five combined endpoint/schema
tests, Rust 1.91 Wasm compilation and feature isolation pass without patches.
See [published consumer evidence](../evidence/platform-types-transfer/published/README.md#published-wire-consumer-follow-up).
#34's source fix is merged; it remains open until compatible Wire publication
and Blob consumer qualification. Nothing was committed, released or published.

## Qualification

[Platform 0.12 evidence and reproduction](../evidence/platform012-adoption/README.md)
compare the pending Platform 0.11.1 / Tooling 0.72.1 baseline with Platform 0.12.0
on the same Tooling. Registry Platform/macros/guard source trees are identical.
Standalone Principal/Account derives pass Rust 1.91 host/Wasm, strict Clippy,
schema-only and combined endpoint graphs, exact Rust/TypeScript frames and
malformed-input tests. Endpoint schema/method identity tests, 11 expected compiler
diagnostics plus passing control, lint and optimized/MSRV Wasm checks pass.
Installed limits/rollback and the three Candid corpus correctness tests pass.

The matched 12-case, seven-export probe passes: 384 measured logical calls / 608
physical executions, eight samples after warmup in each profile. Every instruction
and cycle sample matches, as do frames and outcomes. Code sections remain 86,068 B
(opt-z) and 80,630 B (opt-3). Total uncompressed Wasm is 99,951 → 99,947 B and
93,518 → 93,518 B. The four-byte non-code change is not a codec optimization claim.
A profiler rebuild was detected during the first pass; the final receipt uses
one frozen binary and rerun affected samples. Raw work: `/tmp/wire-platform012-adoption/`.

Testkit 0.72.1 retains native-only Platform 0.11.1 in the development graph.
Both Wire integrations use the single root Platform 0.12 requirement. Core stays
Platform-free; schema-only selects no CDK/executor/Candid; Wasm selects no host
serde_json; production endpoints have one runtime owner. No package patch is
needed. Pin/boundary and documentation checks pass. No full CI, macOS or new
full Candid performance matrix was run. The managed 0.72.1 release editor and
disposable release adapter retain their [prior qualification](../evidence/tripwire-followup-0101/README.md).

## PR disposition

PR [#33](https://github.com/dragginzgame/ic-wire/pull/33) was closed as superseded,
with explicit maintainer approval. Its goal of removing `#[path]` imports shipped
in 0.8.0 (`59dd019`) through colocated drivers and ordinary modules. Preserve that
layout and Candid's dev-dependency role; the stale library refactor was not merged.
PR #35 supplies the Platform adoption; its original focused evidence is retained
alongside the expanded local qualification. No release/publication tracker was closed. The optional hosted PR matrix was
still running at the final status check; no hosted/full-CI pass is claimed.

## Maintained boundaries and earlier research

Principal stays bounded bytes (max 29); Account stays owner then optional 32-byte
subaccount. Absent `01 04 00` and explicit zero `01 04 01` plus 32 zero bytes
remain distinct despite Account equality. Platform owns exports/execution;
consumers own authentication, dispatch, persistence, retries and deployment.
Preserve original retry/hash bytes. Synchronous reply failure rolls back; after
await only the current callback rolls back. See [endpoints](../endpoints.md).

The [Tripwire follow-up](../evidence/tripwire-followup-0101/README.md) remains research
on its recorded Platform 0.11.1 / Tooling 0.72.1 suppliers. Both reply writers were
rejected: even the shared writer adds 11–28 KB across 100 endpoint types. Selective
input passed 1,536 updates; a 256 B prefix of 1 MiB saved about 95.7% instructions
but only 0.186% cycles. It remains fixed-shape research needing a pre-copy contract.
LZ4 roundtripped 353 inputs with 1,216 measured queries; only 138 inputs shrank,
with 19–20 KB additional Wasm. Keep compression consumer-owned.

The [Candid matrix](../evidence/candid-comparison-0725/README.md) retains ordinary
vector controls and explicit Packed comparisons. Historical studies are not
relabeled to new suppliers. Evidence manifests use `.snapshot`, not active Cargo
workspace filenames. No experimental codec or dependency entered production.
