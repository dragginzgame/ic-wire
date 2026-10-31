# Published Wire 0.13 / Endpoint 0.2 qualification

On 2026-10-31 the delegated delivery adopted Shared Tooling 0.27.19
(`54fc49e12f439dbe18d232f622a6d78e61773353`) and completed both standard
release groups, each with its full local `make ci` gate. Git delivery preceded
registry publication; core was published before Endpoint.

| Package | Published version | Git tag / commit |
| --- | --- | --- |
| ic-wire / ic-wire-derive | 0.13.0 | core-v0.13.0 / 65d1b91600d64a1e4255774f6bfc8ad9752a63cb |
| ic-wire-endpoint | 0.2.0 | endpoint-v0.2.0 / dfd4978bf6792d12d3fdab6ee9a7b2affeabde1c |

[Registry proof](registry-proof.json) records non-yanked public sparse-index
checksums matching each actual uploaded `.crate` archive. Remote main/tag
observations are retained in `core-remote.txt` and `endpoint-remote.txt`.
Full release, dry-run and publication logs remain in `/tmp/wire-ship-02719/`;
recovery plans remain under `.git/release-state/`. No release effects are replayed
by these consumer runners.

## Registry-only consumer

[Log](published-consumer.log), frozen manifest/lock/source snapshots and
[runner](qualify-published.sh) use only registry Wire/Derive 0.13.0, Endpoint
0.2.0 and Platform facade 0.15.0. The compatible canonical Types selection is
0.15.1. There is no Cargo patch or local Wire supplier. Four schema-only and five
combined tests pass on Rust 1.91.0; both configurations check for
`wasm32-unknown-unknown`. Schema-only excludes runtime/macros; combined selects
one runtime guard. Wasm production graphs exclude Candid, PocketIC, the external
CDK executor and serde_json. Generated Rust/TypeScript frames and malformed-input
refusals agree, including absent, explicit-zero and nonzero subaccounts.

Reproduce with prepared Rust 1.91.0/Wasm and the repository's pinned Node and
TypeScript compiler installed, using a new output directory:

```sh
bash docs/evidence/published-pair-0130/qualify-published.sh /tmp/wire-published-013-repeat
```

The lock preserves the measured package selection; compiler path comes from
`examples/browser/node_modules/typescript`. The frozen Rust test generates the
browser runtime and vectors from the published schema implementation.

## Metrics/Testkit source alignment

[Log](aligned-consumers.log), [source inventory](source-alignment.json) and
manifest patches distinguish this from downstream publication. Released Metrics
0.22.0 and Testkit 0.73.0 Rust sources are copied unchanged into isolated output;
only each copied manifest's Wire requirement changes to registry `=0.13.0`.
A Rust 1.91 test exercises all four Metrics schema types, Principal/Account,
Endpoint Contract and Testkit's public Limits/Error plus generic query/update
signatures. Metadata resolves one registry Core, Derive and Types identity.
No Testkit/PocketIC service or sibling suite is run, and no sibling checkout is
modified. Published Metrics/Testkit still require their owners' adoption releases.

```sh
bash docs/evidence/published-pair-0130/qualify-aligned.sh /tmp/wire-aligned-013-repeat
```

This runner takes the Metrics/Testkit source selections from this checkout's
locked workspace metadata and requires their cached sources/dependencies.
The retained inventory identifies the selections qualified here.

## Protocol and performance

Delivery/tooling changes leave protocol bytes, budgets and acceptance unchanged.
The supplying Platform identity migration and matched instruction/cycle/Wasm
measurements are retained in the [Platform 0.15 study](../platform015-adoption/README.md).
The consumer tests are correctness/identity checks, not new performance or
Candid comparisons. Hosted Linux/macOS status is separate from both passing
local release gates; see the current handoff.
