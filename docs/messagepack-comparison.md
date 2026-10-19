# MessagePack versus Postcard: Blob-shaped codec experiment

Measured on 2026-10-19 at the maintainer's request. This is an experimental
comparison, not a new production codec mode. The fixture and native driver live
in the unpublished evaluation package; no platform SDK or simulator dependency
was added to the workspace.

## Decision and measured results

**Keep Postcard for Wire and the current Blob-shaped contract.** The current hash
array representation is substantially smaller and cheaper with Postcard. Small
RPCs also favor Postcard. Opaque binary bulk data provides essentially no raw-codec
reason to switch. This does not prove one codec is best for every workload:
MessagePack wins on wide random integer vectors under speed optimization.

Speed-optimized results below are medians; ns are native time, IC counts are a
separate measurement. All sizes are bytes. Array denotes compact MessagePack.

| Workload | Codec | Payload bytes | Encode ns | Decode ns | Encode IC instructions | Decode IC instructions |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Small RPC | Postcard | 5 | 28 | 13 | 1,014 | 456 |
| Small RPC | MessagePack array | 24 | 42 | 19 | 1,488 | 906 |
| 1,024 array hashes | Postcard | 32,986 | 14,219 | 19,647 | 1,060,746 | 641,382 |
| 1,024 array hashes | MessagePack array | 52,461 | 145,254 | 161,536 | 3,983,562 | 5,820,814 |
| 1,024 binary hashes* | Postcard | 34,011 | 7,459 | 23,544 | 368,107 | 667,173 |
| 1,024 binary hashes* | MessagePack array | 35,037 | 7,522 | 28,269 | 330,634 | 750,447 |
| 1,024 random u64 | Postcard | 9,732 | 7,340 | 4,054 | 335,344 | 320,520 |
| 1,024 random u64 | MessagePack array | 9,219 | 4,252 | 2,808 | 198,730 | 210,405 |
| 1 MiB binary | Postcard | 1,048,579 | 17,420 | 17,176 | 1,049,922 | 1,049,132 |
| 1 MiB binary | MessagePack array | 1,048,581 | 17,420 | 17,231 | 1,049,801 | 1,049,171 |

*Binary hashes are a separate schema experiment. Their Postcard manifest is
34,011 bytes versus 32,986 for array hashes because each binary hash carries a
length. They reduce codec work, but also change allocations and frozen bytes;
this experiment authorizes no schema change.

For 1,024 ordinary array hashes, speed-optimized Postcard produces **37.1% fewer
bytes**, takes **10.2× less native encode time / 8.2× less native decode time**, and
uses **3.76× fewer encode instructions / 9.08× fewer decode instructions** than
compact MessagePack. Under size optimization, the IC advantages are 13.7× encode
and 4.44× decode; the ranking holds, but ratios depend strongly on the profile.

MessagePack's random-u64 vector is 5.3% smaller and uses 40.7% fewer encode /
34.4% fewer decode instructions under speed optimization. Binary-hash manifests
use 10.2% fewer MessagePack encode instructions, but 12.5% more decode instructions
and 3.0% more bytes. Small u128 values favor Postcard; wide u128 values benefit
from MessagePack's fixed binary representation. Do not average unrelated workloads
into an assumed production traffic mix.

Named MessagePack maps add substantial overhead on small RPCs: 51 bytes versus
24 in array mode and 5 in Postcard. At 1,024 array hashes the named-map payload is
52,577 bytes versus 52,461 in array mode; the large hash arrays dominate. Maps
can supply field names, but that is not Wire's reviewed identity/canonical contract.

The 1 MiB raw-byte results are effectively tied at about 17 microseconds per
native encode/decode and 1.05 million IC instructions per operation. Postcard
adds 3 payload bytes and MessagePack adds 5. The observed tiny timing differences
are not a useful codec selection signal.

### Linked Wasm and gzip

These are complete **benchmark probes**, not projected production Blob sizes.
Both size and speed profiles retain the same 27 workload dispatches.

| Profile | Probe | Linked Wasm bytes | gzip bytes |
| --- | --- | ---: | ---: |
| evaluation | empty | 19,901 | 7,606 |
| evaluation | postcard | 92,179 | 28,866 |
| evaluation | messagepack-array | 341,462 | 77,252 |
| evaluation | messagepack-map | 341,480 | 77,224 |
| evaluation | wire-native | 121,640 | 34,584 |
| speed | empty | 20,800 | 7,804 |
| speed | postcard | 115,407 | 33,371 |
| speed | messagepack-array | 357,133 | 82,937 |
| speed | messagepack-map | 357,565 | 83,044 |
| speed | wire-native | 149,529 | 42,928 |

Size-optimized raw Postcard is about 3.7× smaller linked and 2.7× smaller gzipped
than compact MessagePack for this probe. Speed optimization increases both
artifacts; Postcard still retains the size advantage. A whole-Blob rebuild is
required before making a deployment-size savings claim.

### Bounded Wire control

Wire adds its 11-byte frame and performs admission/canonical verification. With
speed optimization, the 1,024-array-hash Wire manifest is 32,997 bytes and uses
3,778,613 encode / 6,804,815 decode instructions. Raw Postcard uses 1,060,746 /
641,383. The 1 MiB Wire control uses about 2.75 million decode instructions versus
1.05 million raw. These gaps locate contract costs; they do not show that a raw
MessagePack replacement supplies equivalent safety or retry semantics.

The measured next optimization candidate is byte/hash traversal within the
existing bounded Postcard contract. Qualify any implementation against existing
bytes, allocation admission and canonical checks. Changing hash representation
or replacing the encoding requires a separately approved hard cut.

## Method

