# Complexity cleanup qualification

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Work through ten proposed simplifications. Keep seven, narrowing packed sharing
and operation identifiers after measuring the alternatives. Keep one production
Serde/Postcard codec and all public APIs. **Protocol unchanged:** frame bytes,
method identities, Serde order, field bounds, allocation/node/depth charges,
malformed-input acceptance and retained retry/hash inputs are preserved.

| Candidate | Decision |
| --- | --- |
| Repeated conformance policy properties | Use existing `method_limits`; leave independent literal/adversarial oracles intact. |
| Shared lazy corpus collector | Reject. Retain local selection and specialized lane augmentation. |
| Boolean schema-derive context switches | Use four explicit attribute sites; emitted tokens and tested refusals match. |
| Triangular corpus exclusion lists | One constant primary-corpus count; retain representation/size checks separately. |
| Benchmark operation layout | Share plain numeric constants, names and reply offsets. Reject the enum/zip loop. |
| Repeated producer fingerprints | One host-only source inventory; hash its source and the operation module too. Keep the verifier independent. |
| Shared disposable-consumer setup | Decline. Different manifests, target reuse and proofs outweigh the few identical setup lines; no helper framework added. |
| Packed internals | Share the sealed LE encoder only. Reject shared bounded/unbounded decoding; retain pre-allocation checks and copy paths. |
| TypeScript output finalization | Reject. A small expression did not justify another method and noisy timing regressions. |
| Budget terminology | Name conservative charge units explicitly; preserve every debit and refusal. |

The private producer metadata adds `evidence_helper_sha256` and
`operations_sha256`. Completed reports without them require a fresh matched pair;
this tightens evidence admission, not the Wire protocol. No package/dependency
versions, snapshots, staging or delivery are changed by this batch.

## Inputs and method

Baseline is the frozen dirty working tree at
`fcfcf5b795213cb3a0cb5857781fae7eb48f33a3`, including earlier pending 0.15.1
cleanup and the already-selected Testkit 0.80.1 / Runtime 0.18.1 graph. Root
Cargo.toml and Cargo.lock are identical across the pair. Separate frozen sources
have independent target directories; changed copied Rust inputs were touched
before candidate rebuilds. Cargo messages record fresh guest builds and the
initial runtime rebuild. Native/IC samplers also retain baked source identities.
The disposable IC sampler is registered only in the isolated candidate workspace;
it uses the maintained fixtures/operation layout and measures Wire only. It is
qualification code, not another production codec or maintained package.

Rust/Cargo 1.99.1, x86_64 Linux; guest target wasm32-unknown-unknown. Existing root
profiles are held fixed: evaluation uses opt-z, speed uses opt-3, both LTO/one
codegen unit/abort/stripped symbols. The standalone native probe uses opt-3/LTO/
one codegen unit. Node 24.21.0 / TypeScript 7.0.2 qualify browser output. IC
Testkit owns the explicitly prepared PocketIC 16 server lifecycle. Source,
compiler identities, locks, Cargo artifacts, samples and exact commands are in
Retired raw output (see retention notice).

## Uncompressed Wasm and IC instructions

| Corpus | Profile | Total bytes, before → after | Code-section bytes, before → after |
| --- | --- | ---: | ---: |
| 20 cost controls | opt-z | 94,161 → 94,161 | 81,710 → 81,710 |
| 20 cost controls | opt-3 | 121,085 → 121,085 | 109,642 → 109,642 |
| 20 bounded-i16 controls | opt-z | 43,286 → 43,286 | 34,414 → 34,414 |
| 20 bounded-i16 controls | opt-3 | 48,472 → 48,472 | 39,750 → 39,750 |
| 109 bulk controls | opt-z | 98,118 → 98,118 | 86,965 → 86,965 |
| 109 bulk controls | opt-3 | 122,644 → 122,644 | 112,201 → 112,201 |

All six pairs retain **0 bytes / 0% change** in both metrics; complete module
hashes can differ. These are complete qualification fixtures, not application
savings or code-limit headroom. Compressed size is not a metric.

