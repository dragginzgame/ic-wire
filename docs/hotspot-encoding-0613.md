# Ordinary-list encoding hot spots: 0.6.13

This follow-up targets the large losses in the [full matrix](full-comparison-0613.md).
The retained change writes i16 Postcard varints directly through the private
serializer adapter, including its sizing and canonical-comparison passes. One
production Serde/Postcard codec remains. No public API, bytes, framing, schema,
identity, limits, malformed-input acceptance, canonical hashing or retry rules change.
For example, 32767 still encodes as `fe ff 03`; -64 remains `7f`.

## Matched measurement

The baseline is the incoming working tree based on f470bcd9, with the previously
adopted IC Tooling 0.67.0 and Platform 0.6.14 lock, plus the identical unpublished
hotspot fixture/provenance changes on both sides. Workspace packages remain
0.6.12; no version, staging, commit or release is performed. The older full matrix
used Tooling 0.66.3/Platform 0.6.12; its figures are historical baseline evidence,
not substituted for this matched run.

Both sides use Rust 1.99.1/LLVM 23.1.1, Serde 1.0.229, Postcard 1.1.3 and Candid
0.10.39, with the same lock, wasm32-unknown-unknown, LTO, one codegen unit,
abort panics and symbol stripping. `evaluation` selects opt-z; `benchmark-speed`
selects opt-3. PocketIC 16.1.0 is explicitly prepared and owned by Testkit 0.67.0.

Ten variants including the baseline were screened over 50 cases: empty/singleton,
128/16,384-element signed and unsigned lists, negative/mixed i16 distributions,
raw signed bytes and generic/byte-oriented buffers through 1 MiB. The selected
candidate and baseline also run the complete 298-case standard IC corpus and
one/100-type ordinary-record probes, plus 100-type packed and ordinary-bulk
Wasm probes. All three IC samples
per row agree exactly; input/frame hashes agree across variants and profiles.
Guest-produced frames are checked against native fixtures.

IC operations are warm, with four repetitions through 64 KiB and one above.
Setup/reply transport is excluded; result destruction and loop overhead remain.
The loop control is reported without subtraction. Wire budgets and Candid quotas
have different semantics. Native timing is separate: two runs/profile, eleven
samples with 2 ms target batches, reversing variant order on the second run.
All binaries are built before timing and other compilation/IC runs are stopped.
Allocation/peak-heap metrics are not remeasured for this focused change.

## Ordinary i16 results

These are standard-corpus IC instructions per operation for 16,384 elements.
Frame bytes are identical before/after: 16,398 for small magnitudes and 49,166
for maximum-width values. The Candid frame is 32,780 bytes in either case.

| Profile/path | Before | After | Change | Candid | After/Candid |
| --- | ---: | ---: | ---: | ---: | ---: |
| opt-z small encode | 8,324,732.5 | 2,868,860.5 | -65.54% | 86,354.5 | 33.22x |
| opt-z wide encode | 10,651,260.5 | 4,933,244.5 | -53.68% | 86,267.5 | 57.19x |
| opt-3 small encode | 3,457,874.25 | 1,475,411.25 | -57.33% | 80,484.25 | 18.33x |
| opt-3 wide encode | 4,014,930.25 | 2,638,675.25 | -34.28% | 80,397.25 | 32.82x |
| opt-3 small decode | 2,945,517.5 | 2,306,552.5 | -21.69% | 1,408,358.5 | 1.64x |
| opt-3 wide decode | 5,919,238.5 | 4,182,545.5 | -29.34% | 1,408,452.5 | 2.97x |

The remaining encoding gaps are substantial. This is a byte-preserving improvement,
not bulk parity. Candid writes fixed-width primitive vectors in bulk; ordinary
Postcard lists still visit elements and wide values emit varints. Existing
`Packed<i16>` changes the representation to fixed little-endian bytes and already
beats Candid in the full matrix; producers/consumers must coordinate that field
choice. Small integers can grow when packed.

## Byte-vector result and practical selection

The generic byte-vector runtime path is unchanged. In the standard opt-3 corpus,
1 MiB encoding remains 78,644,279 instructions versus Candid's 2,112,114 (37.23x).
The matched byte-buffer field uses 1,049,704 versus its Candid control's 2,115,234
(0.50x). Both Wire fields have exactly the same 1,048,590-byte frame and SHA-256
`a6c9856864dc18a874d12d43b30d39581ce53ebc35e15efbc94ecb228edb14f6`.

For binary fields, use the existing `Bytes<N>`, `serde_bytes::ByteBuf`, or
`#[serde(with = "serde_bytes")]` on Vec<u8>. This activates Serde's byte-buffer
hook; no new codec is needed. Byte buffers use bulk resource accounting rather
than per-element accounting, so identical bytes do not imply identical budget
acceptance. Retain application length limits and regenerate metadata/clients when
changing a declaration. The existing native-schema test covers exact bytes and
this admission distinction. Ordinary vectors are not silently rewritten.

## Wasm and tradeoffs

Whole-probe total/code-section bytes are reported separately. These are
uncompressed Wasms, not installed application or per-crate savings.

