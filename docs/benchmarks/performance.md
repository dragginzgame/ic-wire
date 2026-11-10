# Performance measurements

Use the maintained runners described in [development commands](../guides/development.md)
and the [evaluation contract](evaluation.md). Results describe their recorded
source, inputs, compiler, lock and profile; they are not application guarantees.

| Question | Measurement owner |
| --- | --- |
| Native codec time, allocations and heap | `evaluate-codecs.sh` and `evaluate-allocation` |
| Wire/Candid instructions and uncompressed Wasm | `evaluate-codec-matrix.sh` with the prepared PocketIC server |
| Endpoint and input-borrowing costs | `evaluate-endpoint-matrix.sh` and `evaluate-input-borrowing.sh` |
| Host schema-generation cost | [Generator probe](../../scripts/benchmarks/generator/mod.rs) and the [matched study procedure](evaluation.md#host-schema-rendering-suppression-8-november-2026) |

See the [latest comparison](candid-comparison.md) for completed matrix results
and the [topic findings index](../evidence/README.md) for retained and rejected
experiments. In particular, the [encoder studies](../evidence/encoder-followup.md)
record current candidates and their regressions; do not infer adoption from an
older summary.

Opt-z is the primary size-constrained profile; opt-3 remains a regression check.
Report native time, IC instructions, total uncompressed Wasm and code-section
bytes separately. Preserve small controls and absolute repeated results. Raw
artifacts belong under `target/evidence/`; summaries cannot verify retired files.

Alternative-codec runners and finite-codec probes are retired. Their commands
and results belong in [history](../history/README.md), not the current workflow.
