# Public ByteBuf qualification

`ic_wire::ByteBuf` re-exports the existing serde_bytes type. The dependency is
now normal and alloc-only; no serializer/visitor implementation is added.
The consumer fixture compiles and tests with only ic-wire and Serde, with and
without the metadata feature. Wasm graphs include serde_bytes/alloc and exclude
serde_json; Rust 1.91 Wasm checks pass. The public path retains byte metadata,
exact frames and bulk admission, including Rust/TypeScript checks.

Matched 50-case hotspot builds use Rust 1.99.1, the same Cargo.lock, fixtures,
parameters and LTO/one-codegen-unit/abort/stripped profiles. Dependency placement
and features differ as intended. Total/code-section bytes:

| Profile | Before | After | Delta |
| --- | ---: | ---: | ---: |
| opt-z | 76,520 / 66,243 | 76,520 / 66,243 | 0 / 0 |
| opt-3 | 97,507 / 87,860 | 97,507 / 87,860 | 0 / 0 |

Opt-3 binaries are identical. Opt-z hashes differ, so both builds were measured:
all **1,100 paired IC rows** have identical counters, frames and input hashes.
Every row has three equal samples; operations repeat four times through 64 KiB
and once above. Loop overhead remains included. Cases include empty/singleton,
small/large integers, generic bytes and byte-buffer controls through 1 MiB.
The benchmark retains its existing byte-adapter cases; separate type-identity
and consumer tests qualify the public name. No native timing rerun, application
speedup or universal Wasm-size claim follows from this API exposure.

[Identity and hashes](identity.json), [Wasm results](wasm.json),
[all paired counters](instructions.csv) and [raw evidence](raw.tar.gz) retain
sources, source overlay, raw samples, artifacts, compiler/server identity,
parameters, commands and logs. Archive compression is storage only. Reproduce
from the frozen sources, touching local Rust inputs when changing source copies
in a shared target directory; the build helper records the exact Cargo commands.
The instruction driver uses explicit PocketIC 16.1.0 through Testkit 0.67.0.
The production protocol, byte-buffer acceptance and retry bytes are unchanged.
