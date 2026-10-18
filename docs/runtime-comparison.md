# Production runtime versus Candid

This comparison is retired. Its control implementation and `evaluate-runtime`
command were removed after codec selection; current integration tests use Wire
only. Measurements and source identities below are historical evidence, not a
claim that every schema or end-to-end request is ten times faster. Reproduction
requires the original source identified by the report and repository history.

Measured locally on 2026-10-16 using the released 0.2.0 runtime and the new
`runtime-benchmark` fixture. [Raw evidence](evidence/runtime-comparison.json)
records all samples, source/input/artifact SHA-256 hashes, compiler, flags,
PocketIC identity, decoder parameters and exports. Historical private Postcard
reports remain unchanged; their larger speedups do not describe this runtime.

## Scalar and SQL-shaped extension (pending 0.2.3)

The [expanded installed report](evidence/runtime-shapes.json) measures the released
0.2.2 runtime with three endpoints linked in each matched artifact: the original
mixed record, a standalone u64, and a bounded heterogeneous page. This is a new
artifact matrix; compare codecs within this run, not its Wasm sizes against the
older one-endpoint builds below. All earlier evidence remains unchanged.

The scalar method admits just 21 wire bytes, 21 allocation bytes, one node and
depth one. Its handler wraps an increment; cases cover zero, the 127/128 varint
boundary and u64::MAX wrapping to zero. The Candid control is a record carrying
method, direction and u64 value. Its transport allowance remains 5120 bytes for
Candid headers; quota and canonical checks remain as documented below.

The page echoes a reply-shaped value, incrementing a synthetic page counter.
It permits eight columns of at most 32 UTF-8 bytes and eight rows of eight cells.
Each nonempty row covers null, bool, near-minimum i64, near-maximum u64, finite
f32/f64 (including negative zero), 32 UTF-8 text bytes and 32 opaque bytes. Wire
uses explicit cell tags 1–8. Page allowances are 5120 frame bytes, 16384 allocation
bytes, 256 nodes and depth eight. Candid uses ordinary record/variant projections;
page length and finite-float checks occur after quota-limited decoding. There is
no DTO conversion inside either measured region. This is an update-based echo
for measuring both page decoding and encoding, not query latency or SQL execution.

Internal-only median strict-decode plus response-encode instruction counts:

| Input | Request bytes Candid / Wire | Reply bytes Candid / Wire | Instructions Candid / Wire | Candid / Wire |
| --- | ---: | ---: | ---: | ---: |
| Scalar 0 | 43 / 12 | 43 / 12 | 172,849 / 2,804 | 61.64× |
| Scalar 127 | 43 / 12 | 43 / 13 | 172,849 / 2,838 | 60.91× |
| Scalar u64::MAX | 43 / 21 | 43 / 12 | 172,849 / 3,947 | 43.79× |
| Page, zero rows | 155 / 50 | 155 / 50 | 783,418 / 23,421 | 33.45× |
| Page, one row | 259 / 158 | 259 / 158 | 850,108 / 45,879 | 18.53× |
| Page, eight rows | 987 / 914 | 987 / 914 | 1,332,805 / 200,669 | 6.64× |

Three sequential samples per case include the first call. Counter overhead is
included, handler work is separate, and argument copying/reply delivery are outside
the measured region. The management-enabled ratios are 43.77–61.09× for scalars
and 6.64–33.40× for pages. These observations are not confidence intervals.

| Three-endpoint artifact | Complete Wasm Candid / Wire | Reduction | Code-section payload Candid / Wire |
| --- | ---: | ---: | ---: |
| Internal-only | 814,894 / 124,739 bytes | 84.69% | 421,282 / 57,377 bytes |
| Also real management raw_rand | 851,614 / 720,114 bytes | 15.44% | 437,263 / 357,682 bytes |

