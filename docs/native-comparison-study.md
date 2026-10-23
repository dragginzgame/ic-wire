# Native canonical-comparison experiments

Historical finite-codec research. Its runner was retired from the active tree in
0.6.4; the commands below apply to an isolated checkout of
`9a1c06e6829da053190d997cdeed92a861addedb` (0.5.22), with that revision’s
lockfile and finite sources. Retained results are not current-codec benchmarks.

Against released 0.5.16 (`24c1250`), two candidates changed the exact-content
comparison used when serialized bytes do not share the input's start pointer.
Both were rejected. The production comparator, protocol bytes, identity,
admission limits and canonical rejection rules remain unchanged by this batch.

- **Word reduction:** enable the existing Wasm/test comparison on native hosts.
  Buffers of at least 64 bytes use 16-byte XOR/OR reduction; shorter buffers of
  at least eight bytes use eight-byte reduction, with exact tail comparison.
- **Early exit:** also enable that path on native hosts, but compare 16-byte
  chunks using iterator `all`, stopping at the first unequal chunk. Smaller
  buffers retain the word-reduction implementation.

Both candidates preserve the same-input-region shortcut. These experiments
measure valid canonical inputs, so the early-exit candidate normally visits all
chunks. They do not establish malicious/mismatching-input timing behavior.

## Results

[All medians](evidence/native-comparison/native.csv) and
[raw samples](evidence/native-comparison/samples.csv) include encode, raw decode,
complete decode and canonical-only stages plus small/integer/borrowed controls.
The following are complete-decode medians from the first run; percentage ranges
include both repeated runs of each experiment.

| Candidate / owned blob | Baseline | Candidate | Repeated regression |
| --- | --- | --- | --- |
| Word reduction / 4 KiB | 101 ns | 240 ns | 137% |
| Word reduction / 1 MiB | 33.2 us | 61.2 us | 74–84% |
| Early exit / 4 KiB | 102 ns | 182 ns | 77–78% |
| Early exit / 1 MiB | 33.4 us | 45.6 us | 33–37% |

Word-reduction canonical-only time regresses 253–255% for 4 KiB and 158–166%
for 1 MiB. Early-exit borrowed and enum controls also regress, although their
successful path is logically unchanged; do not attribute those differences to
extra payload comparisons. No instruction-level cause or universal timing claim
is established. These are native host measurements, not IC/Wasm/browser results.
The existing native slice equality remains preferable for these owned workloads.

## Reproduction

The archived [protocol probe](https://github.com/dragginzgame/ic-wire/blob/9a1c06e6829da053190d997cdeed92a861addedb/scripts/benchmarks/protocol/mod.rs) and its
[runner](https://github.com/dragginzgame/ic-wire/blob/9a1c06e6829da053190d997cdeed92a861addedb/scripts/benchmarks/protocol/run.sh) are reused. The runner now accepts
an optional candidate contract file, so experiments need not edit production.
Reconstruct either candidate from the released comparator and its retained patch:

```sh
git show 24c1250:crates/ic-wire/src/contract/mod.rs > /tmp/wire-compare-base.rs
patch -o /tmp/wire-compare-word.rs /tmp/wire-compare-base.rs \
  < docs/evidence/native-comparison/word.patch
bash scripts/benchmarks/protocol/run.sh /tmp/new-word-study 24c1250 \
  /tmp/wire-compare-word.rs
```

Use `early.patch` for the second candidate. Patches are experiment inputs, not
production changes. Both experiments ran twice in baseline/candidate then
candidate/baseline process order. The runner compiled both variants, asserted
canonical roundtrips/nonminimal-integer rejection, and compared frame files.
The full malformed-input suites were not run against rejected candidates.

Inputs, stage meanings, warm-up (100 operations), calibration (10 ms or 65,536
operations), 11-batch medians, bounds and frame construction match the
[protocol study](protocol-study.md#reproduction-and-scope). Input frame hashes are
in the median CSV. Linux x86_64, AMD Threadripper 7970X, Rust 1.99.1, default
release profile (opt-level 3, no LTO), and no Rust flag overrides were used.
Unlike the earlier protocol study, CPU affinity was not fixed. Each experiment
ran alone; generation/build work completed before its timings. Compare paired
results here rather than absolute times across the two studies.

Both lanes used the same released dependency lock, SHA-256
`d13c18d9cb7ca6d86ad7f1c09c869d929180aaa4d19d77f7012b913bea04648a`.
The probe is unchanged, SHA-256
`8a3e6fcd1cf09fd2f127ee89ecd3cfd8b5e454b9b1321ee15db54a7d74b141dd`.
Candidate contract hashes:

- Word reduction: `f1fecf1c4fbbec60ab4cc6a646cef430a7c001932c1397e20d8e0b26ac9c727e`.
- Early exit: `0ea43045515eaf59df819ece65c552f306d37dcbd9016877c54e932ec7511b0a`.

Raw builds, frames and binaries remain in `/tmp/ic-wire-native-word/` and
`/tmp/ic-wire-native-early/`; candidate sources are in `/tmp/ic-wire-native-compare/`.
No package version, dependency, Git ref or released runtime code was changed.
