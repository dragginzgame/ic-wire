# Standalone payload schema validation (0.7.21 draft)

`typescript::validate_payload_schema(types, schema)` checks the complete named
registry and a standalone root through the existing generator's identifier,
shape/reference and alias-cycle checks. Registry and root share the same 65,536
syntactic-node allowance and inline depth limit of 64. Recursive records and
collections remain valid; alias-only cycles and malformed unused definitions
fail. No client module is emitted. The shared type renderer still makes temporary
strings; this is not an allocation-free validator.

The helper validates trusted metadata, not actual values, resource policies or
consumer scalar callbacks. Bulk advice is separately available through
`lint_bulk_roots`. Generator interfaces, emitted TypeScript, runtime acceptance,
Postcard bytes and identities are unchanged. Existing consumers need no migration.
This is a host-only API; no codec/IC instruction or Wasm-size change is claimed.

## Host measurement

[Metrics and source identities](metrics.json) identify Rust 1.99.1 / LLVM 23.1.1,
x86_64-unknown-linux-gnu, `evaluation` (opt-z, fat LTO, one codegen unit, panic
abort, stripped symbols) and the unchanged Cargo.lock. The baseline is the local
working tree at release `1608df0661137003c7126040e8656598b98d5bd8` plus the pending
scoped-output/warning fixes, before adding this validator. Local before-source
copies and full generated output/logs remain in `target/schema-validator-0721/`.
The measured candidate implementation is unchanged after a documentation wording
edit. No dependency or compiler settings differ between runs.

The [maintained example](../../../crates/ic-wire/examples/typescript_conformance/native_schema/mod.rs)
prepares one/100 `Page<u64>` record schemas, a `ref Page0` root and payload policy
before timing. The previous API alternative generates a disposable payload module,
including policy checks, bulk advice and output destruction. The standalone path
performs schema validation only. Both check the entire registry; only the module
alternative emits clients. Diagnostic computation is included, output excluded.
This compares two ways to qualify metadata; it is not a speedup to generation.

Seven samples of 128 iterations alternate the two paths, after warmup. Median
amortized host nanoseconds, with all [raw samples](validation-samples.json):

| Types | Generate/discard module | Validate schema | Change | Generation sample range | Validation sample range |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 5,178 | 1,414 | -72.69% | 5,122–5,252 | 1,293–1,491 |
| 100 | 158,205 | 83,863 | -46.99% | 151,315–224,897 | 81,938–126,145 |

The new API has no released predecessor; full module generation is the explicit
old alternative. Shared-host wall time is not IC instructions or endpoint latency.
Two additional candidate runs retain their standalone samples:
[repeat 2](repeat-2-schema-validator-host-benchmark.json),
[repeat 3](repeat-3-schema-validator-host-benchmark.json).

## Existing generation controls

All 229 generated/vector/runtime/mutation files match before/after byte-for-byte.
Only the two existing host timing JSON files change, plus the new standalone
timing file. All 36 ordinary warning messages match exactly; no synthetic Page
warnings leak. TypeScript/runtime conformance evidence from the preceding warning
fix remains applicable to these identical files; it was not rerun here.

The existing host generation control includes metadata construction, rendering
and bulk advice; it differs from the new prebuilt-metadata comparison above.
These before/after measurements retain fluctuations and regressions:

| Types / derived | Before ns | First after ns (change) | Repeat 2 ns | Repeat 3 ns |
| --- | ---: | ---: | ---: | ---: |
| 1 / false | 3,185 | 3,086 (-3.11%) | 3,081 | 3,104 |
| 1 / true | 3,692 | 3,710 (+0.49%) | 3,511 | 3,634 |
| 100 / false | 300,720 | 457,979 (+52.29%) | 308,369 | 299,285 |
| 100 / true | 345,933 | 493,588 (+42.68%) | 356,745 | 343,767 |

The first large-control increases did not persist in repeated executions of the
same candidate binary. These non-interleaved shared-host runs do not isolate a
causal library regression or establish exact performance equivalence. No existing
generator speedup is claimed. Raw [before](before-native-schema-host-benchmark.json),
[after](after-native-schema-host-benchmark.json),
[repeat 2](repeat-2-native-schema-host-benchmark.json),
[repeat 3](repeat-3-native-schema-host-benchmark.json) samples and independent
[before](before-method-limits-host-benchmark.json) /
[after](after-method-limits-host-benchmark.json) method-limit timings are retained.

## Reproduction and qualification

From the matching source/locked graph, with no active build sharing target/:

```sh
cargo run --locked --offline --profile evaluation -p ic-wire \
  --features typescript --example typescript-conformance -- NEW_OUTPUT \
  > generation.stdout.log 2> generation.stderr.log
```

Read `schema-validator-host-benchmark.json`; take the median of seven
`ns_per_iteration` samples for each `types` / `validate_only` pair. Percentage is
`100 * (validation / generation - 1)`. Run the baseline without the validator for
existing generation controls. Compare non-timing generated files exactly.

Focused checks pass: three standalone tests plus the existing payload generator
policy/diagnostic test; strict Clippy for the library, native tests and conformance
example; one feature-enabled validator doctest; Rust 1.91 library compilation
with TypeScript enabled. Tests cover recursive constructors, malformed root/unused
schemas, missing references, invalid identifiers, alias cycles, inline depth and
the exact aggregate work boundary. No full CI, full Candid matrix, native macOS
or installed SDK qualification was run for this addition.
