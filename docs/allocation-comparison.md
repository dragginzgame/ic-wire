# Borrowing, buffers and bounded layouts

This report records the 0.4.6-era experiment. Finite borrowing has subsequently
been implemented for 0.5.0; see the [current runtime guide](../crates/ic-wire/README.md).

Measured 2026-10-19 against released 0.4.6 plus the native API additions in this
working tree. Production remains Postcard. Hashes are consumer data, not a Wire
protocol requirement. No rkyv or alternative production codec is introduced.

## Selected changes

Native decode now accepts `Deserialize<'de>`: byte/string views may borrow the
input frame. Canonical validation and conservative admission remain mandatory;
this is not unchecked access. Finite `Wire` remains owned. Native
`encode_into`, `encode_request_into` and `encode_response_into` accept reusable
caller storage. Existing fresh-buffer APIs and frame bytes remain compatible.
See the [library examples](../crates/ic-wire/README.md).

Packed hashes are a consumer schema experiment, not a runtime special case.
For Blob adoption, prefer a contiguous byte field with a maximum length and
length divisible by 32. The compiled projection test encodes a borrowed
`Vec<[u8; 32]>::as_flattened()` view and reads the decoded slice with
`as_chunks::<32>()`, without a staging copy. A schema change requires new method
identities and frozen consumer vectors, coordinated producer/consumer deployment,
and review of any canonical hashing or persisted representation. No dual reader,
migration shim or Blob repository change is supplied here.

## Results

Representative speed-profile results. Native values are median nanoseconds per
operation; IC values are instructions per operation. Decode includes a full
byte checksum scan and destruction of owned output, not just deserialization.

| Workload / path | Frame bytes | Native encode | Native decode + scan | IC encode | IC decode + scan |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1,024 hashes, native array | 32,881 | 69,713 | 100,292 | 3,155,509 | 6,792,652 |
| 1,024 hashes, finite array | 32,881 | 40,745 | 57,257 | 1,441,409 | 2,304,373 |
| 1,024 hashes, native packed | 32,882 | 721 | 4,735 | 40,718 | 344,560 |
| 1,024 hashes, finite packed | 32,882 | 632 | 4,605 | 36,624 | 339,349 |
| 1,024 hashes, borrowed packed | 32,882 | 722 | 3,923 | 40,718 | 309,764 |
| 1 MiB bytes, native owned | 1,048,590 | 20,374 | 145,591 | 1,369,782 | 10,880,204 |
| 1 MiB bytes, reusable output | 1,048,590 | 17,289 | 148,867 | 1,049,315 | 11,200,324 |
| 1 MiB bytes, borrowed decode | 1,048,590 | 20,208 | 121,912 | 1,369,782 | 9,831,299 |

Packed versus native array reduces IC encode/decode-plus-scan instructions by
98.7%/94.9%, at one extra byte. Finite array saves 54.3%/66.1% with identical
frames. Borrowing saves another 10.1% of packed decode-plus-scan and 9.6% for
1 MiB bytes. Borrowed lanes use owned input for encoding: their encode column
is not a borrowing optimization. Both profiles and every case appear in the
[complete summary](evidence/allocation-2026-10-19/summary.csv).

Finite encoding reserves the schema maximum: an empty finite blob frame is
12 bytes but its output capacity is 1,048,597 bytes. Native capacity is 12.
On IC at opt-3 that empty finite encode costs 40,640 instructions versus 938
native; native-host allocator behavior does not reproduce this penalty.
Use finite contracts for useful field bounds; avoid huge blanket maxima when
most messages are tiny. Native and finite valid frames match, but their
admission contracts differ: native traversal allowances are not per-field bounds.

Reuse moves some allocation costs between intervals. The apparent 23.4% saving
in 1 MiB IC encoding almost entirely reappears in subsequent owned decoding;
combined encode/decode-plus-scan saves only about 0.003%. Native combined medians
also show no improvement. Do not extrapolate an end-to-end speed win. Reuse is
useful for caller-controlled storage, but retains its high-water capacity: after
a 1 MiB frame, encoding an empty frame still holds 1,048,590 bytes. Successful
calls retain capacity; serialization failure can lose it. Output is empty on
error, and retry frames need separate immutable custody before reuse. Reported
capacity is output Vec capacity, not peak heap or total allocation volume.

## Method and reproduction

44 cases: manifests with 0/1/32/1,024 hashes across native array, finite array,
reused array, native packed, finite packed, reused packed and borrowed packed;
opaque bytes with 0/32/4,096/1,048,576 bytes across native, finite, reuse and borrow.
The simplified manifest contains upload/object IDs, root hash, byte count and
content type. Native DTOs use ordinary Vec/String/ByteBuf, finite DTOs bounded
List/Text/Bytes. Distinct identities separate array and packed layouts. Packed
shape rejection and same-layout equality have owning tests. Fixtures are not
actual Blob endpoint DTOs or an installed consumer qualification.

Native: Rust 1.99.1, LLVM 23.1.1, Linux x86_64, Threadripper 7970X, pinned CPU 8
without exclusive host ownership; eleven rotated batches, calibrated to at least
2 ms for the faster operation (cap 1,048,576 repetitions). Setup and frame checks
are outside clocks. Native uncertainty is visible in all retained raw samples.
IC: external PocketIC 16.1.0 probe, wasm32-unknown-unknown, five queries per case,
four repetitions, `ic0.performance_counter(0)` around shared fixture operations.
No SDK/simulator dependencies were added to this workspace. Postcard and Serde
versions and all dependencies are fixed by archived Cargo.lock files.

Profiles use LTO, one codegen unit, panic abort and stripped debug info;
`evaluation` uses opt-z, `speed` opt-3. The combined probe is 108,102/119,281 linked
Wasm bytes and 33,240/37,347 gzip-9 bytes respectively. These include every lane;
they do not establish individual API or layout size savings. Opt-3 improves many
instruction counts at increased code size; consumer deployment must qualify that
tradeoff on its complete binary.

Run `make evaluate-allocation` into its fresh output directory; the driver refuses
to overwrite an existing run. For opt-3 use the same evaluation command with
`CARGO_PROFILE_EVALUATION_OPT_LEVEL=3` and a fresh directory/profile label `speed`.
Raw [native speed](evidence/allocation-2026-10-19/native-speed.json),
[native size](evidence/allocation-2026-10-19/native-evaluation.json),
[IC speed](evidence/allocation-2026-10-19/ic-speed.json),
[IC size](evidence/allocation-2026-10-19/ic-evaluation.json) and
[identities](evidence/allocation-2026-10-19/identity.json) retain parameters and
samples. All 44 frame hashes, lengths and capacities agree across native/IC and
both profiles. Exact source, external harness/lock archives, Wasms and logs are
retained locally under `target/allocation-2026-10-19/`. Harness path dependencies
must be redirected to the archived source for standalone reproduction.

Focused runtime/evaluation tests, strict Clippy, Rust 1.91 host/Wasm checks and
Rust/TypeScript conformance qualify the API additions. This is not full CI,
native macOS timing, end-to-end network latency or Blob deployment evidence.
