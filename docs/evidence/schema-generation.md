# Hash advice and bounded host diagnostics

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Rust/Cargo 1.99.1, x86_64-unknown-linux-gnu, workspace `evaluation` profile
(opt-z, LTO, one codegen unit, panic abort, stripped), identical Cargo.lock and
manifests. Baseline is the pending Platform 0.21 source before this lint change;
candidate changes only `bulk_lint/mod.rs`. Each frozen copy has its own Cargo
target. The copied example is [the maintained probe](../../scripts/benchmarks/bulk-advice/mod.rs).
No native codec time, IC instructions or Wasm saving is claimed here.

The deterministic generator produces direct-hash, aliased-hash and integer
control records with 1/8/128/1,024 fields. Two serial baseline/candidate rounds,
eight warmups, calibrated batches (20 ms target, maximum 4,096 repetitions),
eleven samples per operation. Output byte lengths are prepared outside timing;
`Display` includes string allocation but excludes terminal I/O. Source and all
generated TypeScript files are retained in `generator-sources.tar.xz`.

| Host operation | Round | Before ns | After ns | Change |
| --- | ---: | ---: | ---: | ---: |
| Generate one hash field | 1 | 2,183.415 | 2,247.281 | +2.93% |
| Generate one hash field | 2 | 2,155.052 | 2,267.893 | +5.24% |
| Generate 1,024 hash fields | 1 | 1,292,422.688 | 1,497,367.750 | +15.86% |
| Generate 1,024 hash fields | 2 | 1,245,501.562 | 1,411,347.812 | +13.32% |
| Generate 1,024 integer fields | 1 | 1,065,198.562 | 1,062,837.531 | -0.22% |
| Generate 1,024 integer fields | 2 | 1,058,826.500 | 1,062,268.250 | +0.33% |
| Format 1,024 integer warnings | 1 | 40,793.410 | 555.105 | -98.64% |
| Format 1,024 integer warnings | 2 | 42,663.018 | 554.953 | -98.70% |

The hash generation increase is real additional work: baseline collected zero
hash warnings and candidate retains all 1,024. It adds about 0.17–0.20 ms for
that large synthetic registry. Existing integer controls are approximately
unchanged. Formatting emits eight entries plus a summary, rather than 1,024
lines. Tiny formatting has small regressions; [Retired artifact: summary and retention status](primitive-bulk.md)
and original eleven-sample CSVs retain them. This does not measure minutes of
terminal output or claim a terminal-I/O speedup.

All **24 matched generated outputs** (12 workloads per round) are byte-identical.
[Retired artifact: summary and retention status](primitive-bulk.md) record bytes and SHA-256. The diagnostic
report still holds every exact suppression path; only its default formatting is
bounded. Tests cover stale acknowledgements, aliases, newtypes, nested lists,
recursion and excluded fixed/byte/packed fields.

To reproduce, extract `generator-sources.tar.xz` into a new directory. For each
`before` / `after-lint` tree, use a distinct absolute `CARGO_TARGET_DIR` and run:

```sh
cargo build --locked --profile evaluation -p ic-wire --features typescript --example bulk_advice_probe
# Run the built evaluation/examples/bulk_advice_probe twice with fresh output
# directories, redirecting stdout to before-1.csv / after-1.csv etc.
```

Use the recorded compiler, prepare locked dependencies and repeat serially with
no competing measurement/build. The archive already contains the exact copied
probe under `crates/ic-wire/examples/bulk_advice_probe/main.rs`. The raw CSV
columns include kind, count, operation, batch, repetitions, output bytes, warning
count and ns/op. Aggregate each eleven-sample group by median; retain both rounds.
