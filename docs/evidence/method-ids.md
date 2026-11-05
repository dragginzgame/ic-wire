# Numeric method identity collisions

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Baseline: released 0.7.12, 974b6ff3e4a89c37a6c93081d7a26bee2e709f7b.
The Retired raw output (see retention notice) adds a set of parsed u64 method identities
to the existing name validation. Previously two names could share ID 7 and
interpret payload ff as either u8=255 or i8=-1. Generation now returns Schema
for duplicate numeric IDs, including different spellings and query/update pairs.
No partial source or diagnostic report escapes. Valid-registry output is unchanged.

This is a host metadata acceptance tightening. Consumers with conflicting
registries must assign distinct IDs and coordinate the affected deployed clients.
Independent registries and direct Rust/JavaScript codec calls still require
consumer-owned identity/schema binding. The fix does not add schema hashes or
restore canonical re-encoding of normalized collections. Runtime frames and
admission are unchanged. The regression test demonstrates both the same-ID
reinterpretation and rejection when the expected runtime ID differs.

## Host generation cost

The maintained scripts/benchmarks/generator/mod.rs probe measures records, enums
and methods with 8/128/1,024 entries. The first two are controls without methods.
Both builds use the same Retired raw output (see retention notice), Retired raw output (see retention notice), opt-level 3,
LTO and one codegen unit. No custom RUSTFLAGS were used. Each run warms 100 times,
calibrates batches to 20 ms (up to 4,096 repetitions) and takes 11 samples. Three
paired runs alternate order, giving 33 samples per cell. Compilation finishes
before timing. Diagnostic IO is excluded; reports are consumed through black_box.

| Input | Baseline ns | Candidate ns | Change |
|---|---:|---:|---:|
| 8 record | 2947 | 2905 | -1.4% |
| 128 record | 43203 | 42560 | -1.5% |
| 1024 record | 382102 | 364717 | -4.5% |
| 8 enum | 2933 | 2884 | -1.7% |
| 128 enum | 43860 | 43127 | -1.7% |
| 1024 enum | 401880 | 390652 | -2.8% |
| 8 methods | 9361 | 9396 | 0.4% |
| 128 methods | 144106 | 147341 | 2.2% |
| 1024 methods | 1254776 | 1252147 | -0.2% |

Values are pooled medians; Retired raw output (see retention notice) also retains per-run medians.
The six CSV files contain raw batches (kind,count,batch,repetitions,output_bytes,ns).
No speedup is claimed: the controls drift by up to 4.5%, and the method measurements
show small changes within that scale of host variation. The measured 128-method
case increases 2.2%, or about 3.2 microseconds. All nine generated sources are
byte-identical across baseline/candidate in all three runs. No IC-instruction or
Wasm-size change is claimed for this host-only check.
