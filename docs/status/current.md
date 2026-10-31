# Current handoff

## Delivered Core 0.13.1 / Endpoint 0.2.1

The maintainer's delegated Git/crates.io deployment is complete:

- **ic-wire / ic-wire-derive 0.13.1**: core-v0.13.1,
  `974510a88590f2bd54e8ea45de90e0830e928063`.
- **ic-wire-endpoint 0.2.1**: endpoint-v0.2.1,
  `a6cfed5b367fff84997985a357ceeb6731ec5400`.

Source preparation is 203a97c. Both standard patch releases passed full local CI
(Core **673 s**, Endpoint **731 s**), preparation and atomic main/tag delivery.
Publication dry runs and actual uploads succeeded, derive/Core before Endpoint.
All three public non-yanked registry checksums match downloaded/upload archives
and clean VCS provenance matches the exact tags. [Delivery and reproduction](../evidence/published-pair-0131/README.md)
retain the effect/consumer proof. Do not replay these effects. Logs remain under
target/deploy-0131 and the standard .git/release-state plans/validation journals.
No sibling checkout or cleanup was touched. No new pending version is selected.

The recorded Shared snapshot is **0.27.21** at
`143a9abb9ae81feae9400ff3844b03b1ddfdd75a` (upstream commit calls its follow-up
0.27.22 without updating VERSION). Canonical export owns all 119 payloads.
Root Cargo catalogs all six members once, grouped by release with changelog links;
complete alias rosters update private/unused entries too. No extra dependency
edges result from catalog-only entries. Independent release groups and ordinary
whole-target Cargo cleanup remain intact.

## Actual suppliers and acceptance

The release lock selects Platform/macros/guard **0.15.3** (compatible declared
minimum 0.15.2), canonical Types **0.15.1**, Build/Host **0.73.1** and Testkit
**0.75.0**. Platform/macros/guard src matches 0.15.2; installed endpoint limits,
malformed refusals and transaction/reply rollback pass on the actual 0.15.3 source.
The supplying runtime change is qualified separately from older evidence.

Registry-only checks pass on Rust 1.91: four schema-only and five combined endpoint
cases, standalone derives, host/Wasm feature isolation and exact generated
Rust/TypeScript frames/malformed refusals. Schema-only has no runtime; combined
has one guard. Production Wasm excludes host serde_json, Candid, PocketIC,
external executor and fuzz/profiler tooling. No patches/local Wire supplier.

The typed registry consumer also passes all four native Metrics 0.24 schemas,
Principal/Account, Contract and Testkit 0.75 public Limits/Error plus typed
query/update signatures with one Core/Derive/Types identity. Testkit's separate
Metrics 0.23.2 remains arithmetic-only. Application adoption is consumer-owned.

[Endpoint-release hosted CI](https://github.com/dragginzgame/ic-wire/actions/runs/41227760151)
was still in progress at inspection; the earlier Core run was cancelled by the
subsequent push. Neither that nor the final documentation push is claimed as a
complete hosted matrix. The two real local release gates above both passed.

## Closed issues and maintained boundaries

#36 is closed: stable Core/derive/schema ownership and independent Endpoint are
published and the former supplying/native acceptance is supplied. #39 is closed
and shipped in Core 0.13.1: fuzzing belongs to private qualification's explicit
native fuzz-runtime feature; heap/allocations select optional native profilers.
Ordinary codec/benchmark/qualification graphs exclude them. A maintained graph
gate enforces default exclusions, explicit selection and Wasm isolation.
[Focused evidence](../evidence/qualification-tools-0131/README.md) retains 49 codec
checks, four qualification tests, 1,000 address-sanitized fuzz executions, both
profiler smoke tests, conflict diagnostics and Clippy/Rust 1.91. Leak shutdown was
explicitly disabled; no fresh macOS/fuzz hosted run is claimed. #37 stays superseded.

The [approved boundary](../stable-identities.md) keeps one Serde/Postcard codec,
Limits/Error, the sole schema trait and canonical Types impls in Core. Endpoint
owns Contract/attributes/admission/trapping replies; Platform owns execution.
No compatibility wrapper, alternate encoding, schema mirror or runtime is added.
Protocol bytes, field order, budgets, primitive/malformed acceptance, rollback and
original retry/hash inputs are unchanged. Principal stays bounded bytes max 29;
Account absent `01 04 00` and explicit zero `01 04 01` plus 32 zero bytes stay
distinct. Collection/custom-Serde normalization remains consumer-owned.

No runtime speed, Wasm-size or build-time saving is claimed for this batch.
[Earlier artifact qualification](../evidence/platform0152-testkit074/README.md)
retains its original source/version measurements. The prepared PocketIC server
lacks the instruction-trace API; no fresh full instruction/Candid matrix ran.

Root Cargo is virtual with literal member versions. Root CHANGELOG indexes the
[core](../../releases/core/CHANGELOG.md), [endpoint](../../releases/endpoint/CHANGELOG.md)
and [historical](../../releases/history.md) ledgers. Future delivery needs its own
scoped authorization. Evidence Cargo manifests remain inactive snapshots; Candid
stays confined to unpublished qualification dependencies. Existing
[Candid matrix](../evidence/candid-comparison-0725/README.md) and
[extraction qualification](../evidence/stable-identities-0120/README.md) retain
their identified inputs and limits.
