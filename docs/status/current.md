# Current handoff

## Delivered: Core 0.17.10

Core (`ic-wire-derive`, `ic-wire`) **0.17.10** is on `origin/main` and crates.io.
Tag **core-v0.17.10** selects **f08c249**, from source **555664a**, in Shared
batch **batch-Kkjq3mjE**. All 20 Linux stages and final all-feature archives pass;
registry downloads match uploaded checksums and source. The normal checkout is
reconciled. The [receipt](../../releases/receipts/published-core01710/delivery.json)
owns exact identities. Logs remain under `target/core01710-delivery/` and the
managed checkout under `target/release-frozen/batch-Kkjq3mjE/`. Endpoint remains
**0.7.3**. No cleanup was performed. Hosted CI remains asynchronous.

## Testkit 0.92 adoption

The root requirement and lock select published **Testkit 0.92.0**, aligning its
native `CborError` identity with **Platform 0.24.0**. The older Platform 0.23.6
edge disappears. Wire does not match the changed error identity. All seven native
PocketIC constructors retain explicit 300-second request budgets, independent
startup allowances, typed failures and awaited teardown. Testkit stays outside
published packages. Codec bytes, identities, limits and acceptance are unchanged.
Installed Endpoint typed calls, admission and transaction rollback pass with
PocketIC 16.1.0; logs are under `target/testkit092-adoption/`. This installed proof
preceded the metadata-only version preparation; the captured full Core gate passed.

Current suppliers: **Build 0.78.0**, **Host 0.75.0**, Platform **0.24.0**,
Types **0.18.2**, Runtime/macros/guard **0.19.5**, Testkit **0.92.0**.
Rust/Cargo **1.99.1** remains floor and pin. Root manifests/lock own selection;
Shared **0.31.5 / b2026a1** owns the adopted snapshot. Prepare tools through
`make install-tools`; the release editor remains Build 0.78.0.

## Private checked bool-copy research

Large bool decode uses 37–56% fewer IC instructions and 10/173 fewer uncompressed
Wasm bytes at opt-z/3. Singleton decode adds 85/73 instructions; native opt-3
large decode is 1.8–4.0% slower. The
[matched study](../evidence/primitive-bulk.md#checked-bool-copy-follow-up-7-november)
records all tradeoffs, 14 passing tests, strict Clippy and 2,594 paired IC rows.
Raw artifacts remain under `target/evidence/bool-copy-20261107/`. The study used
Testkit **0.91.0**, before this adoption; do not relabel its measurements.
Public bool support still needs schema/client/budget qualification. Production
bytes, identities, limits and acceptance are unchanged.

## Previous delivered state

Core **0.17.9** is on origin/main and crates.io. Tag **core-v0.17.9** selects
**2568005**, from source **ad51c6b**, in Shared batch **batch-C3zNXsh0**. All
20 Linux release stages and final all-feature archives passed; registry readback
matched source and uploaded checksums. The normal checkout was reconciled.
The [Core receipt](../../releases/receipts/published-core0179/delivery.json) owns
exact identities. Endpoint **0.7.3** remains published; its
[receipt](../../releases/receipts/published-endpoint073/delivery.json) owns its proof.
Hosted CI is asynchronous; local Linux passes do not establish native macOS proof.

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

Next optimization candidates: qualify public bool support, narrow bound wrappers,
and packed 16-byte identifiers. Stress-fixture gains are not application savings.
Consult prior rejected encoder/parser experiments before repeating them.
Future delivery uses explicit groups, Core's full 20 stages, and qualified
Endpoint-only Linux's 11; stage reuse/final-candidate opt-ins remain disabled.
