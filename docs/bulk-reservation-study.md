# Fresh-encoding bulk reservation experiments

Against 0.5.16 (`24c1250`), two candidates override fresh encoder growth when a
single bulk write requires at least 4,096 bytes. Both remain unselected. Production
runtime code, wire bytes, admission limits and error behavior are unchanged.
Neither candidate changes reusable scratch encoding or native dynamic encoding.

The released policy grows geometrically until the required length exceeds one
quarter of the finite schema ceiling, then reserves the ceiling. The **exact**
candidate instead reserves just the required length for bulk writes. The
**headroom** candidate adds 32 bytes, capped at the same schema ceiling, so an
immediately following scalar prefix can fit without another growth step. Small
writes retain the released growth policy. Both retain all limit/overflow checks.

## Capacity and timing

[Capacity results](evidence/bulk-reservation/capacity.csv) measure the returned
Vec's capacity, outside timed loops. This is not allocator usable size, peak
memory, allocation count or process RSS. The probe now writes per-case `.capacity`
files containing frame length and capacity. Actual emitted bytes are identical.

| Input | Frame bytes | Released capacity | Exact capacity | Headroom capacity |
| --- | ---: | ---: | ---: | ---: |
| 262,128-byte blob | 262,142 | 262,142 | 262,142 | 262,174 |
| 262,144-byte blob | 262,158 | 1,048,597 | 262,158 | 262,190 |
| 524,288-byte blob | 524,302 | 1,048,597 | 524,302 | 524,334 |
| Four 64 KiB chunks | 262,168 | 262,204 | 393,258 | 262,200 |
| Sixteen 64 KiB chunks | 1,048,636 | 1,048,757 | 1,048,757 | 1,048,668 |

Exact reservation saves 75%/50% capacity for the two single-blob threshold cases,
but increases four-chunk capacity by 50%. Small intervening prefixes still use
ordinary growth, so exact bulk reservation is not an overall minimum-capacity
policy. Headroom removes this counterexample while retaining the single-blob
capacity savings, at the cost of up to 32 extra bytes after a bulk write.

[Timing medians](evidence/bulk-reservation/native.csv) and
[all samples](evidence/bulk-reservation/samples.csv) include two paired runs per
candidate and all original encode/decode/canonical controls. Selected headroom
encode results, in microseconds per complete fresh encode including destruction:

| Input | Baseline runs 1 / 2 | Headroom runs 1 / 2 | Change |
| --- | --- | --- | --- |
| 262,144-byte blob | 4.78 / 5.10 | 4.09 / 4.88 | 14% / 4% faster |
| 524,288-byte blob | 9.51 / 8.91 | 9.97 / 9.87 | 5% / 11% slower |
| Four chunks | 4.31 / 4.38 | 4.84 / 4.40 | 12% / 0% slower |
| Sixteen chunks | 18.79 / 18.56 | 19.72 / 18.84 | 5% / 2% slower |

Exact-candidate timings were inconsistent, including a 28% regression followed
by a 19% improvement for the 262,144-byte case. No universal encode speedup is
established. Neither policy replaces the default: exact has a capacity regression;
headroom exchanges lower capacity on selected shapes for repeated timing costs.
No configurable production allocation mode is introduced.

## Reproduction and qualification

Use the [isolated runner](../scripts/benchmarks/protocol/run.sh) with either
retained patch, following the candidate-file procedure in the
[native comparison study](native-comparison-study.md#reproduction). Patch inputs
are [exact](evidence/bulk-reservation/exact.patch) and
[headroom](evidence/bulk-reservation/slack.patch). Apply them to the released
`crates/ic-wire/src/contract/mod.rs` and pass the resulting file as argument three.

The [probe](../scripts/benchmarks/protocol/mod.rs) adds blob lengths 262,128,
262,144 and 524,288 to its existing 0/32/4096/1048576 cases. The bound remains
1 MiB; the threshold pair straddles the quarter-ceiling jump including framing.
New list cases contain four or sixteen 65,536-byte blobs, each byte 42, with
`List<Bytes<65536>,16>` and method ID 10. Other schemas, values, limits and stage
semantics match the [protocol study](protocol-study.md#reproduction-and-scope).

Each experiment uses two runs ordered baseline/candidate then candidate/baseline,
100 warm-up operations, calibration to 10 ms or 65,536 calls, and 11-batch medians.
Inputs, frame comparisons and capacity observations are outside timing; allocations
and destruction are inside. Both variants compiled and passed the probe's
roundtrip, nonminimal-integer refusal and identical-frame checks. Rejected
candidates were not qualified through the full malformed-input/allocation suites.

Linux x86_64, Threadripper 7970X, Rust 1.99.1, default release profile, no Rust flag
overrides and no fixed CPU affinity were used. Only one experiment ran at a time.
These are native timings and returned capacities, not IC instructions, Wasm size,
peak-memory or macOS results. Source identities (SHA-256):

- Shared lock: `d13c18d9cb7ca6d86ad7f1c09c869d929180aaa4d19d77f7012b913bea04648a`.
- Extended probe: `ca8de6827430b09fcb8cbfd0fbdcff85a0a18b8b546defa8e461fb3ae5bf2486`.
- Exact candidate: `7345238d97680b28ab4a7ecd75b5ef7945df8165011aadffc54256123300e681`.
- Headroom candidate: `ca2d4d173c3323800ea08565a21d2a6571d2b8c941a2bac466db394c22f74a40`.

Raw builds, frames and capacities remain in `/tmp/ic-wire-bulk-results/` and
`/tmp/ic-wire-bulk-slack/`; candidate sources are in `/tmp/ic-wire-bulk-study/`.
