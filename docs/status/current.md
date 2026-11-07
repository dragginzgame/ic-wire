# Current handoff

## Delivered: Core 0.17.12 and Endpoint 0.8.0

All four public crates are on origin/main and crates.io. Core tag
**core-v0.17.12** selects **6082752**, from source **558897d**, in batch
**batch-FcrckPoz**. Endpoint tag **endpoint-v0.8.0** selects **a967a9c**, from
Core's release commit, in batch **batch-vZAxlP49**. Separate frozen batches
were required for the patch/minor increments; both completed fresh full
20-stage Linux gates. Final all-feature archives pass and independent registry
readback matches every uploaded archive and release identity.

The final Endpoint 0.8.0 / Core 0.17.12 fixture also passes installed typed calls,
admission and rollback on PocketIC 16.1.0. The normal checkout is reconciled;
there is no pending source change or release. The
[delivery receipt](../../releases/receipts/published-core01712-endpoint080/delivery.json)
owns exact identities, package checksums and qualification. Logs are under
`target/core01712-endpoint080-delivery/`; retain both managed checkouts under
`target/release-frozen/`. Hosted CI is asynchronous; local Linux passes are
not native macOS proof. No cleanup was performed.

## Runtime adoption contract

Endpoint 0.8 selects Runtime/macros/exact guard **0.20.0**. Align Timers and other
execution suppliers before adopting it; the singleton guard rejects mixed
execution generations. Wire already uses the retained ordinary `cdk` macro
family. Direct Runtime users replace removed `runtime_*` aliases with ordinary
`ic_platform_runtime::cdk` macros. Wire frame bytes, schemas, method identities,
budgets, acceptance, export names and rollback rules remain unchanged. No client
regeneration or data migration is required by this supplier adoption.

Benchmark Platform **0.25.0** now delegates IC0 to Runtime 0.20, removing the old
Runtime 0.19.5 entry. Testkit **0.92.0** retains native-only Platform **0.24.0**
without execution features. Production Endpoint has one Runtime/guard, while
schema-only Core remains execution-free. No supplier patch or wrapper is needed.
Focused adoption logs remain under `target/runtime-adoption/` and
`target/platform-adoption/`; do not relabel older performance measurements.

## Compiler, suppliers and release boundary

Rust/Cargo **1.99.1** remains floor and pin. Current selections are Build **0.78.0**,
Host **0.75.0**, Platform **0.25.0**, Types **0.18.2**, Runtime/macros/guard
**0.20.0** and Testkit **0.92.0**. Root manifests/lock own exact selection.
Shared **0.31.5 / b2026a1** owns the adopted snapshot. Prepare tools through
`make install-tools`; the release metadata editor remains Build 0.78.0.

## Public bool adapter and retained measurements

Core 0.17.11 introduced explicit `serde_packed::bools` for `Vec<bool>`, generated
`boolean[]` clients and default advice. Adoption preserves `02 00 01` for
`[false,true]`, but requires client regeneration and bounds/budget review because
traversal becomes one buffer. Existing ordinary fields and Packed invariants stay
unchanged. The [paired study](../evidence/primitive-bulk.md#public-bool-adapter-for-01711)
preserves every Wire IC counter and all allocations against the private prototype;
fixture Wasm adds 35/16 total and 19/0 code bytes at opt-z/3. Native opt-3 large
decode regresses 1.75–2.02× against that prototype, despite unchanged IC costs.
Raw artifacts remain under `target/evidence/public-bools-20261107/` and
`target/evidence/bool-copy-20261107/`. These studies retain their original
suppliers. Float bulk support remains private research. No new speed claim.

## Evidence and recovery custody

The complete **36-cohort** matrix measures Core 0.17.6 working source with
Testkit 0.89.4 and its original supplier graph. The
[current comparison](../candid-comparison.md) owns 2,105 case entries, 8,420 primary
comparisons, 144 fresh modules and limitations. Preserve both explicitly protected
paths: `target/evidence/codec-matrix-20261107/` and
`target/codec-matrix-build-20261107/`. Cleanup interrupted one cohort; `recovery-1/`
retains the verified resume and prepared PocketIC 16.1.0 binary. The completed
matrix retained earlier probes and reran only missing work. No cleanup is authorized.

Keep release journals and retained captured checkouts. Core 0.17.9 logs remain
under `target/core0179-delivery/`, with its checkout under
`target/release-frozen/batch-C3zNXsh0/`. Prior receipts under
[releases/receipts](../../releases/receipts/README.md) retain their original inputs.
The failed **batch-1afAufRB** had no delivery effects and is superseded; do not
resume it. Completed **batch-1i9z2IIG** was reconciled, but a separate cleanup
removed its checkout; its journal does not prove checkout availability. Its
deferred lock was preserved at **7fd813f** on `deferred/core0176-lock` and reapplied.
The completed schema-1 0.17.5 batch is historical; never resume it with schema 2.
Repository history was intentionally replaced: fresh-clone rather than merge
obsolete refs, and never re-upload published registry versions to repair history.

## Contracts and next work

Public `serde_packed::nonzero_i64` / `heap_u64` shipped in 0.17.6. Adoption from
ordinary lists changes bytes and admission: `[-1_i64]` changes `01 01` to
`08 ff ff ff ff ff ff ff ff`; heap `[1]` changes `01 01` to
`08 01 00 00 00 00 00 00 00`. Coordinate clients and budgets; record-field
allowances can differ from Packed DTOs. Retain original retry/hash bytes because
collection/custom-Serde normalization remains consumer-owned. See
[consumer guidance](../../crates/ic-wire/README.md). Big-endian execution is unqualified.

Account's [0.16 cut](../../releases/core/0.16.md) remains `01 20 + 32 bytes` for
present subaccounts, `00` for absent ones. Method IDs/name defaults remain
production behavior; fingerprint/no-manual-ID macros are research. Keep one
Serde/Postcard codec and no automatic consumer Serde rewriting. Metadata advice
is not universal lint coverage for codec-only/custom layouts.

Next optimization candidates: native bool scan code generation, narrow bound wrappers,
and packed 16-byte identifiers. Stress-fixture gains are not application savings.
Consult prior rejected encoder/parser experiments before repeating them.
Future delivery uses explicit groups, Core's full 20 stages, and qualified
Endpoint-only Linux's 11; stage reuse/final-candidate opt-ins remain disabled.
