# Generator warning flood (0.7.21 draft)

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

The TypeScript conformance example runs a host-only generation benchmark at one
and 100 synthetic Page types, seven samples of 100 iterations per handwritten /
derived lane. Its callback printed every bulk-advice report, producing 141,804
synthetic warnings per run. This was qualification output, not consumer warnings.

The benchmark now black-boxes the computed report. Normal generation callbacks
still print their warnings. The CI wrapper retains generation stderr, rejects
synthetic Page warnings before replay, and otherwise replays diagnostics and
preserves the generator's failure status. No global suppression, new feature,
codec, protocol or generated-client change is introduced.

## Matched evidence

Baseline: `1608df0661137003c7126040e8656598b98d5bd8` (released 0.7.20).
The only changed benchmark behavior is diagnostic output; lint computation stays
inside timing. Same physical source/target directory, compiler, lockfile, inputs
and profile before/after; Cargo recompiles the changed example normally.
See Retired raw output (see retention notice) and raw Retired raw output (see retention notice) /
Retired raw output (see retention notice) samples. New rows label diagnostic output as excluded.

| Observation | Before | After |
| --- | ---: | ---: |
| Synthetic warnings | 141,804 | 0 |
| Normal contract warnings | 36 | 36, identical messages |
| Identical generated/vector files | — | 229 |

Only the two host timing JSON files differ. All TypeScript source, Rust vectors,
mutation inputs and runtime output files are identical. Full local source,
compiler identity and stderr logs remain under `target/warning-output-0721/`;
the large warning logs are deliberately not copied into versioned evidence.
The Node checks' later `admission-results.json` and compiled-JS directory are
validation artifacts, excluded from the generator-output comparison.

Host generation medians, nanoseconds per iteration, seven samples:

| Types / metadata construction | Before | After | Change |
| --- | ---: | ---: | ---: |
| 1 / handwritten | 9,325 | 3,106 | -66.69% |
| 1 / derived | 9,873 | 3,698 | -62.54% |
| 100 / handwritten | 864,969 | 322,059 | -62.77% |
| 100 / derived | 932,459 | 364,581 | -60.90% |

These changes remove stderr formatting/writes from the timing harness; they are
not library generation, native codec or IC performance improvements. Old studies
that included diagnostic output keep that historical scope. No IC instruction or
Wasm-size experiment applies because production runtime/generator code is unchanged.
