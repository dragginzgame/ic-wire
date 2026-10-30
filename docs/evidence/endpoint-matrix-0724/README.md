# Complete endpoint cost matrix

This is the first complete cost baseline for the pending endpoint integration,
not an optimization comparison. It measures the actual standalone fixture Wasm,
with no added canister counters: 17 exported endpoints and 110 cases, each with
one excluded warmup and eight measured calls under both existing profiles.
Production runtime, generated entrypoints and protocol are unchanged by this
qualification batch. Consumer application endpoints remain consumer-owned.

The later [upstream adoption comparison](../upstream-adoption-0724/README.md)
qualifies Platform 0.8.3 / Tooling 0.70.3 against this original frozen baseline.

## Coverage and acceptance

The runner reads exports from the compiled Wasm using `WebAssembly.Module.exports`.
The test and report require exact equality between that export inventory and the
benchmarked methods. Adding a fixture endpoint without a case fails the run.
Each endpoint has a successful control and wrong-ID, trailing-byte, oversized and
truncated requests. Additional cases cover empty/small/boundary blobs, maximum
u64 values, node limits, default admission, guarded refusals and reply overflow.

The matrix includes owned and borrowed query/update handlers, unit requests and
responses, ready async handlers, an update that actually suspends, a composite
query with a self-call, renamed methods, implicit IDs, independent limit overrides
and framework-style guarded dispatch. Every reply is checked against an encoded
oracle or the observed state. State checks verify no mutation on admission failure,
synchronous rollback, and the retained pre-await mutation when a callback traps.

## Measurement semantics

`ic_instructions` sums **all physical guest executions** for one logical call:
entrypoint, self-call, reply callback and any cleanup callback. Page-access charges
are included, also reported separately; do not add them again. Successful async
calls are not reduced to their first execution. Failed calls retain actual traced
work instead of being recorded as zero. Nonreplicated queries have different
page-write tracking from updates; the matrix preserves each physical record.

`cycles_debited` is the observed canister balance decrease across the call on the
selected application subnet. It includes applicable update/ingress/call fees and
execution charges; it is not inferred by multiplying an instruction count. Query
cycle debits are zero in this simulator while instructions are nonzero. These
are measurements for this server/configuration, not universal IC billing rates.
Native timing, host latency, scheduler CPU time and production contention are
not measured. Full guest instructions do not imply all host work was measured.

The test advances a simulator round before sampling and uses a distinct caller
for each invocation to avoid query-result cache reuse. The caller and exact
request/reply hashes are recorded. State-observation queries, round advancement,
installation and tracing HTTP requests are outside the measurement interval.
A missing physical record, mismatched module, dropped/truncated trace or incomplete
instruction detail fails the benchmark. Every sample is retained; reported ranges
are observations, not confidence intervals. Reply byte counts are Wire frame
sizes; transport refusals have no Wire reply and are represented by null.

Wasm measurements describe the **whole 17-endpoint artifact**, not a per-method
attribution or a consumer's predicted savings. Both total uncompressed bytes and
code-section payload bytes are reported. No gzip metric is used.

## Results

Both profiles pass **1,760 measured logical calls**, covering **1,936 physical
executions**, plus 220 excluded warmup calls. All 17 exports have cost coverage;
all 110 cases pass reply/refusal and state-oracle checks. Cross-profile caller,
request and logical reply hashes match. Focused host Clippy also passes.

Small successful controls for every export are below; this table is not a blended
workload average. Each cell is the mean of eight samples. The complete per-case
reports include large/boundary controls, all failures, per-sample values and ranges:
[evaluation](evaluation.json), [benchmark-speed](benchmark-speed.json).

| Endpoint | Control | opt-z instructions | opt-z cycle debit | opt-3 instructions | opt-3 cycle debit |
| --- | --- | ---: | ---: | ---: | ---: |
| echo | blob-0 | 253,966 | 0 | 247,107 | 0 |
| borrowed_echo | blob-0 | 253,758 | 0 | 247,326 | 0 |
| count | unit | 251,448 | 0 | 246,478 | 0 |
| unit-call | unit | 488,803 | 6,984,803 | 484,676 | 6,980,676 |
| async_query | value-0 | 255,293 | 0 | 248,491 | 0 |
| stable-name | value-0 | 249,111 | 0 | 245,077 | 0 |
| response_only | scalar | 249,111 | 0 | 245,077 | 0 |
| composite | self-call | 1,484,952 | 0 | 1,469,820 | 0 |
| mutate | reply-0 | 489,999 | 6,981,999 | 485,161 | 6,977,161 |
| async_mutate | reply-0 | 1,245,543 | 18,038,543 | 1,229,259 | 18,022,259 |
| async_ready | reply-0 | 494,324 | 6,996,324 | 487,479 | 6,989,479 |
| borrowed_mutate | reply-0 | 490,733 | 7,068,733 | 485,586 | 7,063,586 |
| nodes | nodes-0 | 249,468 | 0 | 244,916 | 0 |
| default_blob | blob-0 | 249,818 | 0 | 245,095 | 0 |
| default_mutate | reply-0 | 490,007 | 6,998,007 | 485,166 | 6,993,166 |
| default_unit | unit | 248,700 | 0 | 244,666 | 0 |
| composed | allow | 489,859 | 6,985,859 | 485,111 | 6,981,111 |

For example, an opt-z async update with an oversized reply costs **1,727,846
instructions / 18,520,846 cycles** across four physical executions, including the
cleanup callback. This is recorded as a failed logical call with its actual work
and the expected +10 pre-await state change; callback mutation rolls back.

| Whole fixture | Total uncompressed Wasm | Code-section bytes |
| --- | ---: | ---: |
| evaluation | 103,638 | 89,112 |
| benchmark-speed | 98,503 | 84,974 |

No per-endpoint Wasm attribution or speedup over a prior complete matrix is claimed.

## Reproduce

```sh
POCKET_IC_BIN=/absolute/prepared/instrumented/pocket-ic-server \
  make evaluate-endpoints EVIDENCE_DIR=/absolute/fresh/directory
```

The runner freezes the current workspace manifests, lock and sources before
building `evaluation` (opt-z) and `benchmark-speed` (opt-3), with LTO, one codegen
unit, abort panics and stripped symbols. The [receipt](receipt.json) identifies
the compiler, source, lock, server, raw evidence and artifacts. Use the instrumented
PocketIC 16.1.0 server in that receipt; the standard server lacks the trace API.
IC Testkit owns the private server and cleanup. Local loopback required sandbox
approval. No simulator or profiler code is added to the published runtime.

The raw directories retain every call's physical records in `samples.jsonl`,
server configuration/provenance in `trace-header.json`, frozen sources and Wasm.
The checked-in per-profile JSON retains every measured instruction/cycle/page
sample and its range. Run `node scripts/ci/summarize-endpoint-matrix.mjs PROFILE_DIR`
to independently audit the raw receipts and regenerate a report. Compare future
runs by method, mode, case, input identity, compiler, dependencies and configuration;
no arbitrary universal regression threshold is imposed.

The initial attempt detected missing query records and was rejected. The corrected
harness requires simulator-round publication and distinct callers, then checks
physical trace presence on every call. Earlier failed logs remain in the raw
receipt paths; they contribute no accepted samples.
