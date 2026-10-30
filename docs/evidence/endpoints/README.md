# Endpoint extraction qualification

The standalone fixture qualifies Wire-owned typed handlers through published
Platform 0.8.2. It does not qualify Canic adoption. Build/Testkit 0.70.2 and the
other incoming lock selections were preserved. The comparison changes only
inline versus extracted Wire hook ownership under the same Platform generator;
it never copies Canic's manual export machinery or installs another executor.

## Reproduce

```sh
make check-endpoint
POCKET_IC_BIN=/prepared/pocket-ic make check-endpoint-ic
POCKET_IC_BIN=/prepared/pocket-ic \
  bash scripts/ci/evaluate-endpoint.sh target/endpoint-cost-new
```

The measurement runner requires a new output directory and a prepared server;
Testkit owns server admission/lifecycle. It records compiler, host, exact lock,
source/server hashes, Wasm hashes, repeated raw samples and separate uncompressed
Wasm/code-section sizes. No gzip measurement participates in selection.
The maintained comparison source is
[`endpoint-cost`](../../../examples/endpoint/examples/cost/mod.rs).

The baseline is that fixture's `inline-baseline` feature: equivalent handwritten
Wire guard/decoder/encoder hooks at Platform's generator, using Canic's audited
identity, budget and trap behavior. The candidate uses the new Wire attribute
and helpers. Both artifacts have the same unit and ByteBuf echo handlers and
identical ID/limit constants. This measures extraction overhead relative to inline
Platform hooks; it is **not** a before/after measurement of a Canic application,
its former manual exports, Fleet policy or dispatch instrumentation.

## Measurement boundaries

Compiler: Rust 1.99.1; target: `wasm32-unknown-unknown`; native host:
`x86_64-unknown-linux-gnu`. Both `evaluation` (`opt-level=z`) and
`benchmark-speed` (`opt-level=3`) inherit LTO, one codegen unit, panic abort and
symbol stripping. The server is the prepared PocketIC 16.0.0 binary, admitted by
Testkit's supported stable 16.x policy. The lock selects PocketIC client 16.1.0.
Each profile/lane/input has eleven samples. Requests use unit or a ByteBuf filled
with `a5` at lengths 0, 32 and 65,536. IDs are 22/unit and 21/blob. Full-frame
allowances are bytes 131,072, allocation 8,388,608, nodes 32 and depth 8.

IC samples use `performance_counter(0)` while serializing the final response
counter field on the encoder's second pass. They include entry/copy/decode,
handler execution, the first encode pass and most of the second pass. They
exclude final counter-field serialization, reply syscall and cleanup. The counter
uses fixed-width bytes so instrumentation cannot change output length between
passes. This is not total ingress cost or a native timing.

Native samples time decode plus both response encoding passes, allocation and
destruction, with `black_box` inputs/results. Counter values are zero on native.
Small inputs use 10,000 repetitions per sample; the 65,536-byte input uses 100.
Native timing excludes Platform ingress/executor work. Other host builds were
active during this run, so timing deltas are observations, not established
speedups. Host macro-generation time was not isolated or measured; build logs
are not presented as generator benchmarks.

## Focused correctness

Native tests compare contract types/constants with actual handlers and existing
schema projection, including a bounded record, unit and tuple responses. Eight
compile-fail cases plus a valid renamed-import/raw-identifier/qualified-type/cfg
control passed. Strict core/derive/fixture Clippy, Wasm build, Rust 1.91 Wasm
checking and core/endpoint dependency isolation passed.

Installed tests passed successful record/unit calls, exact native response
bytes, malformed/truncated/trailing/wrong-ID/wrong-direction frames, oversized
requests, node admission, below/at/above reply byte limits, synchronous mutation
rollback, ready-future rollback, async query and actual composite self-call.
After a real async update await, the pre-await increment committed and the
post-await increment rolled back on reply failure. Explicit framework composition
used one guarded dispatch, preserved its rejection and produced the unwrapped
Wire response. No Candid or Canic runtime dependency is present.

Raw local evidence is under `target/wire-endpoint/`, with compile diagnostics
under `target/endpoint-check.CH8ejI/`. The initial sandbox port refusal and corrected
fixture-boundary failure are retained, separate from passing final logs.
Portable samples and measurement results are retained beside this report.

## Results (inline → extracted)

All values below are sample medians. Every IC sample for a given row was equal.

| Profile | Input | IC instructions to counter | Change | Native ns/op | Change |
| --- | --- | ---: | ---: | ---: | ---: |
| opt-z | unit | 244,772 → 244,933 | +0.066% | 57.11 → 58.53 | +2.49% |
| opt-z | empty blob | 245,285 → 245,481 | +0.080% | 76.18 → 75.82 | −0.48% |
| opt-z | 32-byte blob | 245,719 → 245,922 | +0.083% | 83.18 → 85.57 | +2.87% |
| opt-z | 65,536-byte blob | 763,548 → 763,751 | +0.027% | 2,558.75 → 2,288.94 | −10.55% |
| opt-3 | unit | 243,616 → 243,616 | 0% | 8.35 → 7.52 | −9.96% |
| opt-3 | empty blob | 243,862 → 243,862 | 0% | 29.94 → 29.21 | −2.44% |
| opt-3 | 32-byte blob | 244,296 → 244,296 | 0% | 35.38 → 34.43 | −2.69% |
| opt-3 | 65,536-byte blob | 762,001 → 762,001 | 0% | 2,485.43 → 2,250.28 | −9.46% |

| Profile | Total uncompressed Wasm | Change | Code section | Change |
| --- | ---: | ---: | ---: | ---: |
| opt-z | 31,806 → 32,414 bytes | +608 / +1.91% | 28,163 → 28,655 bytes | +492 / +1.75% |
| opt-3 | 34,622 → 34,622 bytes | 0 / 0% | 30,872 → 30,872 bytes | 0 / 0% |

The size-optimized candidate retains small instruction and artifact regressions.
The speed-optimized sizes and sampled IC cost are equal; equal sizes do not mean
byte-identical artifacts. This extraction is an ownership/API improvement, not
an established performance win. Native timing fluctuations do not justify a
speedup claim or erase the size-optimized regressions.

Raw samples: [opt-z IC](evaluation-ic.json), [opt-z native](evaluation-native.json),
[opt-3 IC](speed-ic.json), [opt-3 native](speed-native.json), [Wasm sizes](wasm.csv).
The [frozen lock](Cargo.lock), [input hashes](inputs.sha256),
[Canic audit hashes](canic-sources.sha256) and artifact hashes
([opt-z](evaluation-artifacts.sha256), [opt-3](speed-artifacts.sha256)) bind the run.
The [fixture manifest](fixture-Cargo.toml) retains its exact measured bytes before
the final formatter-only section ordering change.
Repository baseline was `e90aed4f7dc0cd7ee31a9bcfb211044220ab0cec` plus the
uncommitted endpoint implementation identified by those hashes. No source,
version, publication or release effects occurred in Canic or Platform.
