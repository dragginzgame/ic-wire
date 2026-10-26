# Current handoff

## Released 0.7.13; pending 0.7.14

The checkout started clean at `b2f2f2e` (Release 0.7.13). This batch adds explicit
package selection to the local registry publication recipe after a partial upload.
Runtime, generator, protocol bytes and acceptance rules are unchanged. Own versions,
staging, commits and actual publication remain maintainer-owned.

## Partial publication retry

The reported crates.io response says ic-wire-derive 0.7.13 already exists and
Cargo stopped before publishing ic-wire. Keep the released versions and select
only the remaining package:

```sh
make publish PUBLISH_PACKAGES=ic-wire
```

The default selects both public crates. Every selection retains the lint gate;
empty selections and private/unknown package names fail before Cargo runs.
Registry state is not inferred automatically, and upload errors are not suppressed.
The operator should confirm the already uploaded derive crate is the intended
release. If the runtime cannot resolve that version, wait for index propagation.
A read-only sparse-index response still showed 0.7.12 during this inspection;
it does not override the registry's reported already-uploaded response.
No live publication, release, staging or commit was performed in this batch.

## Validation

The disposable local Make fixture passes: both packages and either individual
package route to the expected locked Cargo command, lint refusal prevents uploads,
and invalid selections invoke no Cargo. Cargo is a stub for these checks, so
neither the real checkout nor a registry receives publication effects. Formatting,
shell syntax, documentation links and whitespace checks pass. Runtime/codec
benchmarks are not applicable to this publication-recipe-only change; full CI
was not rerun. See [publication recovery](../development.md#first-publication).

The released [method identity fix](../evidence/method-ids-0713/README.md),
[unsigned evidence guide](../unsigned-magnitude.md) and
[full comparison](../full-comparison-0710.md) retain their own qualification scopes.
Installed Canic/Toko adoption and application Wasm headroom were not requalified.
