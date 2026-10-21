# Layout and canonical-decoding study

This study compares consumer schema choices and profiles finite Rust decoding
against 0.5.15 (`39535ba`). The selected runtime change extends the existing
same-input-region comparison shortcut from Wasm/tests to native hosts. Protocol
bytes, method identity, admission limits and malformed-input acceptance are
unchanged. The generator changes released in 0.5.16 do not affect these timings.

## Consumer layout experiments

These are hypothetical consumer contracts, not changes to blob-storage or the
ic-wire protocol. The probe gives each incompatible layout a distinct method ID.
The following byte examples describe payloads after the unchanged 11-byte header:

| Experiment | Before | After | Frame bytes | Consequence |
| --- | --- | --- | --- | --- |
| 32-byte key | `Bytes<32>`: `20` then 32 bytes | `[u8;32]`: just 32 bytes | 44 → 43 | Only exactly 32 bytes remain valid |
| Unit enum tag | ID 16384: `80 80 01` | ID 1: `01` | 14 → 12 | Explicit variant identity changes |
| Present key | `Option<[u8;32]>`: `01` then key | Mandatory `[u8;32]`: just key | 44 → 43 | Absent value is no longer representable |

These changes require coordinated consumer updates and new incompatible contract
identities. Old request bytes must not be retried under the new identity. No
migration, compatibility lane or automatic schema rewrite is proposed. Choose
small tags when designing new contracts; do not casually renumber published tags.
Transparent newtypes do not add Postcard bytes, so deleting them is not a wire-size
optimization. Tighter collection bounds constrain admission rather than inserting
a smaller bound into each serialized value.

Small byte savings are not sufficient evidence for adoption. The two confirmation
runs found fixed-key decode approximately 24–25% slower than bounded bytes,
despite saving one byte. Removing a presence byte changes API semantics; it is
appropriate only when the consumer actually requires the field. See the retained
samples for encode/decode tradeoffs and small-value timing variation.

## Canonical verification

The probe measures fresh encoding, raw Postcard decode, complete Wire decode,
and canonical serialization/comparison of a previously decoded value separately.
Raw Postcard is a diagnostic control, not a deployable replacement: it omits
Wire's header, admission and canonical checks. Subtracting raw time from full
time does not isolate canonical cost exactly, and stage medians are not additive.

A nonminimal zero (`80 00` rather than `00`) is accepted by raw Postcard but
rejected by Wire as `Noncanonical`; the runner asserts this. Native generic
contracts also need to reject maps normalized by deserialization, including
unordered/duplicate keys. Checking varints alone cannot replace the current
canonical verification for arbitrary consumer Serde implementations.

The confirmation profile puts canonical-only time for 1,024 u32::MAX values around
8.2–8.4 microseconds, against 10.5–10.9 microseconds for complete decode. A future
single-pass decoder would need equivalent typed errors, map/custom-Serde semantics
and differential malformed-input qualification. It is not implemented here.

One safe improvement is retained: when canonical serialization references the
exact input slice, equal length plus equal start pointer establishes equality.
The production comparator already used this on Wasm. Native builds now use it
too, after the same allowance and length checks. Different storage still receives
exact content comparison; no hashes, validation bypass or new mode are introduced.
Owned data still needs allocation/copying and content validation. Borrowed data
requires the input frame to outlive the decoded value. Native timing gains do not
imply additional Wasm or IC instruction gains because Wasm already had this path.
The timed operation does not read/hash/process the returned blob; doing so still
costs work proportional to its length.

## Retained native change: measured results

[Median results](evidence/protocol-study/native.csv) and
[all confirmation samples](evidence/protocol-study/samples.csv) record paired
baseline/candidate runs from the maintained runner:

| Complete decode | Baseline, runs 1 / 2 | Candidate, runs 1 / 2 | Change |
| --- | --- | --- | --- |
| Borrowed 4 KiB blob | 38.1 / 37.9 ns | 13.3 / 13.4 ns | about 65% faster |
| Borrowed 1 MiB blob | 9,822 / 9,720 ns | 14.0 / 13.7 ns | about 99.9% faster |
| Owned 1 MiB blob | 34,565 / 35,237 ns | 34,440 / 34,105 ns | 0–3% faster |
| 1,024 u32::MAX values | 10,519 / 10,908 ns | 11,010 / 10,780 ns | 5% slower / 1% faster |

