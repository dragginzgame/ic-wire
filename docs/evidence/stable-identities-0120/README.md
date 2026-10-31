# Stable Wire identities and endpoint extraction qualification

2026-10-31, local pending Wire 0.12 / Endpoint 0.1.0. Released core is still
0.11.0 (`568196014214a27b1a679b119ffde45284ed5eae`). Own existing versions are
unchanged; new Endpoint 0.1.0 is the initial package selection. No publication,
real checkout staging/commits/releases or sibling checkout edits are performed.

## Boundary and migration

Wire owns codec, schema trait, Limits/Error and Principal/Account schema impls.
`platform-types` now selects published `ic-platform-types` 0.1.0. Platform 0.14
reexports those exact types. `ic-wire-endpoint` independently owns Contract,
attributes, CDK hooks, admission and trapping replies. It uses Platform 0.14.
The existing derive crate retains metadata and endpoint generation; there is one
codec and one schema model. A small core header validator delegates to the same
primitive checks used before the move. No header layout is duplicated.

Remove `ic-wire/endpoint`, add `ic-wire-endpoint` and move attributes/helpers from
`ic_wire::endpoint` to `ic_wire_endpoint`. Keep codec/schema imports on `ic_wire`.
For native Contract schemas select `ic-wire-endpoint/typescript`. Align native
Testkit/metadata suppliers once on the supplying Wire minor. No Cargo patches,
wrappers, aliases or second trait are required in production consumers.

The protocol is unchanged: eleven-byte frame, method identity, Serde declaration
order, primitive checks, byte/allocation/node/depth budgets and retained retry/
hash inputs. Principal remains bounded bytes, max 29. Account remains owner then
optional 32-byte subaccount: `01 04 00` stays absent; `01 04 01` plus 32 zero bytes
stays explicit zero. Complete payload decoding remains required after pre-copy
header admission. Response encoding still traps with the same transaction rules.

## Evidence and scope

[Logs](checks/) retain standalone derives, current/MSRV host/Wasm, feature graph
isolation, exact Rust/TypeScript frames and malformed refusals, endpoint Clippy,
11 expected compiler diagnostics plus passing renamed-import control, core tests,
tooling diagnostics and installed endpoint limits/rollback. The current suppliers
are registry Platform 0.14 / Types 0.1, Build/Testkit 0.73.

`make check-stable-identities` additionally constructs a **simulated** next
endpoint 0.2 / facade 0.15 identity from the published source. Both independent
consumer builds pass the real five Principal/Account/Contract tests and Rust 1.91
host/Wasm checks, with one deployed runtime guard and no Candid/host serde_json.
[Package IDs](identities.json) prove identical codec, derive and stable-types
selections while endpoint/facade IDs change. This is a prospective identity test,
not qualification or publication of a future Platform implementation.

The [consumer alignment runner](qualify-consumers.sh) copies released Metrics
0.22 and Testkit 0.73 into disposable directories and changes only their Wire
manifest dependency path to the pending core. It exercises real Metrics metadata,
Testkit typed query/update signatures and Limits/Error causes against one core
identity. These are source-adoption inputs, not a claim that current registry
Metrics/Testkit already consume the pending Wire minor.

Both groups' disposable release fixtures exercise the real editor/adapter/runner,
exact file custody, validation failure, receipt tampering, commit/tag/atomic local
push and resume. Uploads and broad validation are substituted; source effects are
confined to disposable Git repositories. Publication selection/order is separately
checked by the Make sentinel fixture, using distinct package versions.
Archive verification against the next core/derive registry versions remains
part of ordered publication: existing own versions have not been advanced and
those supplying versions are not published yet.
No full CI, macOS matrix or new full Candid performance matrix is claimed.

## Matched installed comparison

