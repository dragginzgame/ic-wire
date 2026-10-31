# Published Core 0.13.1 / Endpoint 0.2.1 delivery

2026-10-31. Explicit maintainer deployment delegated scoped Git releases and
crates.io publication. [Exact identities](release-identities.json):

| Group | Published packages | Tag / commit |
| --- | --- | --- |
| Core | ic-wire / ic-wire-derive 0.13.1 | core-v0.13.1 / 974510a88590f2bd54e8ea45de90e0830e928063 |
| Endpoint | ic-wire-endpoint 0.2.1 | endpoint-v0.2.1 / a6cfed5b367fff84997985a357ceeb6731ec5400 |

Source preparation is commit 203a97c. Each standard patch release completed its
full real local `make ci` gate (Core 673 s, Endpoint 731 s), metadata preparation,
commit/tag and atomic main/tag push. Both publication dry runs and actual uploads
succeeded in supplying order: derive/Core before Endpoint. No sibling working
checkout or cleanup was touched. The two [remote observations](endpoint-remote.txt)
retain exact tag refs; normal release journals remain under .git/release-state.
Full effect logs remain under target/deploy-0131 and those Git validation journals.
These effects are complete and must not be replayed by qualification runners.

[Registry proof](registry-proof.json) independently downloads all three official
archives and matches their public non-yanked sparse-index checksums to the local
upload archives. Cargo VCS identity is clean and equals each exact release tag,
with the expected crate path. The normalized public Core manifest no longer has
the libfuzzer-sys dev dependency. Retained publication logs distinguish dry run
from upload; these are registry facts, not inferred from Git tags.

## Actual supplying runtime

Requirements remain compatible with Platform 0.15.2, but the committed release
lock selects **Platform/macros/guard 0.15.3**, canonical Types **0.15.1**,
Testkit **0.75.0** and Build/Host **0.73.1**. Testkit's arithmetic-only Metrics is
0.23.2; the schema consumer below also selects published Metrics 0.24.0.
[Registry source comparison](platform0153-review.json) finds Platform's 112 src
files, five macro files and runtime guard byte-identical to 0.15.2. An isolated
copy of the actual committed source passes installed limits/transactions and
reply rollback using the previously prepared server;
[installed log](runtime-installed.log) records this 0.15.3 execution separately
from earlier 0.15.2 evidence.

## Registry-only consumer

[Log](registry-consumer.log), frozen manifest/lock/source snapshots and
[runner](qualify-published.sh) use registry Core/Derive 0.13.1 and Endpoint 0.2.1,
with no Cargo patch, local Wire supplier, copied producer implementation or wrapper.
The locked facade is 0.15.3 and canonical Types 0.15.1.

- Rust 1.91.0 passes four schema-only and five combined tests, standalone derives
  and both Wasm checks. Schema-only excludes runtime/macros; combined has one
  runtime guard. Production Wasm excludes Candid, PocketIC, external executor,
  host serde_json and fuzz/profiler dependencies.
- Generated Rust/TypeScript Principal/Account frames and malformed-input refusals
  agree, including absent, zero and nonzero subaccounts.
- A separate Rust 1.91 typed consumer composes all four native Metrics 0.24 schemas,
  Principal/Account, Endpoint Contract and Testkit 0.75 Limits/Error plus typed
  query/update signatures. Exactly one registry Core/Derive/Types identity resolves.
  This is typed API/schema qualification, not an executed remote Testkit typed call.

Reproduce with the frozen registry cache, prepared Rust 1.91/Wasm and repository's
pinned Node/TypeScript compiler, using a fresh absolute output directory:

```sh
bash docs/evidence/published-pair-0131/qualify-published.sh /tmp/wire-published0131-repeat
```

The compiler is read from examples/browser/node_modules/typescript. The runner
installs nothing and has no Git/release/publication effects.

## Protocol and performance

Codec/schema bytes, budgets, field order, primitive/malformed acceptance, handler
rollback and retained retry/hash inputs are unchanged. Principal remains max-29
bounded bytes; Account absent `01 04 00` and explicit zero `01 04 01` plus 32 zero
bytes remain distinct. The complete catalog and opt-in qualification changes are
build/tooling ownership changes, with no new codec or schema trait.

The [qualification-tools evidence](../qualification-tools-0131/README.md) retains
49 codec tests, four qualification tests, 1,000 address-sanitized fuzz executions,
both profiler smoke tests and explicit conflict diagnostics. New release delivery
claims no runtime speed, Wasm-size or build-time saving. Earlier artifact studies
retain their recorded inputs. The prepared server lacks the instruction-trace
API, so a fresh instruction/Candid performance matrix was not run. Local full CI
is separate from the still-running hosted Linux/macOS matrix; see the current
handoff for bounded hosted observations.
