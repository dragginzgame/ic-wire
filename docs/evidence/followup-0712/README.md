# 0.7.12 follow-up: consumer adoption, rejection and Wasm sharing

Baseline: released Wire 0.7.11, commit
`a99a99cc3fadbde004e69d9ee4888c99711b3bee`. The working tree
was clean at the start. Production codec source remains unchanged in this batch.
The candidate below is research only, not a new codec mode or retained runtime change.

## Consumer verification

Read-only inspection of Canic's working tree on 2026-10-25, based on
`cf2d3d1fba84a00a76cdbc541ae7e68fe36e9bb5`, found incomplete adoption.
Its manifest requirement is `0.7.10`, but its lockfile already resolves `0.7.11`.
The following callers still use the removed three-argument generator API:

- `crates/canic-host/src/artifact_io/wire/mod.rs` and its tests.
- `crates/canic/src/endpoint/schema/mod.rs` and `crates/canic/tests/wire_schema/mod.rs`.
- `crates/canic-contracts/examples/wire_conformance/main.rs`.
- `crates/canic-testing-internal/examples/wire_composition/main.rs`.

This source cannot compile those calls against 0.7.11. No Canic suite was run,
no sibling was edited, and its substantial uncommitted work was left intact.
Canic must pass an acknowledgement map and diagnostic handler at each caller,
route reports to normal CLI/build output, and test both default visibility and
intentional suppression. Its artifact generator should not silently discard the
report. See the [migration examples](../../packed-fields.md#host-side-bulk-advice-and-ci).
Installed warning visibility therefore remains **unverified**, not complete.

## Rejection audit

The existing adversarial tests cover pre-handler framing refusal, independent
calls, zero-byte elements and huge declared lengths. Recursive values already
have depth/node refusal tests. The added test checks shared budgets across nested
lists and counts visits for truncated ordinary byte lists with 0/1/16/127/256
available elements and declared counts 1,024, 1,048,576 and usize::MAX.
Work stops after the available prefix plus the failing element; nested collections
cannot reset their budget. These are structural checks, not instruction estimates.
Runtime errors are fixed enum variants; they do not accumulate payload-sized
error strings. No admission bypass or unbounded diagnostic construction was found.

A universal `declared length > remaining bytes` rejection is invalid for generic
Serde collections: unit values can encode in zero bytes. It would reject valid
payloads. Byte-oriented fields can check their contiguous extent immediately;
ordinary vectors must visit their prefix. Byte buffers and Packed remain the
simple solution for bulk fields. This audit does not remove the ordinary-list gap.

Fresh standard-corpus counters compare Wire 0.7.11 with Candid 0.10.39 through
ic-platform 0.6.16, Postcard 1.1.3 and Testkit 0.68.4. These opt-3 truncated-input
results include small controls and result destruction:

| Case | Wire owned instructions | Candid instructions |
|---|---:|---:|
| Ordinary byte vector, 1 element | 322.75 | 23,580.75 |
| Ordinary byte vector, 1 MiB | 42,472,926 | 24,367 |
| Byte buffer, 0 bytes | 358.75 | 21,838.75 |
| Byte buffer, 1 byte | 380.75 | 23,108.75 |
| Byte buffer, 1 MiB | 636 | 24,861 |
| Nested blobs, 128 entries | 180,151 | 345,345 |
| Wide u64 list, 16,384 entries | 6,656,275 | 24,567 |

Inflated lengths and budget refusal are covered structurally by tests, not by
new Candid instruction cases. Do not generalize truncation timings to every
malformed input or to installed endpoints.

## Wasm experiment

The candidate changes only four private varint readers to `inline(never)` on
Wasm, retaining native compiler hints. This shares their full bodies across
callers. Same source directory, compiler, lock, features and profile are used for
both variants; fixtures include small scalar controls and large lists.
[Raw sizes and hashes](wasm-sizes.json) distinguish code sections and total bytes.

| Fixture / profile | Total before → candidate | Code before → candidate |
|---|---:|---:|
| Standard / opt-z | 377,811 → 377,817 | 337,782 → 337,788 |
| Standard / opt-3 | 573,482 → 562,566 | 518,734 → 507,821 |
| 100 types / opt-z | 310,738 → 310,918 | 291,971 → 292,149 |
| 100 types / opt-3 | 398,739 → 398,871 | 364,245 → 364,376 |

Only standard opt-3 improves: total −1.90%, code −2.10%. The other total sizes
regress by 6, 180 and 132 bytes. No gzip comparison or symbol-attribution savings
claim is used. Native timing is not measured: the candidate is Wasm-only.

**Rejected:** instruction costs outweigh the isolated size saving. At opt-3,
owned decode for a 16,384-element small-u16 list rises from 815,349.5 to
1,503,529.5 instructions (+84.40%). Empty/single-element controls rise from
298.5 to 340.5 (+14.07%) and 693.5 to 775.5 (+11.82%). Scalar u16 zero rises
from 256.5 to 302.5 (+17.93%). No runtime candidate was copied into the repository.

Both standard runs cover 298 cases and 7,006 operation rows apiece, each with
three identical repeated counters. All paired frames and input hashes match;
Candid control counters are unchanged. The 100-type fixture is size-only: the
standard regressions and its size increase were sufficient to reject the candidate.
The [raw instruction archive](instructions.json.gz) includes both identities and
all samples; [owned changes](instruction-changes.csv) retain improvements and
regressions. Archive compression is storage only. The [analyzer](analyze.mjs)
checks pairing, hashes and repeats. Focused native/runtime tests, strict Clippy,
formatting and Rust 1.91 tests pass; full CI and consumer builds were not run.

## Reproduce

Use the pinned Rust 1.99.1 toolchain with the Wasm target and cached dependencies.
[Compiler](compiler.txt) and [input hashes](input-hashes.txt) identify the run.
The [measurement script](measure.mjs) applies the candidate in an isolated tree,
builds both profiles for both cohorts, records artifact hashes/sizes and restores
the parser. Production source is never changed.

```sh
run=$(mktemp -d)
mkdir "$run/source"
git archive a99a99c | tar -x -C "$run/source"
cp "$run/source/crates/ic-wire/src/native/parser/mod.rs" "$run/parser-before.rs"
node docs/evidence/followup-0712/measure.mjs "$run"
```

For instruction counters, select the prepared PocketIC 16.0.0 server whose hash
is retained in the raw archive's identities. Then run:

```sh
export POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic
for profile in evaluation benchmark-speed; do
  CARGO_TARGET_DIR="$run/target" cargo +1.99.1 build --offline --locked \
    --manifest-path "$run/source/Cargo.toml" --profile "$profile" \
    -p ic-wire-benchmarks --example codec-wasm --target wasm32-unknown-unknown --features candid
  for variant in baseline candidate; do
    cp "$run/target/wasm32-unknown-unknown/$profile/examples/codec_wasm.wasm" \
      "$run/standard-$variant/$profile-candid.wasm"
  done
done
CARGO_TARGET_DIR="$run/host-target" cargo +1.99.1 build --offline --locked \
  --manifest-path "$run/source/Cargo.toml" -p ic-wire-benchmarks \
  --example codec-instructions --features wire,candid,ic
for variant in baseline candidate; do
  "$run/host-target/debug/examples/codec-instructions" "$run/standard-$variant"
done
```

The maintained driver checks exact native/Wasm frames, runs three repeated samples,
uses four repetitions for small inputs and one above 65,536 bytes, and keeps
loop overhead separate. Transport and setup are excluded from the counters.

After both instruction runs, archive and compare the samples with:

```sh
node docs/evidence/followup-0712/analyze.mjs "$run" /absolute/output-directory
```
