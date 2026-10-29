# Current handoff

## Released 0.7.22; pending 0.7.23 IC Tooling adoption

HEAD is `ac08fdc1757b7d3f1483caecadd7ac1c817a30d8` / release 0.7.22.
Own package versions stay 0.7.22; the 0.7.23 changelog is a draft. Leave staging,
commits, version changes, release and publication to the maintainer. The working
tree was clean at the start of this adoption. No workflow was dispatched.

The earlier [0.7.21 CI run](https://github.com/dragginzgame/ic-wire/actions/runs/40726709588)
passed all nine qualification jobs on Linux/macOS and three final gates; it is
historical evidence, not qualification of the pending graph. No hosted result for
0.7.22 was checked during this update.

## Current dependency adoption

Root Build/Testkit minimums and the managed Build release editor now select
0.70.0, published from `ecc8d61d19f0b711788f7e10e5dd728d0db6ff60`.
Platform remains 0.8.1. Testkit now selects that same facade, removing the older
Platform 0.7.6 copy. The lock changes only these IC Tooling packages, their host
helpers and required Platform edges/features; other selections are preserved.
The incoming lock is `target/tooling-adoption-0723/incoming.Cargo.lock`.

Tooling's breaking changes concern Platform-owned ledger values and Registry
principals, which Wire does not exchange. Testkit explicitly enables external
Principal conversion and preserves the PocketIC API used here. Existing caller
source and feature selections need no migration. Build stays default-features
false; only the managed editor installation selects `cli`. Rust 1.91 remains
the floor. This qualification-only update leaves published Wire APIs, runtime
bytes, identities and admission unchanged; the pending patch draft is valid.
Shared Tooling remains 0.27.3 at `7ee5c83744aa822ecd070703dbf0a7d95076d5f6`;
its snapshot files are unchanged.

Focused evidence is under `target/tooling-adoption-0723/`: diagnostic caller
Clippy, nine compile-fail cases plus control, three corpus/boundary/Nat tests,
strict benchmark all-target Clippy with both codecs and IC host callers,
Rust 1.91 Build/Testkit caller checks, managed editor admission, disposable
release-adapter execution and dependency/documentation checks. `receipt.json`
records commands and source/lock hashes. No fresh PocketIC execution, codec
performance/size matrix, full CI or native macOS run is claimed for this batch.
There is no production codec/generator cost change or measured saving.

## Released SDK and CI behavior

Canonical SDK acceptance and consumer boundaries are recorded in
[#29 — Payload-only client codecs and bounded caller storage for SDK integration](https://github.com/dragginzgame/ic-wire/issues/29#issuecomment-6475585612).
The registry 0.7.21 consumer qualified scoped storage/fallback, short-storage and
admission refusal, unit/tuple/framed identities, standalone schema validation and
strict generated TypeScript exact bytes/malformed refusal. Evidence remains in
`target/issue29-0721/`; this was not installed endpoint/rollback qualification.
The [SDK recipe](../sdk-integration.md) keeps transport, copying, authentication,
persistence, retries and deployment consumer-owned.

Hosted CI partitions the fixed 17-gate roster into isolated core/TypeScript/
benchmark jobs on each supported OS. Existing scaffold statuses require every
job; local/release CI keeps its serial roster. Released evidence remains under
`target/parallel-ci-0721/` and `target/parallel-suite-audit/`. No CI wall-time
saving is claimed. The synthetic generator consumes bulk advice without warning
spam while ordinary warnings remain visible; its
[warning study](../evidence/generator-warnings-0721/README.md) records identical
output and warning/failure controls.

`native::with_encoded` keeps bounded two-pass encoding, scratch/owned fallback,
synchronous callback lifetimes and no callback on failure. Its
[study](../evidence/with-encoded-0721/README.md) retains instruction gains alongside
Wasm and small-input/native regressions. Host-only `validate_payload_schema`
reuses complete registry/root checks; its
[study](../evidence/schema-validator-0721/README.md) records validation versus
generation cost and unchanged generated output, not a production codec speedup.

## Comparison boundaries

Platform's bounded Candid nesting cap remains 16 constructed levels. The paired
corpus uses `tree-depth-7`; depth 8 explicitly proves Wire success and typed
Candid refusal. Historical depth-8/depth-7 rows have different inputs and must not
be paired; old reports also exclude the later Candid admission work.

Prior adoption evidence stays under `target/platform-adoption-0722/`,
`target/ic-adoption-06912/`, `target/ic-adoption-06911/` and
`target/ic-adoption-0698/`, bound to those graphs. SDK studies freeze
Build/Testkit 0.69.6 / Platform 0.7.3. Historical
[caller-storage](../evidence/caller-storage-0719/README.md),
[runtime](../evidence/runtime-0716/README.md) and
[encoder](../evidence/encoder-followup-0716/README.md) studies retain their inputs.
Installed SDK qualification remains consumer-owned.