Other small complete-decode controls range from 6% slower to 3% faster, apart from
one noisy borrowed-32-byte baseline (15 ns versus 9 ns on repetition). The large
borrowed gain is repeatable; no broad owned/integer speedup is claimed. Existing
native malformed-map checks, borrowed buffer checks, runtime tests and the focused
TypeScript gate pass. The full gate, hosted macOS and installed consumers were not
run. The eleven-byte frame and all decoder rejection rules remain unchanged.

## Reproduction and scope

Run the maintained [probe](../scripts/benchmarks/protocol/mod.rs) through its
[isolated runner](../scripts/benchmarks/protocol/run.sh), with a new output path:

```sh
taskset -c 0 bash scripts/benchmarks/protocol/run.sh /tmp/wire-protocol-run 39535ba
```

`taskset` is the Linux affinity setting used here; omit it on hosts without it
and record that difference. The runner snapshots the baseline in a disposable
checkout, uses the current lockfile in both builds, then changes only the contract
implementation for the candidate. A benchmark-only helper exposes the production
comparator in that snapshot; it is never added to the maintained library API.
The helper uses the admitted payload length as its serialization allowance.
All timed inputs are canonical, so this is sufficient for the stage measurement.

Each case warms 100 operations, calibrates to at least 10 ms (65,536-call cap),
and records 11 batches. Small cases can hit the cap before 10 ms. Two runs reverse
baseline/candidate process order. Setup, frame writes and correctness assertions
are outside timing; allocation and destruction are included. Canonical-only
measurements reuse a prepared value. Full decode black-boxes and drops its result.
The probe also checks canonical re-encoding and compares frames between builds.
Raw CSV columns are case, stage, frame bytes, batch, repetitions and ns/operation.

Inputs: keys/blobs contain byte 42; blobs have lengths 0/32/4096/1048576 and a
1 MiB schema bound. Integer lists contain 1,024 u32::MAX values. Unit enum values
use the tags above. Frame/allocation/node/depth allowances are 2 MiB/8 MiB/4096/32;
method IDs are 1–9 as specified by the probe, shared by owned/borrowed blob cases.
The fixed layouts have different node/heap shapes; these are deliberate schema
tradeoffs. Borrowed and owned blobs produce identical bytes and share identity.

Environment: Linux x86_64, AMD Threadripper 7970X, Rust 1.99.1 (`58cf8f9c3`, LLVM
23.1.1), default release profile (opt-level 3, no LTO), CPU 0 affinity, no Rust flag
overrides. No IC, Wasm size, browser, macOS or application-throughput claim is made.
The shared dependency lock SHA-256 is
`5662d8e01c56e0dcebab91167976f6163bf543fe89bbf620bd065d893832a27c`.
Probe SHA-256 is `8a3e6fcd1cf09fd2f127ee89ecd3cfd8b5e454b9b1321ee15db54a7d74b141dd`.
Candidate contract SHA-256 is
`694e6027a4dc1395276b2668b53e55049f1442b13cf32ed8567f9a23f0330c7b`.
Exploratory builds, three baseline/candidate runs and frame files remain under
`/tmp/ic-wire-protocol-study/`; the maintained-runner confirmation is under
`/tmp/ic-wire-protocol-confirm/`.

The follow-up [native comparison experiments](native-comparison-study.md) reject
word-reduction and early-exit chunk alternatives for owned blobs. The runner's
optional third argument selects an experimental contract file outside production.

The [bulk-reservation study](bulk-reservation-study.md) extends the probe with
threshold-sized blobs, chunk lists and returned capacity files. Earlier results
use the probe hash recorded above (source in revision `24c1250`); use that probe
version for exact reproduction rather than attributing newer measurements to it.

The [native borrowed-view study](native-views-study.md) adds public native API
measurements with a full byte scan and multi-field text projections. Select it
with `IC_WIRE_NATIVE_VIEWS_ONLY=1`; its source identities and evidence are retained
separately from the original finite-codec measurements above.
