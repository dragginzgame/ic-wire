# Packed i16 field research (0.6.8)

Historical research snapshot. The selected approach is now promoted to
`PackedI16<N>`; see the [production qualification](packed-i16-production.md).
The implementations and dependency roles below describe the archived experiment.

The bulk approach works in this prototype: a safe little-endian byte view lets
ordinary Serde/Postcard beat Candid on encode and decode across all twenty tested
i16 inputs in both profiles. This is an unpublished benchmark experiment, not a
new production API or a transparent optimization of existing Vec<i16> fields.

## Representation and implementation

The research adapter serializes Vec<i16> storage through Serde's byte interface.
On little-endian targets, bytemuck 1.25.2's
[statically checked slice cast](https://docs.rs/bytemuck/1.25.2/bytemuck/fn.must_cast_slice.html)
provides a safe view without copying or temporary allocation. It remains a direct
dev dependency of the unpublished benchmark package; published crates are unchanged.
The benchmark retains an explicit little-endian conversion fallback for big-endian
targets, which has not been platform-qualified. No local unsafe code is added.

Decode validates byte length and the 16,384-item maximum before allocating,
then returns an owned Vec<i16>. Odd lengths are rejected. Every i16 bit pattern
is valid, so no element-level semantic validation is needed. Native Wire still
performs admission and exact canonical reserialization; these are not bypassed.
The byte representation changes traversal charges compared with an integer list.
Production error/budget semantics and TypeScript metadata remain to be qualified
before exposing any public packed field API. No syntax is proposed as shipped.

The byte-length prefix replaces the element-count prefix and values use fixed
little-endian words instead of zigzag varints. For example, unframed `[32767]`
changes from `01 fe ff 03` to `02 ff 7f`; `[0]` changes from `01 00` to `02 00 00`.
Consumers choosing this representation would need coordinated schema/client
changes. Existing fields, hashes and retry bytes remain unchanged in production;
there is no compatibility reader or second production codec.

## IC instruction results

For 16,384 wide i16 values, with conversion/allocation/destruction included:

| Profile / operation | Ordinary Wire | Packed adapter | Candid | Packed / Candid |
| --- | ---: | ---: | ---: | ---: |
| opt-z encode | 9,274,947.5 | 34,602.5 | 86,093.5 | 0.402x |
| opt-z decode | 9,050,359.5 | 82,471.5 | 1,186,279.5 | 0.070x |
| opt-3 encode | 4,014,993.25 | 33,587.25 | 80,320.25 | 0.418x |
| opt-3 decode | 5,919,337 | 80,218 | 1,408,211 | 0.057x |

The ordinary-Wire column is the control lane in the final research fixture;
separate baseline artifacts and rows also retain the control without packed lanes.
Against the in-fixture control, opt-3 encoding falls 99.16% and decoding 98.64%.
The packed adapter uses fewer instructions than Candid for every measured
encode/decode pair, including empty and one-element inputs. This is a bounded
single-type corpus, not a universal codec performance claim.

Small-input regressions against ordinary Wire remain. Empty-list opt-z decode
is 680.5 → 983.5 instructions (+44.53%, 303 instructions); opt-3 is 431 → 525
(+21.81%, 94 instructions), compared with the separate ordinary baseline.
An empty packed frame is 12 bytes; a one-element frame is 14. For one small i16,
packed opt-3 encode/decode cost 675.25/886 instructions. All controls, distributions
and error-path measurements remain in the raw evidence.

Large wide-value frames shrink from 49,166 to 32,782 bytes, versus Candid's
32,780. Large small-value frames grow from 16,398 to 32,782: nearly twice as large.
This is the central payload-size tradeoff of fixed-width fields.

## Native costs and Wasm footprint

Large wide-value packed-adapter opt-3 native encode medians are 0.530–0.541 µs
across two runs; decode medians are 0.965–0.987 µs. Each operation performs one
allocation: 32,782 bytes for encode and 32,768 for decode, with no reallocations.
Peak live bytes match those sizes. No prepacked source buffer is supplied:
encoding starts with Vec<i16>, and decoding returns Vec<i16>.

| Fixture / profile | Total Wasm | Code-section bytes |
| --- | ---: | ---: |
| Ordinary-only Wire / opt-z | 42,014 | 33,106 |
| Wire with both packed research paths / opt-z | 48,871 | 39,527 |
| Ordinary-only Wire / opt-3 | 45,551 | 36,897 |
| Wire with both packed research paths / opt-3 | 53,807 | 44,778 |
| Candid / opt-z | 215,803 | 190,164 |
| Candid / opt-3 | 284,146 | 259,050 |

The research fixture grows 6,857 total/6,421 code bytes at opt-z and 8,256
total/7,881 code bytes at opt-3. It contains both packed paths, their dispatch,
and the ordinary control. This is not an isolated public-adapter size estimate,
bytemuck symbol attribution or a 100-derived-type scaling qualification.
No compressed Wasm measurement influences selection.

## Experiments and ownership

Three stages are retained, not just the best result:

- Per-element vector growth: conversion-before-codec opt-3 encode costs
  607,403.25 instructions, about 7.56x Candid; an adapter that allocates on both
  sizing/writing passes costs 1,181,530.25. Removing varints alone was insufficient.
- Initialized fixed-width writes and exact-length output extension: conversion
  encode falls to 99,697.25 (1.24x Candid) at opt-3, but remains 985,559.5
  (11.46x) at opt-z. Unpacking becomes a cheap bulk operation.
- Safe byte view: the adapter avoids packing allocation entirely on little-endian
  targets. A separate conversion-before-codec lane still measures an explicit
  owned packing copy and an owned decoded result: opt-3 encode/decode are
  66,799.25/34,002 instructions (0.832x/0.024x Candid). Its encode needs two
  allocations; it does not hide the conversion outside measurement.

The adapter is the promising field-integration direction. The conversion lane
shows the cost when a caller explicitly creates a packed buffer. Both stay
research-only; no published wrapper, metadata adapter or automatic Vec rewrite
has been added. Existing production Serde behavior is untouched.

## Qualification and reproduction

Base revision `a848368` plus the incoming IC Host/Tooling 0.66.0 adoption, preserved
unchanged. Measurements pin IC Platform 0.6.9; a separate working-tree update
to 0.6.10 appeared during the task and was preserved, without changing the
isolated benchmark snapshots. The final cast/control pair shares the same lock and direct benchmark
bytemuck dependency. Earlier loop/bulk stages retain their original manifests.
Rust 1.99.1 / LLVM 23.1.1, Linux x86_64 and wasm32-unknown-unknown; LTO, one codegen
unit, panic abort, stripped symbols; evaluation/benchmark-speed use opt-z/opt-3.
PocketIC 16.1.0 uses three identical samples, four operations through 64 KiB and
one above. No loop overhead is subtracted. All cases here are below 64 KiB.

Inputs have 0/1/128/1,024/16,384 elements and four distributions: small 0–63,
wide i16::MAX, negative -1 through -64, and mixed varint boundaries/extrema.
Native timing uses two runs/profile with eleven calibrated samples, minimum 2 ms
batches and a 1,048,576 repetition cap. Timing is unpinned and includes destruction.
Allocation and peak-heap instrumentation use separate builds; setup is excluded.

[All measurements](evidence/packed-i16-068/measurements.csv),
[artifacts](evidence/packed-i16-068/artifacts.csv),
[identity](evidence/packed-i16-068/identity.json) and the
[raw source/sample archive](evidence/packed-i16-068/raw.json.gz) retain the evidence.
The archive has loop/bulk/cast source variants, initial manifests, build scripts,
all sample arrays and logs. `packed-i16-baseline` selects ordinary controls;
`packed-i16` adds the research lanes, exclusively in the unpublished crate.
Run `CORPUS_FEATURE=packed-i16 POCKET_IC_BIN=PREPARED_SERVER bash
scripts/ci/evaluate-codecs.sh NEW_OUTPUT` from the extracted source. The archived
scripts also reproduce separate baseline builds and the earlier loop/bulk stages;
restore the appropriate source/manifests before each stage. Output directories
must be new. Runtime/limits and deterministic inputs must remain identical.

Exhaustive i16 byte conversion, exact frame examples, size/parity limits,
roundtrip, selected-case identity, malformed/truncated/trailing frames and reuse
controls pass. Strict benchmark lint, Rust 1.91 checks and dependency boundaries
pass. Big-endian execution, public TypeScript metadata, installed consumers and
full CI are not qualified by this research.
