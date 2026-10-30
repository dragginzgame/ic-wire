# Platform identity schema transfer

Wire's opt-in `platform-types` owns the existing host `Wire` implementations for
Platform's owned Principal and Account. Platform still owns Serde. No codec,
primitive acceptance, frame identity, budget or retry-byte rule changes.

## Inputs and release boundary

The baseline is released Wire 0.9.0 at the recorded
[revision](baseline-revision.txt), with registry Platform 0.10.0. The candidate
uses the [Wire transfer patch](wire-transfer.patch) and a frozen copy of prepared
Platform source: [421edbc](platform-revision.txt) plus the
[reviewed source removal](platform-prepared.patch). Only that isolated copy's
workspace/macros/runtime-guard versions are prepared as 0.11.0. No sibling
working checkout is modified. The [lock delta](supplier-lock.patch) replaces
just those three suppliers; other dependencies and codec settings match.

Platform 0.11.0 was unavailable on crates.io during this initial qualification.
The new feature was independently qualified on registry 0.10 and the prepared
0.11 source. The subsequent [published-supplier adoption](published/README.md)
selects registry 0.11 in the shared root dependency and moves the pending Wire
release to 0.10.0. The measurements below retain their original prepared-source
scope; they are not substituted for the published-supplier results.

## Schema and isolation proof

`make check-platform-types` builds an independent consumer, outside Wire's
workspace feature unification. It tests both `platform-types` alone and combined
with `endpoint`, on native and Wasm with Rust 1.91. Native strict Clippy and the
prepared TypeScript compiler also pass. The check joins the existing TypeScript
CI group and retains graphs and generated outputs in its reported temporary path.

- Principal schema is exactly bounded bytes with maximum length 29. Empty,
  anonymous and length-29 owners roundtrip; lengths 30, 31 and 255 refuse.
- Account fields remain owner then optional 32-byte subaccount. Rust and TS
  encode identical request/reply frames and retain absent, zero and nonzero
  subaccounts. For anonymous ownership, payload `01 04 00` stays distinct from
  `01 04 01` followed by 32 zeros, regardless of Account's equality semantics.
- Short/long subaccounts, invalid option tags, oversized owner, trailing bytes,
  wrong method and wrong direction refuse in Rust and TS.
- The schemas and vectors generated on registry 0.10 and prepared 0.11 compare
  byte-for-byte equal. The recovered implementation and fixtures came from the
  five paths specified at Platform 421edbc; the endpoint contract test is new.
- Schema-only graphs exclude CDK/macros/runtime guard, executor, Candid and
  endpoint machinery. Combined graphs select one coordinated Platform runtime.
  Wasm excludes serde_json; default Wire excludes Platform and serde_json.

Rust limits remain 512 frame bytes, 65,536 allocation-allowance bytes, 1,024 nodes and
16 depth, as in the recovered tests. There is no new serializer or schema trait.
Host schema generation/native codec time was not benchmarked: the transferred
schema construction and Serde implementation are unchanged, with no speed claim.

## Endpoint qualification and matched measurements

Prepared-0.11 native/schema/strict lint, 11 compiler diagnostics plus passing
control, Wasm and Rust 1.91 checks pass. Installed endpoint limits, malformed
inputs and transaction/rollback checks pass. A graph-check false positive matching
`candid` inside a local directory named `candidate` was corrected to match package
names. Runtime graphs remain Candid-free and have one executor owner; Testkit's
native-only Platform 0.9.1 does not enter the canister graph.

The focused probe reuses the [12-case selection](../platform010-tooling071/cost-cases.patch)
and [analysis](../platform010-tooling071/analyze.mjs). It covers seven exports,
empty/32-byte controls, a 65,522-byte blob, malformed input, borrowed replies,
async callbacks, composite self-call and mutation/reply failure. Each lane uses
one warmup plus eight repetitions, giving 384 measured logical calls and 608
physical executions across both suppliers and profiles. Compiler:
[exact identity](compiler.txt); instrumented server: [SHA-256](server.sha256).
Both profiles use LTO, one codegen unit, panic abort and symbol stripping.

| Metric | Registry 0.10 | Prepared 0.11 | Change |
| --- | ---: | ---: | ---: |
| opt-z total uncompressed Wasm | 99,947 B | 99,819 B | -128 B (-0.128%) |
| opt-z code section | 86,068 B | 86,068 B | 0 |
| opt-3 total uncompressed Wasm | 93,518 B | 93,390 B | -128 B (-0.137%) |
| opt-3 code section | 80,630 B | 80,630 B | 0 |
| opt-z empty echo instructions | 253,815 | 253,815 | 0% |
| opt-z largest blob instructions | 3,884,431 | 3,884,431 | 0% |
| opt-z mutate/reply-32 instructions | 492,549 | 492,549 | 0% |
| opt-z mutate/reply-32 observed cycle debit | 7,000,549 | 7,000,549 | 0% |
| opt-3 empty echo instructions | 247,492 | 247,492 | 0% |
| opt-3 largest blob instructions | 3,879,734 | 3,879,734 | 0% |

Every instruction and observed cycle sample matches, not only the averages;
frames, outcomes and transaction oracles also match. Zero query cycle debits are
the harness's observed balance changes, not a general query billing claim.
The total-Wasm difference is confounded by shorter embedded local-source paths
in the candidate; **do not treat it as codec savings**. Code size is unchanged.
There are no measured instruction/cycle regressions in this subset. This is not
a full endpoint/Candid matrix or qualification of a downstream application.

[Full per-case results](comparison.json) include absolute samples, ranges and
percentages; [raw samples](samples.json.gz) retain physical-execution evidence.
Compression is only evidence storage, never a Wasm acceptance metric.

## Reproduction

With the recorded compiler, cached dependencies, Rust 1.91 native/Wasm targets,
prepared browser TypeScript compiler, explicit instrumented server and a
read-only Platform Git repository containing 421edbc:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic-server \
  bash docs/evidence/platform-types-transfer/reproduce.sh \
  /absolute/fresh/output /absolute/path/to/ic-platform
```

The runner reconstructs frozen sources, verifies the supplier-only lock delta,
runs schema/endpoint checks and the focused installed comparison. It performs no
fetch, installation, publication, release or sibling writes. Results retain
source hashes, compiler, server identity and artifact-bound physical traces.
Original local outputs are `/tmp/wire-platform-types-transfer/`; schema outputs
are `target/platform-types.seVVKN` and the candidate's
`target/platform-types.JqpLeE`. The reconstruction, lock delta and source equality
were checked separately; a second complete benchmark run was not required.