Baseline is the prior monolithic endpoint with Platform 0.13; candidate is the
split plus Platform 0.14 / Types 0.1. Both use Tooling 0.73 and otherwise identical
registry selections. Therefore this measures the combined supplier/package change,
not isolated savings from moving a module. The retained patches identify every
input. Both profiles use release LTO, one codegen unit, panic abort and stripping.
Compiler/target: [Rust 1.99.1 / LLVM 23.1.1](compiler.txt), Linux x86_64 host,
wasm32-unknown-unknown guest. The runner freezes the [instrumented server](server.sha256).

There are 12 cases across seven exports, eight measured repetitions after an
excluded warmup, 384 logical calls / 608 physical executions. All request/reply
hashes, outcomes and transaction oracles match. [Comparison](comparison.json) and
[raw traces](samples.json.gz) retain absolute costs, samples and variation.

| Profile | Total uncompressed Wasm | Code section |
| --- | ---: | ---: |
| opt-z | 99,951 → 100,665 B (+714 B, +0.714%) | 86,068 → 86,766 B (+698 B, +0.811%) |
| opt-3 | 93,518 → 94,073 B (+555 B, +0.593%) | 80,630 → 81,127 B (+497 B, +0.616%) |

The largest instruction regression is +0.0255%; some opt-z async paths improve
slightly. These small layout/call differences are measured costs of this batch,
not a size optimization claim. Debited cycle differences are −39.5 to +115 per
logical call in this harness; query cases debit zero. See the JSON for absolute
cycle means. No native time or gzip artifact metric is used.

| Method/input | opt-z IC instructions | opt-3 IC instructions |
| --- | ---: | ---: |
| `echo/blob-0` | 253,815.00 → 253,828.00 | 247,492.00 → 247,520.00 |
| `echo/blob-32` | 256,230.00 → 256,246.00 | 249,729.00 → 249,757.00 |
| `borrowed_echo/blob-32` | 255,381.00 → 255,397.00 | 249,485.00 → 249,513.00 |
| `async_query/value-0` | 256,985.00 → 257,013.00 | 250,138.00 → 250,166.00 |
| `composite/self-call` | 1,488,232.00 → 1,488,207.00 | 1,473,022.00 → 1,473,102.00 |
| `async_mutate/reply-0` | 1,245,511.25 → 1,245,471.75 | 1,230,033.00 → 1,230,113.00 |
| `async_mutate/reply-117` | 1,727,584.00 → 1,727,555.00 | 1,711,693.00 → 1,711,808.00 |
| `default_blob/blob-32` | 252,355.00 → 252,375.00 | 247,824.00 → 247,852.00 |
| `default_mutate/reply-32` | 492,549.00 → 492,569.00 | 487,891.00 → 487,919.00 |
| `default_blob/blob-65522` | 3,884,431.00 → 3,884,451.00 | 3,879,734.00 → 3,879,762.00 |
| `async_query/truncated` | 249,778.00 → 249,806.00 | 246,849.00 → 246,912.00 |
| `default_blob/wrong-id-at-limit` | 243,786.00 → 243,813.00 | 243,392.00 → 243,454.00 |

## Reproduction

Prepare the recorded compilers (including Rust 1.91), Node 24.21 / TypeScript
7.0.2 and locked dependencies. Focused checks install nothing:

```sh
make check-platform-types check-endpoint check-release check-tooling check-pins check-boundary
make test check-typed-diagnostics check-msrv
bash docs/evidence/stable-identities-0120/qualify-consumers.sh
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic-server \
  bash docs/evidence/stable-identities-0120/reproduce.sh /tmp/wire-identities-repeat
```

The measurement output directory must be fresh. Local loopback is required.
It reconstructs the baseline revision plus admitted dependency patch and applies
the candidate patch, builds isolated targets, installs the real candidate and
checks exact traces. Original successful output is
`/tmp/wire-identity-split/comparison-final`. A first patch-context refusal stopped
before builds; it is not accepted as measurement evidence.
Publication and registry/downstream adoption remain the maintainer's next step.