The expanded run passes 144 measured calls, 248 rejected calls and two real
management calls. Rejections cover truncation, oversize, trailing data, wrong
identity/direction and page row/cell overflow. The runner checks matching exports
and byte-identical independently encoded expected replies for each call. The
original six mixed-record cases are retained in the raw report. Native time,
compression, peak memory, recursive values and real IcyDB contract adoption remain
unmeasured. The reproduction commands below apply to the original checkout.

## Scratch optimization (0.2.2)

A matched rerun against released 0.2.1 measures the pending runtime optimization.
[Before evidence](evidence/runtime-scratch-before.json) and
[after evidence](evidence/runtime-scratch-after.json) retain source and artifact
identities. Compiler, flags, dependency lock, inputs and decoder parameters match;
all request/reply hashes are identical and Candid instruction counts are unchanged.
The incoming local lockfile update (schannel 0.1.30) was preserved and used on both
sides. It is separate from this runtime change.

Encoding still reserves its admitted maximum fallibly, but initializes only bytes
actually emitted. Canonical validation now compares serialized chunks directly
against the input instead of allocating and initializing another maximum-sized
buffer. Both paths enforce the same byte bound; admission stays conservative over
the full schema. No unsafe code, wire layout, schema allowance or public API changes
are introduced. Custom serializer failures remain Malformed; canonical mismatch
remains Noncanonical.

Internal-only median strict-decode plus encode instruction counts:

| Input | Before | After | Before / after | Candid / after |
| --- | ---: | ---: | ---: | ---: |
| Empty collections | 166,489 | 7,865 | 21.17× | 52.61× |
| Blob 128 bytes | 171,021 | 12,169 | 14.05× | 34.46× |
| Blob 4096 bytes | 286,087 | 123,267 | 2.32× | 4.42× |
| UTF-8 text 256 bytes | 180,891 | 21,911 | 8.26× | 19.69× |
| 64 finite f64 | 201,259 | 62,667 | 3.21× | 7.07× |
| All collection bounds | 335,259 | 192,115 | 1.75× | 3.04× |

This trades code size for instruction savings. Internal Wire Wasm increases from
87,890 to 92,612 bytes (+4,722), and management-enabled Wire Wasm from 685,480 to
689,115 bytes (+3,635). Candid remains 674,527 and 749,534 bytes respectively.
Wire is therefore 86.27% smaller for internal-only code and 8.06% smaller with
management Candid retained. Payload lengths are unchanged from the table below.
These remain synthetic mixed-record results, not standalone scalar or consumer
measurements. This measures the combined optimization, not isolated attribution
to each change. No peak-memory or end-to-end latency claim is established.

Both runs pass the same 72 measured calls, 120 rejection checks and two management
calls. The optimized runtime also passes 12 runtime tests with the TypeScript
feature, including custom-serializer overflow/failure and canonical length/value
mismatch cases, six doctests and all 83 Rust/TypeScript vectors. Focused clippy,
Rust 1.91 Wasm compilation and the disposable formatting fixture pass.
The installed replay/rollback trial also passes with the optimized runtime.
The original `make evaluate-runtime` command reproduced that implementation;
all reports here remain historical.

## Original 0.2.0 results

Each row uses the same mixed record schema: revision u64, Bytes<4096>, Text<256>,
List<FiniteF64,64> and Option<u64>. The handler increments revision 42 to 43.
Inputs use deterministic bytes, repeated UTF-8 `é`, finite multiples of -0.125
(including negative zero), and correlation Some(7). Empty collections are **not**
a standalone scalar contract: their full schema maxima still apply.

Instructions below are the median of three per-call sums of strict decode plus
response encode, excluding the handler (207 instructions in every sample).
Request and response lengths are equal for these inputs. Counts are IC
instructions, not native nanoseconds or end-to-end latency.

| Input | Bytes Candid / Wire | Instructions Candid / Wire | Candid / Wire instructions |
| --- | ---: | ---: | ---: |
| Empty collections | 85 / 17 | 413,742 / 166,489 | 2.49× |
| Blob 128 bytes | 214 / 146 | 419,350 / 171,021 | 2.45× |
| Blob 4096 bytes | 4182 / 4114 | 544,615 / 286,087 | 1.90× |
| UTF-8 text 256 bytes | 342 / 274 | 431,481 / 180,891 | 2.39× |
| 64 finite f64 | 597 / 529 | 443,171 / 201,259 | 2.20× |
| All collection bounds | 4951 / 4883 | 584,527 / 335,259 | 1.74× |

