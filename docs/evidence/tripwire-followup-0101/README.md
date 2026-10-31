# Tripwire follow-up on Wire 0.10

Research only. **None of the three prototypes is promoted to production.**
Wire keeps one Serde/Postcard codec. Its bytes, budgets, malformed-input rules,
identity and retained retry/hash inputs are unchanged. The maintained change is
adopting IC Build/Testkit/managed release editor **0.72.1**, retaining Platform
**0.11.1** and Shared Tooling **0.27.11**.

## Inputs and upstream review

Baseline is released Wire `c9d4e7b` plus [supplier inputs](inputs.patch).
The [source comparison](tooling-source-review.json) finds identical Rust `src/`
content in Build, Testkit and the selected Host packages between 0.72.0/0.72.1.
Tooling release commit `85f759d7` changes publication preparation, not these Rust
implementations. Endpoint native/schema/diagnostic/lint/Wasm/MSRV checks, pin and
boundary checks, managed editor admission and the disposable release-adapter
fixture pass; [logs](checks/) are retained. Earlier Platform/schema qualification
remains in the [0.72.0 supplier study](../platform0111-tooling072/README.md).
No release, publication or sibling checkout edit was performed.

Tripwire references are pinned to harness
[`b5a40b12`](https://github.com/Neutrinomic/cdk-rsx-harness/tree/b5a40b12f69e2acec90edf7c94fa9d1929716794),
with SDK `d0e53afc7b7fdc4b7cf4e2ccbe1e55a386133c4b` unchanged. Its
[stack SDK](https://github.com/Neutrinomic/cdk-rsx-harness/blob/b5a40b12f69e2acec90edf7c94fa9d1929716794/research/experiments/2026-10-30-tripwire-optimization/stack-sdk/README.md)
suggests caller-owned output, while its
[LZ4/gzip study](https://github.com/Neutrinomic/cdk-rsx-harness/blob/b5a40b12f69e2acec90edf7c94fa9d1929716794/research/benchmarks/bench-2026-10-30-lz4-gzip.md)
adds compression measurements. Those are hypotheses for our workload, not Wire
savings. Gzip artifact size is not measured or used for selection here.

## 1. Shared inline reply carrier: reject this implementation

One non-generic owner holds up to 128 frame bytes inline, falling back to Vec.
Both paths retain the existing sizing pass and serialization budget. The
[Wire patch](inline-reply.patch) and isolated [Platform hook patch](platform-hook.patch)
remove only the hook's explicit Vec result binding; Platform's reply helper
already accepts `AsRef<[u8]>`. The published supplier is unchanged.

Tests compare exact frame bytes for blob lengths 0–300 and error parity for
frame, allocation, node and depth limits. Native endpoint tests, strict library
Clippy and installed limit/rollback checks pass. The matched endpoint probe
uses 12 cases across seven exports, eight measured samples after warmup,
384 logical calls and 608 physical executions. Frames, outcomes and transaction
oracles match. [Full results](comparison.json) retain every instruction/cycle
sample; [raw traces](samples.json.gz) bind module and server hashes.

| Fixture/profile | Total uncompressed Wasm | Code section |
| --- | ---: | ---: |
| Endpoint matrix, opt-z | 99,951 → 104,574 B (+4.63%) | 86,068 → 90,601 B (+5.27%) |
| Endpoint matrix, opt-3 | 93,518 → 95,419 B (+2.03%) | 80,630 → 82,489 B (+2.31%) |
| 100 endpoint types, opt-z | 232,906 → 271,993 B (+16.78%) | 218,985 → 257,900 B |
| 100 endpoint types, opt-3 | 331,797 → 410,704 B (+23.78%) | 325,229 → 404,024 B |

Opt-z empty echo costs 253,815 → 253,573 instructions (−0.095%); 32-byte
borrowed echo 255,381 → 255,050 (−0.130%). The large-blob fallback regresses
3,884,431 → 3,884,560 (+0.003%). At opt-3, 32-byte default echo regresses
247,824 → 248,079 (+0.103%). This does not justify the extra Wasm.
[Scaling evidence](scale.json) also includes 1/10 types, actual export counts
and hashes. Those are footprint controls, not runtime tests for 100 endpoints.
A follow-up [single shared writer](shared-writer.patch) removes the two serializer
instantiations per type and passes the same byte/budget parity test. It narrows
but does not remove the footprint regression: at 100 endpoint types total Wasm
is 232,906 → 244,107 B (+11,201 B, +4.81%) at opt-z and 331,797 → 359,358 B
(+27,561 B, +8.31%) at opt-3. This candidate fails the footprint screen, so it
was not advanced to an IC timing run. Neither result is a symbol-attribution
claim. No supplier request is needed for these rejected implementations.

## 2. Selective ingress: keep as research

Reuse the retained [known-shape fixture](../tripwire-adoption-0725/selective/fixture/mod.rs)
with current suppliers: mode, nonce and a trailing bounded blob. These are raw
IC export controls, not qualification of Platform-generated executor integration.
Native byte and
malformed-input parity tests pass, including more than 10,000 mutations. This
can skip copying an unused body or copy only a selected prefix, but it is **not**
a generic Serde reader and its fixed limits do not prove general budget parity.

Generated Platform `decode_with` hooks currently receive an already-copied Vec.
Borrowing eliminates a second owned decode copy, not the initial IC ingress copy.
A BTreeMap or arbitrary custom Serde visitor cannot generally be replaced by this
fixed byte-range locator without validating its complete representation and
preserving admission semantics.

The strict parser currently fixes its input to Postcard `Slice<'de>` and supplies
borrowed slices to Serde. Postcard's reader flavors alone do not solve this:
borrowed visitors require stable backing storage, and an allocating reader would
need an admission audit because `CheckedVisitor` charges buffer bytes after the
parser supplies them. There is no demonstrated generic drop-in saving here.

The next useful supplier contract is a scoped **pre-copy input hook**: admit
message length first, expose checked range reads, retain primitive validation,
and provide safe owned range copying without zero-fill duplication. Wire must
then qualify exact errors/budgets and input lifetime, including async ownership.
First require a consumer with substantial partial-blob use; the ingress-cycle
saving below is small despite the instruction win. Keep application-specific
partial consumption explicit. Do not add a second
production decoder or a speculative public range-reader API to Wire.

The refreshed physical probe passes all **1,536 updates**: 24 workloads × four
lanes × two profiles × eight matched nonce variants, after an untraced warmup.
These variants are paired inputs across lanes, not repeated identical timing
inputs. [opt-z](input-evaluation.json), [opt-3](input-benchmark-speed.json) and
[bound traces](input-samples.json.gz) retain exact samples, bytes and outcomes.

| Borrowed → selective, opt-z | IC instructions | Charged cycles |
| --- | ---: | ---: |
| Empty body, consume all | 484,480 → 169,459 (−65.02%) | 6,974,980 → 6,659,959 (−4.52%) |
| 32 B, consume all | 484,999 → 170,601 (−64.82%) | 7,041,499 → 6,727,101 (−4.46%) |
| 1 MiB, consume all | 19,841,846 → 17,106,093 (−13.79%) | 2,123,496,346 → 2,120,760,593 (−0.129%) |
| 1 MiB, consume 256 B | 4,096,993 → 174,876 (−95.73%) | 2,107,751,493 → 2,103,829,376 (−0.186%) |
| Truncated large input | 4,090,737 → 322,023 (−92.13%) | 2,107,736,737 → 2,103,968,023 (−0.179%) |

The 1 MiB prefix case at opt-3 is 4,094,881 → 171,918 instructions (−95.80%)
and 2,107,749,381 → 2,103,826,418 cycles (−0.186%). Against borrowed input,
selective total/code Wasm changes 20,059/17,717 → 20,893/18,291 B at opt-z
(+834/+574 B), and 21,187/19,071 → 21,162/19,042 B at opt-3 (−25/−29 B).
Direct owned range copying also remains in the reports; its zero-initialized
buffer makes the 1 MiB prefix path worse than borrowed decoding. Do not present
selective ingress as a universal optimization for arbitrary schemas or budgets.

A narrower [small-input buffer screen](small-input.patch) keeps the complete
contiguous input and ordinary borrowed Serde decoding, using 128 stack bytes
before falling back to Vec. It adds [769 B at opt-z / 474 B at opt-3](small-input-size.json)
in this fixture. It was not advanced to IC timing or multi-endpoint scaling;
no performance win or general endpoint qualification is claimed. A Platform
pre-copy hook would still be needed to use it in generated endpoints.

## 3. Compression: consumer transport only

The isolated probe uses safe `lz4_flex 0.14.0` block encoding with a four-byte
uncompressed length. Decode checks the length cap before allocation and verifies
actual output length; small prefix, oversized length, truncated block and wrong
length controls reject. It treats complete Wire frames as opaque bytes, retaining
original bytes after decompression. This envelope is research, not a Wire frame
or proposed acceptance-rule change.

The corpus includes every Wire lane in the standard fixture plus deterministic
random/zero controls at 0, 32, 2 KiB, 64 KiB and 1 MiB. IC samples select nine
spaced corpus entries plus controls, with unique callers, a warmup and eight
measured repeats. The counter covers output allocation and codec work **only**;
it excludes input copy, export setup, reply calls, transfer and application work.
The copy control is a Vec clone, not a Candid or full endpoint comparison. Wasm
figures measure adding compression/decompression to this minimal transport
fixture, not a forecast for a full consumer application.

LZ4 roundtrips pass for **353 inputs** (343 actual Wire-lane frames and ten
synthetic controls). Only 138 become smaller. These are fixture entries, not
353 independent application schemas. Small-message and incompressible controls
matter: unit grows 11 → 16 B, random 64 KiB 65,536 → 65,798 B, and random 1 MiB
1,048,576 → 1,052,694 B. Repetitive ASCII 64 KiB frames shrink 65,550 → 287 B;
that fixture is deliberately repetitive, not a claim about arbitrary text.

| Minimal transport fixture | Copy control total / code | LZ4 total / code | Added total / code |
| --- | ---: | ---: | ---: |
| opt-z | 14,403 / 12,828 B | 34,387 / 27,072 B | +19,984 / +14,244 B |
| opt-3 | 15,330 / 13,756 B | 34,455 / 27,829 B | +19,125 / +14,073 B |

The matched opt-z **compression-body** controls show the additional work on
identical input bytes (output sizes differ deliberately):

| Input | Copy body instructions | LZ4 encode body instructions | LZ4 decode body instructions |
| --- | ---: | ---: | ---: |
| Unit Wire frame, 11 B | 469 | 10,345 | 1,294 |
| Random 32 B | 490 | 13,174 | 1,410 |
| Random 64 KiB | 226,461 | 598,759 | 299,319 |
| Zero 64 KiB | 226,461 | 704,595 | 299,943 |
| Random 1 MiB | 2,409,501 | 4,611,987 | 3,565,655 |
| Zero 1 MiB | 2,409,501 | 9,851,804 | 3,566,253 |

Decode is shown as an absolute cost; cloning a compressed buffer is not a
semantic decompression baseline. These are added transport operations, not
regressions in Wire's serialization or comparisons with Candid. Each listed
value has eight post-warmup samples. No native-time claim is made.

[All size rows](compression-ratios.jsonl), [both-profile timing results](compression-summary.json)
and [raw query samples](compression-samples.json.gz) retain 353 inputs and
1,216 measured queries after 152 excluded warmups. Opt-3 is not uniformly faster:
the unit-frame LZ4 encode costs 89,402 body instructions versus 10,345 at opt-z;
random 1 MiB encode improves from 4,611,987 to 4,334,401. Caller identity is unique
for every query; module hashes match the four measured artifacts.

Given the Wasm priority, leave compression out of Wire. A consumer with measured
repetitive bulk traffic could evaluate a bounded, explicitly identified transport
envelope, including decompression admission and peer negotiation. Thresholds and
raw fallback need that consumer's traffic/cycle measurements. Do not transparently
compress arbitrary Wire frames, replace retained retry bytes, or infer end-to-end
cycle savings from this codec-body counter.

## Reproduction

Use the [compiler](compiler.txt), [build environment](environment.json),
[explicit instrumented server](server.sha256),
locked registry packages and `wasm32-unknown-unknown`. Packages must be fetched
before offline builds; LZ4 is added only inside the research copy.

```sh
out=/tmp/wire-tripwire-reproduction
study=docs/evidence/tripwire-followup-0101
bash "$study/prepare.sh" "$out"
export POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic-server
bash "$study/run-reply.sh" "$out"
bash "$study/scale.sh" "$out"
cp -R "$out/candidate" "$out/shared"
patch -d "$out/shared" -p1 < "$study/shared-writer.patch"
bash "$study/scale.sh" "$out" shared
bash "$study/run-input.sh" "$out"
bash "$study/run-small-input.sh" "$out"
bash "$study/run-compression.sh" "$out"
node "$study/analyze.mjs" "$out"
```

The server needs local loopback permission. Research sources and isolated Cargo
patches remain under the output directory; consumers require no Cargo patch.
Source directory paths can affect non-code Wasm metadata, so compare both total
uncompressed bytes and code sections and retain fresh artifact hashes. Raw
sample archives are compressed only for repository storage, not size claims.
No full CI, full Candid comparison or macOS qualification was run for this batch.
