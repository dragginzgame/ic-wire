# Automatic bulk advice: host generation cost

Baseline: released 0.7.10, commit c2adbfb47da6c180bc470c0d5204a35d7b61d574.
Candidate: the attached [patch](candidate.patch), introducing required diagnostic
handling and reusing bounded rooted bulk traversal. This measures host generation
only; codec instructions, wire bytes and Wasm runtime code are unaffected.

Both standalone probes use identical [locked dependencies](Cargo.lock),
[compiler](compiler.txt), host and opt-level 3 / LTO / one codegen unit.
The inputs are one record with 1, 100 or 1,000 u64 fields (clean controls), or
that many ordinary list<u8> fields (each produces a warning). No methods or
acknowledgements are supplied. The report is consumed through black_box without
formatting or terminal IO; actual printing is additional consumer-owned work.

Each of three paired runs uses 30 warmups, calibrated batches of at least 10 ms
(up to 2,048 repetitions), and 11 samples per case. Run two reverses execution
order. Both variants are built before timing; no task-owned compiler was active.
These are local wall-clock observations, not a portable timing guarantee.

| Record fields | Baseline ns | Candidate ns | Change |
|---|---:|---:|---:|
| 1 × u64 | 679 | 938 | +38.2% |
| 100 × u64 | 22735 | 34864 | +53.3% |
| 1000 × u64 | 314908 | 388775 | +23.5% |
| 1 × list<u8> | 742 | 1166 | +57.1% |
| 100 × list<u8> | 29860 | 59273 | +98.5% |
| 1000 × list<u8> | 349067 | 675742 | +93.6% |

Values are pooled medians of 33 samples. [Summary](summary.json) also records
per-run medians; the six CSVs retain all individual samples and repetition counts.
Automatic analysis adds measurable host work, especially when allocating many
warning paths. It is accepted for default visibility: the largest measured case
still takes under 0.7 ms per generation. This is a regression in host generation
time, not a speedup. No runtime performance or Wasm savings are claimed.

All six generated TypeScript outputs compare byte-for-byte across variants in
each paired run. Tests separately verify acknowledgement/source independence.
To reproduce from a checkout containing the baseline commit and cached Cargo
dependencies, run:

```sh
bash docs/evidence/default-bulk-advice-0711/reproduce.sh
```

The [runner](reproduce.sh) reconstructs isolated sources and leaves samples and
generated outputs in its printed temporary directory. The [probe](probe.rs)
contains complete input construction and sampling. No release state is changed.