All **1,762 Wire operation pairs** are instruction-identical: 232 cost-control,
440 bounded-i16 and 1,090 bulk pairs, across both profiles. Each row has three
identical measurements from distinct callers; setup and reply transport are
excluded, result destruction and loop overhead included. Loop controls are
reported, not subtracted. Frame/input identities agree. This is a before/after
cleanup screen, not a new Candid comparison or endpoint cycle measurement.
Retired raw output (see retention notice) and Retired raw output (see retention notice) retain the results.

## Host and native timing

Nine derive controls use 11 alternating paired batches, eight warmups and
1,000 expansions per batch (100 for the 100-field record). Tokens agree for
all nine; four malformed declarations produce equal error messages. Median
changes range **−2.36% to +1.67%**. Examples, ns per expansion:

| Control | Before | After | Change |
| --- | ---: | ---: | ---: |
| Unit | 31,455 | 31,507 | +0.17% |
| Three-field record | 53,033 | 53,920 | +1.67% |
| Generic enum | 132,059 | 132,823 | +0.58% |
| 100-field record | 647,147 | 652,187 | +0.78% |

The native codec probe uses identical fixed values and limits, 11 alternating
process batches and 16 warmups per operation, with 0/1/128/16,384-word controls
and ordinary/scalar controls. All encoded files agree. Median changes range
−10.86% to +6.77%; retain the noise and regressions rather than claim speedups.
The largest percentage increase is empty Packed encode, **18.011 → 19.230 ns**
(+1.219 ns). Bounded 16,384-word decode measures **517.344 → 550.906 ns**
(+6.49%); that decoder is retained unchanged. The IC counts above are unchanged.
Retired raw output (see retention notice), Retired raw output (see retention notice) and
raw archived samples retain absolute results and ranges. This supports a
maintenance cleanup, not a native performance improvement claim.

## Rejected trials and checks

The initial collector/enum/packed-helper combination grows the bulk guest by
1,497 total / 1,477 code bytes at opt-z and 330 / 401 at opt-3. After removing
the collector and enum loop, shared decoding alone still grows affected packed
fixtures by 33–118 total bytes, so that decoder is removed too. Keep only the
encoder sharing; the final table above has no growth. Trial source and both
screens are retained in the archive; the first screen does not attribute the
combined growth to an individual candidate.

The rejected browser finalizer probe uses identical outputs, 11 alternating
batches, small/large blobs and framed/unframed controls. Medians range −46.71%
to +12.14%, including 607.739 → 681.503 ns for framed unit encode. This noisy
single-process probe is insufficient evidence of an improvement; no finalizer
change remains. Keep error/header boundaries explicit as well.

Focused checks pass: derive tests, core unit/packed tests and corpus roundtrips;
strict core/derive/benchmark lint; 52 benchmark CLI checks (including primary
corpus and size conflicts); and 28 cost-gate controls including the new source
identities. Rust–TypeScript byte/refusal/mutation checks and simulated endianness
pass. All 234 generated TypeScript source files match the retained earlier
conformance output, whose generator bytes were previously qualified unchanged.
Standalone Platform Types and combined Endpoint consumers pass native tests,
strict lint, host/Wasm graph checks and Principal/Account browser byte/refusal
checks. No full repository gate or release was run.

Reproduction: run recipes from a prepared Wire checkout. Unpack the archive
into a new writable output directory and adjust the scripts' `out` paths
(default `target/complexity-cleanup-0151`); never overwrite retained evidence.
Prepare the locked Rust/Node/server tools explicitly and use separate Cargo
target directories.
`build-pair.sh` records both guest profiles for each corpus; `build-native.sh`
then `run-native.mjs` produces paired native samples. The `derive/` harness
compiles against the recorded proc_macro2/quote/syn artifacts and checks token
parity before timing. `build-samplers.sh`, then `run-ic.sh` and
`summarize-ic.py` qualify the guest counters. Reconcile absolute artifact/output
paths in those retained recipes after relocation. The guest builds predate the
disposable sampler registration; its extra example does not participate in them.
Original artifacts remain under `target/complexity-cleanup-0151/`.
