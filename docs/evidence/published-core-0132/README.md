# Published Core 0.13.2

2026-10-31. The maintainer's deploy request authorized scoped Git delivery and
crates.io publication. [Identities](release-identities.json) record source
preparation 1029a8f and release eff8c18, tagged core-v0.13.2. The atomic
[remote delivery](remote.txt) pushed main and exactly that tag. Endpoint remains
published 0.2.1; no Endpoint release or upload was performed.

The standard release passed the complete real local `make ci` gate in **532 s**;
[validation summary](validation-summary.log) names every target. The
[publication dry run](publish-check.log) and [upload](publish.log) both succeeded,
with derive supplied before Core and Cargo waiting for registry availability.
[Registry proof](registry-proof.json) matches both non-yanked public checksums
against downloaded and local upload archives, and verifies clean VCS provenance
at the exact release commit and expected crate paths. Core still excludes the
retired libfuzzer-sys dev dependency.

## Published-consumer qualification

[Executed log](registry-consumer.log) uses only registry packages, without patches
or a local Wire supplier. Rust 1.91.0 passes four standalone schema tests, five
combined Endpoint tests, both Wasm checks and one typed schema/API test. Generated
Principal/Account Rust and TypeScript bytes and malformed-input refusals agree.
Schema-only excludes Platform execution/macros; combined selects one guard.
Production Wasm excludes Candid, PocketIC, external executor, host serde_json and
fuzz/profiler tools. Core/derive resolve once at 0.13.2; Endpoint is 0.2.1 and
canonical Types is 0.15.1. The typed consumer also resolves exactly one Testkit
0.75.1 and Metrics 0.24.1 and composes all four native Metrics schemas and typed
Testkit call signatures. This does not execute remote typed Testkit calls.

Frozen manifests/locks and the [runner](qualify-published.sh) retain exact
selections. The runner reuses the earlier frozen schema/TypeScript fixtures and
the tooling adoption's typed fixture, rather than duplicating their source.
With those registry sources, Rust 1.91/Wasm and pinned browser tools prepared:

```sh
bash docs/evidence/published-core-0132/qualify-published.sh /tmp/wire-published0132-repeat
```

The runner installs nothing and has no Git, release or upload effects. Full local
release/publication outputs remain in target/deploy-0132 and the standard
.git/release-state plans and validation journals. No sibling checkout was modified.
Hosted Linux/macOS CI is separate from the passed local gate; consult the current
handoff for its observed status.

## Scope and performance

[Supplier adoption evidence](../tooling0732-metrics0241/README.md) covers
Build/Host 0.73.2, Testkit 0.75.1 and Metrics 0.24.1, managed editor admission and
installed endpoint limits/rollback. The production dependency graph and codec/
generator source are unchanged. No new runtime benchmark applies to this private
tooling selection; no instruction, Wasm-size or build-time saving is claimed.

Protocol bytes, field order, budgets, primitive/malformed acceptance, handler
rollback and original retry/hash inputs are unchanged. Principal stays bounded
bytes max 29. Account absent `01 04 00` and explicit zero `01 04 01` plus 32 zero
bytes remain distinct. Collection/custom-Serde normalization stays consumer-owned.
