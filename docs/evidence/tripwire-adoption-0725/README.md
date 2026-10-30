# Tripwire ideas applied to Wire

Measured 30 October 2026 against current 0.7.24 source; pending 0.7.25.
One production Serde/Postcard codec remains. The retained endpoint change checks
the existing eleven-byte header before copying the complete ingress and shares
cold error formatting across admission, decoding and reply encoding. No dependency,
wire-byte, limit, typed-error or accepted-input change. For ID 7 / u8 42 the frame
remains `49 57 07 00 00 00 00 00 00 00 00 2a`. Consumers need no migration.

Generated guards call `Contract::admit_request()`. Existing standalone
`admit_request(Limits)` remains a size-only primitive for manually composed
Platform guards. Both use the same size check; the contract method also uses the
existing header validator. Full decoder validation remains mandatory. Platform
still owns exports/execution; framework authentication ordering is unchanged.

## Retained endpoint result

All 17 compiled endpoints and 178 cases pass with matching request/reply hashes,
outcomes and transaction-state oracles: **5,696 measured logical calls /
6,048 physical executions** across baseline/candidate and both profiles.
Each case/profile/lane has eight measured repetitions after one excluded warmup.
The added cases exercise bad magic, direction, short headers and wrong identities
at the full request limit. Existing async, composite, borrowed, node-limit and
rollback cases stay covered. Queries use distinct callers and simulator rounds;
missing/incomplete physical traces fail the audit.

| Existing 17-endpoint fixture | Before total / code | After total / code |
| --- | ---: | ---: |
| Opt-z | 103,642 / 89,112 | 103,610 / 89,070 |
| Opt-3 | 98,503 / 84,974 | 98,631 / 85,191 |

Total Wasm changes −32 bytes (−0.031%) at opt-z and +128 (+0.130%) at opt-3;
code changes −42 / +217 bytes. This is not a universal size reduction.

For the 64 KiB wrong-ID default blob query, opt-3 instructions fall
389,819 → 243,392 (−37.56%); opt-z 390,797 → 243,786 (−37.62%). The corresponding
update refusal falls 709,840 → 483,392 (−31.90%) at opt-3, with observed cycle debit
138,273,840 → 138,047,392 (−0.164%). Ingress still transmits the whole request.
Successful calls pay for the extra header read: the largest relative measured increase
is 0.257% at opt-z / 0.251% at opt-3. A 32-byte default blob query rises
247,265 → 247,879 instructions at opt-3 (+614; +0.248%).

These are complete physical costs, including metered page charges, not isolated
codec counters. Query cycle debit is zero on this simulator, not zero work.
The frozen [comparison](retained.json) and [raw receipts](retained-samples.json.gz)
retain all improvements and regressions. Initial header-only results are retained
in [header-only.json](header-only.json): that candidate grew the full fixture by
993 / 1,558 bytes. Sharing error formatting reduced that cost without changing
failure messages or rollback behavior.

## Independent endpoint/type footprint

Each fixture has 1, 10 or 100 distinct Serde request structs, each with a u64 and
bounded byte field, and the same number of generated update exports. Actual
compiled exports and artifact hashes are checked. Both encode/decode are reachable.
These measure endpoint scaling, not 100 arbitrary selective-reader schemas.

| Endpoints/types | Profile | Total Wasm before → after | Code section before → after |
| ---: | --- | ---: | ---: |
| 1 | opt-z | 30,837 → 30,984 | 27,027 → 27,164 |
| 1 | opt-3 | 32,369 → 32,516 | 28,682 → 28,827 |
| 10 | opt-z | 51,795 → 51,369 | 47,114 → 46,673 |
| 10 | opt-3 | 61,161 → 60,417 | 57,206 → 56,458 |
| 100 | opt-z | 239,669 → 233,473 | 225,854 → 219,653 |
| 100 | opt-3 | 328,777 → 321,817 | 322,167 → 315,209 |

The one-endpoint module grows 147 bytes in each profile. Ten endpoints save
426 / 744 bytes; 100 save 6,196 (2.59%) / 6,960 (2.12%).
[scale.csv](scale.csv) records exact hashes. Sizes are total uncompressed Wasm and
code-section bytes; compression is used only to store evidence.

## Selective blob experiment — not a production API

The [fixture](selective/fixture/mod.rs) uses the same encoded request in every
lane: mode:u8, nonce:u64, trailing Bytes<1 MiB>. A bounded 27-byte lookahead and
Wire's existing strict decoder locate the body; no second varint decoder is added.
The complete byte-field range, field maximum and exact frame end are checked
before business work, including requests whose body is never consumed.

Native tests cover size/varint boundaries, selected read ranges, truncation,
nonminimal lengths/nonces, trailing data, field limits and over 10,000 mutations,
with matching typed errors against owned/borrowed Wire. All **1,536 measured
physical updates** pass exact result/frame and trace checks: four lanes, sixteen
valid workloads and eight malformed cases, eight inputs/repetitions, two profiles.
Eight initial empty warmups are excluded. Nonces vary across representation
boundaries; this spread is not a statistical confidence interval.

