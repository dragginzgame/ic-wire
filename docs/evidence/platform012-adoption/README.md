# Platform 0.12 integration qualification

PR [#35](https://github.com/dragginzgame/ic-wire/pull/35) merged as `2a6d186`.
The initial contribution below is followed by additional Wire-side installed
qualification against the pending Tooling 0.72.1 supplier. No codec bytes,
budgets, accepted-input rules or retry/hash inputs change.


2026-10-31; isolated Wire contribution from release
`c9d4e7b45b9ba6d841d7e2a664877e3bea035a5b` (0.10.0).
Only the optional Platform requirement, its three locked packages and migration
documentation change. Own package versions stay 0.10.0; notes target 0.11.0.

Published Wire 0.10 plus direct Platform 0.12 fails E0277 for native Principal
and Account schema bounds, because Wire implements the trait for Platform 0.11
identities. The selected Platform 0.12 supplier fixes those bounds without new
schema implementations or changes to generated shapes, frames or acceptance.
[#34 — Supply platform-types schemas compatible with published Platform 0.12](https://github.com/dragginzgame/ic-wire/issues/34)
tracks publication. Consumers align direct Platform to 0.12 and public Wire
types to the supplying minor. [Tooling #274 — Align Testkit typed Wire calls](https://github.com/dragginzgame/ic-tooling/issues/274)
tracks the corresponding typed native client release.

## Reproduction and results

Prepare the documented Rust 1.99.1 development compiler, Rust 1.91.0 minimum,
their Wasm targets, Node 24.21.0 and TypeScript 7.0.2. Tests admit prepared
inputs offline. Run from this topic checkout:

```sh
bash scripts/ci/check-platform-types.sh /absolute/prepared/typescript/bin/tsc
bash scripts/ci/check-endpoint.sh
make check-boundary check-pins check-doc-links
git diff --check
```

The standalone consumer uses the checkout's Wire source and exact registry
Platform 0.12.0, without a patched supplier. Results:

- Schema-only and combined endpoint consumers pass Rust 1.91 host/Wasm checks
  and strict Clippy. Four schema-only and five combined tests pass.
- Principal Rust/TypeScript frames and length refusals match. Account bytes
  preserve absent, zero and nonzero subaccounts; malformed request/response
  refusals agree. For anonymous owner, absent payload `01 04 00` remains distinct
  from `01 04 01` followed by 32 zero bytes.
- Schema-only graphs contain no runtime or Candid; core-only graphs contain no
  Platform. Combined endpoint graphs select the matching macro/runtime guard;
  Wasm schema selection contains no host serde_json.
- Endpoint schema tests (3), automatic method identity control (1), 11 expected
  compiler diagnostics and passing control succeed. Endpoint/Wire Clippy,
  optimized Wasm build, Rust 1.91 Wasm check and dependency graphs pass.
- Dependency declarations/boundaries, changed-document links and diff checks pass.

Run artifacts are retained in Auth's
`target/platform-adoption-20261031.306Irl/`: `published-consumer.log`,
`platform-types.log`, `endpoint.log` and the isolated `upstream/ic-wire/` checkout.
The schema script owns a disposable target; the endpoint run inherited Auth's
repository-local Cargo target directory. No full CI, macOS, installed PocketIC
execution or performance study was run for this topic.

No runtime or generator cost change is claimed. Recursive byte comparison of
the registry packages' `src/` trees shows Platform, macros and runtime guard
0.12.0 identical to 0.11.0; Wire Rust source is unchanged. The new package
identity and optional gzip dependency are the supplier changes. Existing cost
studies remain evidence for their own selections, not new measurements here.

## Expanded Wire-side qualification

The local follow-up compares Platform 0.11.1 → 0.12.0 while holding IC Tooling at
0.72.1. [Baseline revision](baseline-revision.txt), [pending baseline inputs](baseline-inputs.patch)
and [supplier-only patch](inputs.patch) reproduce that pair. The
[source comparison](source-review.json) finds identical Platform/macros/guard
`src/` trees. Rust code in Wire is unchanged. Both integrations share the root
Platform 0.12 selection; Testkit 0.72.1 retains a native-only Platform 0.11.1.
That development-only copy selects no second executor. Typed client adoption is
still tracked separately in Tooling #274; this does not publish a new Wire identity.

Focused endpoint, standalone type/schema, Rust 1.91 host/Wasm, feature isolation,
strict Clippy, exact Rust/TypeScript bytes, malformed inputs and dependency checks
pass again. The three Candid corpus roundtrip/rejection tests pass. Installed
limits and rollback also pass on the actual candidate Wasm; see [logs](checks/).

The matched seven-export, 12-case probe includes successful small inputs,
large-blob fallback, async/inter-canister execution and malformed-input controls.
Both profiles use eight measured repetitions after an excluded warmup:
384 logical calls and 608 physical executions. Frame hashes, outcomes and
transaction oracles match. Every instruction and cycle sample is identical.
[Comparison](comparison.json) retains absolute results and per-case samples;
[raw traces](samples.json.gz) bind artifact and server identities.

| Profile | Total uncompressed Wasm | Code section |
| --- | ---: | ---: |
| opt-z | 99,951 → 99,947 B (−4 B, −0.004%) | 86,068 → 86,068 B |
| opt-3 | 93,518 → 93,518 B | 80,630 → 80,630 B |

The four-byte non-code difference is not a codec optimization claim. Gzip size
is not an acceptance criterion. No native-time or new full Candid performance
comparison is claimed. The first run detected the mutable profiler binary being
rebuilt between profiles. The final receipt uses the [frozen binary identity](server.sha256):
all samples and installed checks were rerun on it after restoring the original
`log 0.4.34` selection in both lanes. Both guest artifacts were byte-identical
after that lock alignment. Mixed-server evidence was not accepted.

To reproduce with the [recorded compiler](compiler.txt), prepared locked registry
packages and explicit instrumented server:

```sh
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic-server \
  bash docs/evidence/platform012-adoption/reproduce.sh /tmp/wire-platform012-repeat
```

The runner copies the server before building, prepares isolated source and target
directories, installs disposable canisters and verifies exact traces. No sibling
working checkout is changed. Local loopback permission is required. Raw outputs
are retained at `/tmp/wire-platform012-adoption/`. This remains focused qualification,
not full CI or a macOS run. Earlier Tooling/editor and Tripwire studies retain their
original supplier identities.