| Probe/profile | Before total/code | After total/code | Delta total/code |
| --- | ---: | ---: | ---: |
| standard opt-z | 402,893 / 362,784 | 403,263 / 363,157 | +370 / +373 |
| standard opt-3 | 611,388 / 556,574 | 612,061 / 557,247 | +673 / +673 |
| 100 ordinary records opt-3 | 424,179 / 389,650 | 424,179 / 389,650 | 0 / 0 |
| 100 packed records opt-3 | 655,798 / 620,658 | 655,798 / 620,658 | 0 / 0 |
| 100 ordinary-bulk records opt-z | 472,331 / 452,796 | 472,627 / 453,095 | +296 / +299 |
| 100 ordinary-bulk records opt-3 | 675,032 / 639,867 | 675,547 / 640,380 | +515 / +513 |

The standard Candid artifact is 933,720 / 869,518 bytes at opt-z and
1,386,033 / 1,306,893 at opt-3. The isolated hotspot probe grows by 153 / 158 bytes
at opt-z and 647 / 647 at opt-3. Small/empty controls and every regression remain
in the evidence; compiler layout can change costs outside the edited method.

## Native timing and regressions

Native medians below are the median of two run medians, in microseconds/op.
They are a separate 50-case cohort; do not compare them to IC instruction counts.
The archive retains all eleven raw samples/run; the CSV retains both run medians.

| Profile/path, 16,384 elements | Before us | After us | Change |
| --- | ---: | ---: | ---: |
| opt-z small i16 encode | 197.956 | 55.140 | -72.15% |
| opt-z wide i16 encode | 206.896 | 96.068 | -53.57% |
| opt-z wide i16 decode | 294.874 | 347.672 | +17.91% |
| opt-3 small i16 encode | 45.330 | 27.864 | -38.53% |
| opt-3 wide i16 encode | 90.061 | 45.997 | -48.93% |
| opt-3 wide i16 decode | 157.807 | 103.349 | -34.51% |

The full standard IC corpus exposes a real opt-z regression outside i16: small
u16 lists at 16,384 elements rise from 2,524,796.5 to 2,983,548.5 instructions
(+18.17%); their decode rises 3.29%. Empty i16 opt-3 decode rises from 420.5
to 431.5 (+2.62%), while singleton wide encode improves 1,044.25 to 961.25
(-7.95%). Generic bytes and byte-buffer IC costs are unchanged. Retain the
i16 savings with this explicitly measured tradeoff, not a claim of universal wins.

Native controls also vary: opt-3 generic 1 MiB byte-vector encode changes from
1,227.463 to 1,327.381 us (+8.14%), while byte-buffer encode changes 17.580 to
17.267 us (-1.78%). The largest relative native encode/decode regression is an
empty u16 encode, 37.948 to 51.183 ns (+34.88%); substantial opt-z wide-i16
decode regresses in both runs (baseline 293.497-296.251 us, candidate
337.657-357.686 us). Do not dismiss these as established noise or extrapolate
this microbenchmark to application latency. The Wasm growth stays below 1 KiB
in the measured probes, with no removed admission/canonical checks.

## Rejected directions

- Zero-initializing the entire output and writing through a cursor reduces the
  opt-3 generic byte-vector encode by 41.67%, but doubles byte-buffer encoding
  cost and makes that path 18x its baseline at opt-z. Reject.
- Mapping i16/i32 through existing unsigned writers improves small i16 values,
  but increases wide-i16 opt-z cost by 4.31%; the direct i16 writer performs better
  across both profiles.
- Direct unsigned-u16 varints cut opt-z wide encoding by 57.25% but increase
  opt-3 cost by 51.30%. Restrict direct writing to signed i16.
- Forced output-method inlining and ten short-copy branches have mixed results
  and do not justify their regressions or additional code. No such runtime paths
  are retained. An eleventh probe forcing only Wasm u16-method inlining produces
  identical standard/hotspot binaries in both profiles, so it is omitted and its
  IC counters are not resampled. Removing budget checks, preflight or exact canonical verification
  remains outside the contract; earlier ablation also did not close these gaps.

## Evidence and reproduction

See [evidence](evidence/hotspot-encoding-0613/) for raw sources, source overlays,
compiler/lock/server identities, all repeated samples, Wasm artifacts and hashes,
matched CSVs and replay helpers. Each source variant is fixed before its build;
local crates are touched when switching source roots to prevent a shared Cargo
cache from reusing another copy's artifacts. Final commented/formatted production
code reproduces the selected hotspot Wasm artifacts byte for byte.

For a new current-source measurement, use an empty output directory:

```sh
CORPUS_FEATURE=hotspots POCKET_IC_BIN=/prepared/pocket-ic \
  bash scripts/ci/evaluate-codecs.sh target/hotspots-new-run
```

This is Wire-side qualification, not an installed consumer claim. Gzip size is
not an optimization criterion. No broad CI or real big-endian qualification is
claimed. Exhaustive native i16 byte/budget checks, runtime/borrowed tests,
Rust/TypeScript conformance, strict Clippy and Rust 1.91 Wasm checks pass.

## Decode follow-up

The [Candid-inspired decode screen](evidence/hotspot-encoding-0613/decode-followup/README.md)
rejects larger canonical-comparison blocks: 1 MiB decode improves 3–6%, but Wasm
grows and small blobs regress. The maintained hotspot corpus now has 82 cases,
including comparison-block boundaries; the original 50-case evidence above is
unchanged. No additional runtime or protocol change is retained.

The [comparison-scan study](evidence/hotspot-encoding-0613/comparison-scan/README.md)
separates bulk scanning from canonical traversal and checks two exact alternatives
on 87 cases, including five new mixed blob/packed/string/optional records. The
scan dominates large-blob validation; both exact alternatives regress IC costs
for tiny Wasm savings. Neither runtime candidate nor diagnostic shortcut ships.