One MiB, opt-3 complete instructions (means rounded):

| Work | Existing owned | Existing borrowed | Direct owned, initialized buffer | Selective |
| --- | ---: | ---: | ---: | ---: |
| Full scan | 15,848,525 | 12,239,414 | 13,289,272 | 10,017,791 |
| First 256 bytes | 7,703,923 | 4,094,881 | 5,144,739 | 171,918 |
| Business rejection | 7,701,898 | 4,092,856 | 5,142,714 | 164,744 |
| Generate, encode and reply | 70,353,740 | 70,373,809 | 70,373,809 | 70,373,869 |

Against borrowed Wire, selective prefix reading saves 95.80% of instructions;
full scan saves 18.15%. Prefix observed cycle debit falls only 0.186%, because the
whole payload still travels over ingress. The existing encoder is unchanged;
its +60-instruction linked-context difference is about 0.000085%, not an encoder
optimization claim. A truncated 1 MiB frame is rejected in 321,862 instructions
versus 4,090,652 for the complete-buffer controls, including a mode that skips
body business work. Empty and 32-byte controls are retained in the raw reports.

| Blob fixture | Opt-z total / code | Opt-3 total / code |
| --- | ---: | ---: |
| Existing owned | 20,792 / 18,465 | 21,384 / 19,268 |
| Existing borrowed | 20,053 / 17,728 | 21,187 / 19,071 |
| Direct owned, initialized buffer | 20,853 / 18,284 | 21,202 / 19,082 |
| Selective | 20,909 / 18,324 | 21,162 / 19,042 |

Selective costs 856 more total bytes at opt-z (+4.27%) and saves only 25 at opt-3
(−0.12%) against borrowed Wire. It is not a general Wasm-size win. Direct owned
saves about 16.15% full-scan instructions against existing owned input, but remains
8.58% slower than borrowed input: the available safe range-copy control initializes
its destination first. Platform's existing whole-argument helper already avoids
that initialization pass; it exposes no equivalent Vec-returning range helper.

This research supports only that known DTO and fixed nonbinding traversal
allowances. It does not implement arbitrary Serde, general budget accounting,
text validation, source lifetimes across await, or generated endpoint integration.
Current Platform `decode_with` receives an already-copied Vec. Production adoption
needs a Platform-owned pre-copy input hook, scoped source lifetime and safe owned
range copy, followed by Wire's shared decoder/admission design. No new derive,
second production codec, unsafe allocation helper or silent validation relaxation
is introduced here. The next step is that narrow integration design, not publishing
this shape-specific locator as a generic reader.

## Reproducibility and qualification

Both studies use stable Rust 1.99.1, wasm32, existing opt-z/opt-3 profiles, LTO,
one codegen unit, panic abort and stripped symbols. The lock selects Platform
0.8.3 / Testkit 0.70.4 / Postcard 1.1.3 / Serde 1.0.229. The instrumented PocketIC
16.1.0 binary and source hashes are in [receipt.json](receipt.json); dependencies
are retained in [Cargo.lock](Cargo.lock). This is not a comparison against the
previous Tripwire study's different nightly compiler/fixture.

The existing host-expansion runner records eleven alternating samples per shape,
1,000 expansions each, under [generation](generation/samples.csv). Median ns:
unit 76,598 → 75,643; scalar 96,839 → 95,529; async record 115,383 → 114,634.
Other repositories were building on the host; these noisy host timings support
no speedup claim. They are not native codec time or IC instructions. No native
codec timing is required for the IC-only admission change; core codec source is
unchanged. No allocation-count claim is inferred from IC page charges.

Focused endpoint native/schema/diagnostic checks, strict Clippy, dependency
boundaries and Rust 1.91 Wasm checking pass, as do the installed matrices and
selective native/host/Wasm checks. Full CI, macOS and consumer integration were
not run. Incoming Shared Tooling changes are preserved. Nothing is committed,
published or released.

Prepare a baseline workspace at the receipt revision with the retained lock,
exact compiler, cached dependencies, instrumented server and codec-report tool.
Run the [header reproduction](reproduce-header.sh) or
[selective reproduction](selective/reproduce.sh), each with BASELINE_SOURCE and
FRESH_ABSOLUTE_OUTPUT arguments and prepared POCKET_IC_BIN / CODEC_REPORT.
The scripts copy inputs and build in private targets. [endpoint.patch](endpoint.patch)
and [cases.patch](cases.patch) retain the production/fixture changes;
[header-only.patch](header-only.patch) restores the initial unretained candidate.
The analyzers pair input/output hashes, actual exports, outcomes and physical costs.

The first header/scale attempt reused stale Cargo artifacts across frozen source
roots sharing one target. Those candidate outputs are excluded, retained under
`/tmp/wire-tripwire-adoption-0725/candidate/` and [scale-controls.csv](scale-controls.csv).
All reported changed candidates were rebuilt in separate target directories.
Original runs, failed compile attempts and logs remain under
`/tmp/wire-tripwire-adoption-0725/`.
