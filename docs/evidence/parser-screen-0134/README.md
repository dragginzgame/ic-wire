# Parser footprint screen, Core 0.13.4 follow-up

Measured 2026-11-01. Reject all three new candidates. The accepted admission
cleanup from the [previous study](../cost-experiments-0134/README.md) remains
applied; this screen adds no production runtime change. Codec bytes, acceptance,
budgets and retained retry/hash inputs remain unchanged in the working library.

## Total uncompressed Wasm and code-section bytes

One fixed 100-derived-type fixture retains encoding and decoding with features
`wire,derive-scale`. Every library/guest build was observed as rebuilt, including
the baseline. Its modules exactly match the previous study's selected candidate.
Deltas are against that baseline, after the accepted cleanup.

| Candidate | Profile | Total before → after | Total delta | Code before → after | Code delta |
| --- | --- | ---: | ---: | ---: | ---: |
| Direct byte-budget subtraction | opt-z | 310,637 → 310,637 | 0 | 291,902 → 291,902 | 0 |
| Direct byte-budget subtraction | opt-3 | 398,548 → 398,548 | 0 | 364,120 → 364,120 | 0 |
| Shared buffer/string readers | opt-z | 310,637 → 310,758 | +121 (+0.0390%) | 291,902 → 292,021 | +119 (+0.0408%) |
| Shared buffer/string readers | opt-3 | 398,548 → 398,642 | +94 (+0.0236%) | 364,120 → 364,212 | +92 (+0.0253%) |
| Outlined u32/u64 varint tails | opt-z | 310,637 → 310,867 | +230 (+0.0740%) | 291,902 → 292,129 | +227 (+0.0778%) |
| Outlined u32/u64 varint tails | opt-3 | 398,548 → 399,610 | +1,062 (+0.2665%) | 364,120 → 365,178 | +1,058 (+0.2906%) |

Code-section bytes are the section-10 payload of the same final raw module.
All values use uncompressed bytes. Archives only store evidence; compressed
sizes play no part in selection. These complete fixture modules do not establish
application linker savings or consumer code-limit headroom.

The direct byte-budget rewrite is bit-identical at both profiles. Parser helper
and tail extraction grow both metrics. None passes the footprint objective, so
no IC instructions, native timing, allocation or malformed-input qualification
was run for these rejected changes. Compilation alone is not correctness or
performance qualification. No Candid comparison or instruction win is claimed.

## Source and recipe

Baseline Git head: `c2d3dfe5266e696a9a98983bdac53fb38024e522`, plus the accepted
admission cleanup and private harness in [source.tar.gz](source.tar.gz). The
sampler additionally records `native_parser_sha256` for future instruction
studies; this has no guest/runtime instrumentation. Strict sampler Clippy passes.
Three independent patches retain the rejected source:

- [bytes-direct.patch](bytes-direct.patch): compare allowance and subtract
  directly instead of unpacking `checked_sub`.
- [buffer-shared.patch](buffer-shared.patch): share length/range extraction and
  UTF-8 validation in non-generic helpers, retaining owned/borrowed callbacks.
- [varint-tail.patch](varint-tail.patch): keep local one-byte handling and move
  multi-byte u32/u64 parsing into width-specific shared helpers.

Rust **1.99.1**, LLVM **23.1.1**, host `x86_64-unknown-linux-gnu`, target
`wasm32-unknown-unknown`. The `evaluation` recipe is opt-z, fat LTO, one codegen
unit, panic abort and symbol stripping. `benchmark-speed` changes only opt-level
to 3, checking the speed-oriented recipe's footprint tradeoff. There is **no
post-link optimizer or deployment transformation**. These are library recipes;
no consumer deployment/profile setting changes. Manifest/lock/toolchain snapshots
and [identity.json](identity.json) bind inputs. Keep global Cargo config and other
build inputs fixed too; this is not a hermetic-build claim.

## Reproduction

Extract the capsule into a new absolute directory. Prepare the recorded compiler,
target and locked dependency caches. Use one fixed source path and an exclusive
`CARGO_TARGET_DIR`, with `CARGO_NET_OFFLINE=true` and `RUSTUP_AUTO_INSTALL=0`.
No server or publication is needed.

For each candidate restore the four baseline files under
`crates/ic-wire/src/native/` (`mod.rs`, `encode/mod.rs`, `decode/mod.rs`,
`parser/mod.rs`), apply its independent patch with `patch -p1`, and refresh their
mtimes. Run both profiles and retain build JSON and each artifact before the
next build:

```sh
cargo build --locked --offline --profile evaluation -p ic-wire-benchmarks \
  --example codec-wasm --target wasm32-unknown-unknown \
  --features wire,derive-scale --message-format=json > evaluation-build.jsonl
cargo run --quiet --locked --offline -p ic-wire-benchmarks \
  --example codec-report -- --artifact evaluation-build.jsonl
# Copy the resolved module and record its physical size and SHA-256.
# Measure code with codec-report --code-size /absolute/copied/module.wasm.
# Repeat using benchmark-speed. Do not stack candidate patches.
```

Require `fresh=false` for both `ic_wire` and `codec-wasm` in every build record.
The exact local runner [screen-size.sh](screen-size.sh) expects runtime copies
under `variants/<name>/` and an adjacent `source/`; adapt its exclusive cache
path before using it elsewhere. The capsule is restored to the baseline.

[wasm.csv](wasm.csv), [artifacts.tar.gz](artifacts.tar.gz),
[build-records.tar.gz](build-records.tar.gz) and [fresh-builds.json](fresh-builds.json)
retain all eight raw modules, hashes, effective settings and rebuild observations.
[SHA256SUMS](SHA256SUMS) covers the evidence. Rejected patches remain research.