These are internal-only artifacts. With management traffic retained, the same
ratios range from 1.74× to 2.48×; the raw report retains separate samples.
Wire saves 68 bytes per request or response in this schema: 80% for empty
collections, but only 1.37% at all bounds. Floats remain eight bytes each in
both encodings; blobs likewise dominate larger messages.

| Linked functionality | Complete Wasm Candid / Wire | Reduction | Code-section payload Candid / Wire |
| --- | ---: | ---: | ---: |
| Owned endpoint only | 674,527 / 87,890 bytes | 86.97% | 335,261 / 41,160 bytes |
| Also real management raw_rand | 749,534 / 685,480 bytes | 8.55% | 379,472 / 342,671 bytes |

These are full linked Wasm sizes, without postprocessing or compression. Both
management artifacts successfully call `raw_rand`; that call is outside the
measured region. There is no production Canic/IcyDB saving claim.

## What is being compared

Wire uses production `decode_request` and `encode_response`, explicit field IDs,
the 11-byte envelope, schema admission, bounded allocation, finite-float checks
and canonical re-encoding. Its limits are 5120 frame bytes, 16384 allocation
bytes, 128 nodes and depth 8. Canonical checking and response encoding each
allocate scratch for the schema maximum even for short actual values. Source
inspection identifies this as an optimization candidate; this experiment does
not isolate its instruction cost or establish a causal savings estimate.

The Candid control uses a separate ordinary Candid record with the same logical
fields plus method/direction identity. No DTO conversion occurs inside either
measured region. It uses the existing strict Candid decoder (quota 100000,
skipping quota 1000, maximum 64 types and 1024 header bytes), complete consumption,
identity validation and canonical re-encoding. Blob decoding checks 4096 before
copying; text length, float count and finiteness are checked after decoding.
Candid quotas and Wire's preallocation admission are different guarantees.
This is a strict canonical Candid control, not a default permissive CDK decoder.

Both adapters reject frames over 5120 bytes before copying arguments. Counters
start after argument copying and stop before reply delivery. Counter overhead
is included; scheduling, full-call costs, native timing, browser performance,
peak memory and compressed deployment size are unmeasured. Three sequential
samples include the first call; no warm-up is discarded or statistical confidence
interval inferred. All raw samples are retained.

The runner checks identical endpoint exports, installed round trips, independently
encoded expected replies and exact repeated bytes. It runs 72 successful measured
calls and 120 rejected calls (oversize, truncation, trailing data, wrong direction
and wrong identity), plus two management calls. This is a stateless benchmark,
not retry deduplication or authentication evidence. SQL unions, recursive values,
f32, standalone scalar schemas and actual consumer contracts are not measured.

## Historical reproduction

Use the prepared Rust/Wasm and PocketIC dependencies described in
[development](development.md). No tool installation happens implicitly:

```sh
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic make evaluate-runtime
```

The command builds four artifacts under `target/runtime-comparison`, checks them
in local PocketIC, and writes `report.json` only after all assertions pass.
It uses the locked dependencies (Postcard 1.1.3, Candid 0.10.38), Rust 1.99.0,
`wasm32-unknown-unknown`, and the shared `canister-evaluation` profile: size
optimization, LTO, one codegen unit, abort panic and stripped debug information.
The saved run is Linux/PocketIC 16.0.0; macOS execution is not established by it.
Runtime source and package versions are unchanged by this benchmark batch.

The later runner used IC Build 0.12 to select and retain each artifact
through measurement. Fresh reports include build/input/validation digests and
verify Wasm hashes against the retained build records. Keep sources unchanged
until the command exits; persistent mutation refuses report finalization.
Saved reports above retain their original schemas and source identities.