27 deterministic workloads × four lanes × two optimization profiles:
raw Postcard, compact MessagePack arrays, named MessagePack maps, and production
Wire native. The Wire control includes framing, resource admission and canonical
verification; its cost is not an apples-to-apples raw-codec comparison.

Workloads cover small/wide RPC IDs, small/random/max 64-bit integers, signed
integers, small/wide 128-bit IDs, ASCII/UTF-8 metadata, manifests with 1/32/1024
hashes, binary buffers from zero through 1 MiB, a plain `Vec<u8>` control, and
success/error results. Manifests reproduce Blob's major field types and value
ranges in a flat consumer-shaped fixture, **not the actual nested consumer DTO**.
Principals are represented by binary byte buffers. No hashing, storage, admission
business logic, authentication, network transport or HTTP streaming is timed.
The 1 MiB control does not establish Blob upload throughput.

Each workload is constructed before the clock starts. Encode allocates a fresh
output; decode produces an owned value. Timings include allocation, destruction,
loop/dispatch and clock overhead. Exact value roundtrips and byte identity checks
run outside the measured intervals. No borrowed decode or reused-buffer claim.
The input seed uses explicit 64-bit operations and little-endian words, so native
and Wasm fixtures match. Every result retains an encoded-input SHA-256.

Native: Linux x86_64, AMD Ryzen Threadripper 7970X, logical CPU 8 affinity,
frequency boost enabled, no exclusive CPU reservation. Each lane calibrates to
at least 3 ms for its faster operation, capped at 1,048,576 operations per batch;
15 batches rotate lane order. Reported statistics are median/p10/p90 per-operation
nanoseconds; intervals describe batch variation, not population confidence bounds.
Inputs are warm and allocations repeat, so these are steady-state microbenchmarks.
Native timing noise and other host activity remain possible.

IC: PocketIC 16.1.0, independently installed minimal query probes, five calls per
case, four operations per timed batch, `ic0.performance_counter(0)`. The harness
constructs the fixture before the counter and returns the sample afterward using
a fixed binary control response. Returned input hashes match native. These are
IC codec instruction counts, not query wall time, cycle bills or end-to-end cost.
The simulator/host runner is an external disposable experiment; it is not part of
the maintained workspace or its CI. The source and locks are retained locally
with the experiment artifacts.

Compiler: Rust 1.99.1 (`58cf8f9c3`, LLVM 23.1.1), `wasm32-unknown-unknown` and
`x86_64-unknown-linux-gnu`. Both profiles use LTO, one codegen unit, aborting panics,
and stripped debug information. `evaluation` uses opt-level `z`; `speed` changes
only opt-level to `3`. No target-cpu/RUSTFLAGS override or wasm-opt pass.
Codec selections: Postcard 1.1.3 (`alloc`, defaults disabled), rmp-serde 1.3.1
(defaults; `to_vec` versus `to_vec_named`), rmp 0.8.15, Serde 1.0.229 and
serde_bytes 0.11.19. Named mode changes struct fields to maps, not every tuple.
Wire uses the task-start working tree based on 0.4.5, including the incoming Wasm
canonical byte-comparison optimization, with limits 2 MiB bytes, 64 MiB allocation
allowance, 262,144 nodes and depth 32. Allowances are not measured peak heap usage.

Wasm sizes are complete linked probe files and deterministic `gzip -n -9` files.
Each raw codec links independently, with separate features for array/map encoding.
An empty query harness gives context but is not subtracted as a precise codec
contribution: monomorphization, error paths and linking interact. Payload gzip is
not measured, because RPC payloads are not automatically gzip-compressed.

## Representation and contract caveats

Ordinary `[u8; 32]` hashes stay ordinary arrays in the primary manifest cases.
The separately named binary-hash cases use `ByteBuf`; that changes the schema
and allocation pattern. MessagePack arrays need a marker for every byte; values
above 127 need an extra byte. Postcard's u8 elements need one byte. Explicit binary
storage avoids this MessagePack overhead, but should not silently replace the
consumer's frozen hash representation.

rmp-serde encodes every `u128`, even a small one, as a 16-byte big-endian binary
value with a two-byte header. The Rust roundtrips pass, but interoperating clients
must agree on this convention. MessagePack's native integer domain stops at
64 bits. See the [MessagePack specification](https://github.com/msgpack/msgpack/blob/master/spec.md)
and [rmp-serde serializer source](https://docs.rs/rmp-serde/1.3.1/src/rmp_serde/encode.rs.html).
Postcard uses varints for integers wider than a byte; see the
[Postcard specification](https://postcard.rs/spec-c68ec81.pdf).

The tested raw MessagePack decoder accepts both shortest and longer integer
representations, and ignores trailing input. Focused tests record these facts.
It does not implement Wire's byte/allocation/node budgets or canonical retry/hash
contract. Raw Postcard here checks full consumption but also lacks Wire admission
and canonical verification. A hypothetical bounded MessagePack implementation
would add work and needs separate qualification. No such implementation was built.

## Reproduction and evidence

`make evaluate-messagepack` runs the maintained size-optimized native comparison
and refuses an existing output directory. Run it on an otherwise quiet host for
fresh timings. The complete commands, raw samples, per-operation CSV and identities
are in [the evidence directory](evidence/messagepack-2026-10-19/metadata.json).
Native profiles and IC reports retain all measurements, not just the rows selected
below. Local binaries, external harness source/locks and logs are retained at
`target/messagepack-2026-10-19/`; the harness paths in its source archive can be
adjusted for another checkout. Retain that archive when transferring this experiment:
the raw JSON in Git alone is not a standalone PocketIC reproduction kit.

Focused roundtrips (108 case/lane pairs), representation/acceptance probes,
strict all-target/all-feature Clippy, Rust 1.91 host checking, dependency boundaries
and formatting are checked. No full workspace CI or native macOS timing is claimed.
