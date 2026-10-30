# IC Wire contributor instructions

Read [the current handoff](docs/status/current.md) and this overlay first.
Use README.md for orientation when needed, docs/design.md for codec/architecture
changes, docs/runtime.md for runtime contracts and docs/evaluation.md for codec
qualification work. These guides are task-specific, not a startup reading list.
The Postcard runtime is qualified for bounded Rust integration.

Apply the reviewed [Shared Tooling baseline](DRAGGINZGAME.md) and its referenced
rules, vendored at `62edfbd43087a718bf0283e9040676a49211976d` (recorded version 0.27.11).
[.shared-tooling.snapshot](.shared-tooling.snapshot) records the exact selection,
hashes and modes. This file is the IC Wire overlay. Do not infer policy from a
moving sibling checkout or patch snapshot-owned files in place. The main snapshot
also owns the release-receipt helper; no separate contribution snapshot remains.

## Maintainer decisions

- ic-wire and ic-wire-derive were first published at 0.3.0.
  Keep the qualification package `publish = false`. Subsequent publication,
  remote creation, pushes and releases require explicit authorization.
- Establish a reusable Rust developer experience and a substantial measured win
  before broad adoption. Postcard is selected for controlled RPC and owned user
  endpoints. Preserve the explicit runtime contract in docs/runtime.md.
- Keep the core codec Wasm-safe and independent of platform SDKs and host tooling.
  The optional `endpoint` feature connects typed Wire handlers to published IC
  Platform entrypoints; Platform owns exports and execution. Canic policies and
  artifact admission remain downstream. Do not introduce another executor.
- Keep other workspace dependency roles limited to codec qualification. The unpublished
  benchmark package may use Candid and PocketIC through IC Platform as dev
  dependencies, with IC Testkit owning server admission/lifecycle. The unpublished
  standalone endpoint fixture qualifies the optional integration through the same
  owners. Do not add browser transports; application policy belongs to consumers.
- Use one selected wire encoding, not a configurable codec framework.
- Default new contract examples to byte-oriented fields for binary data and
  `Packed<T>` for large homogeneous supported integer/hash arrays. Keep ordinary
  collections for general records and small-value varint use cases. Preserve
  explicit field bounds; existing declarations require coordinated adoption.
  Keep ordinary-vector benchmark controls visible; do not silently rewrite Serde.
- Canic, Relay and IcyDB retain their own contracts and runtime semantics.
- IcyDB database storage/key encodings are outside initial scope.
- Maintainer-operated releases use direct Git delivery to origin/main through
  Shared Tooling. Versioning, tags and pushes do not publish packages.
  Ordinary agent work does not execute releases.

## Development and validation

See [local development](docs/development.md) for explicit setup, commands and the
supported host matrix. The shared [validation policy](DRAGGINZGAME.md#validation-and-evidence)
owns execution cadence. Select focused Make targets for changed behavior;
`make ci` is the complete scaffold gate, not an ordinary handoff prerequisite.
The local Make adapter fixture uses disposable repositories only; it must never
invoke release effects in the real checkout.

## Implementation discipline

Use the virtual root Cargo workspace, crates/ic-wire for the library, inherited
package metadata/lints and root workspace.dependencies for future dependencies.
Use directory modules with mod.rs. No placeholder public APIs or speculative
crate splits. A derive crate can be added when the selected design needs it;
additional experimental crates must remain unpublished.

Preserve dirty work. Ordinary work leaves staging, commits, own versions and
releases to the maintainer. Maintain runtime/native codec tests, compile-fail
diagnostics and standalone Rust/TypeScript conformance. The unpublished
qualification package owns allocation/borrowing fixtures and compiler diagnostics.
The unpublished `ic-wire-benchmarks` package compares Candid through an
`ic-platform` dev dependency. Codec comparisons and raw Postcard probes are
qualification controls, not production modes. Keep Candid out of published crates.
The benchmark package denies unsafe code except its explicitly scoped IC export
attribute; the entrypoint uses safe system-call wrappers. Other packages inherit
the workspace's unsafe-code prohibition.

Use focused checks for changed code. Check for an active build sharing target/
before compiling. Do not run broad Canic or sibling suites. Keep compiler, target,
profile, input identity and all codec parameters with measurements. Distinguish
native time, IC instructions, Wasm code-section size and total uncompressed Wasm size.
Optimize one codec strategy for code-section pressure and total uncompressed Wasm
size, measuring instruction tradeoffs. Gzip size is not an optimization objective
or acceptance criterion; historical/compressed measurements may remain as evidence
but must not influence candidate selection or headline improvement reports.
Do not turn symbol attribution into a savings claim.

For changes affecting runtime or generator cost, include a reproducible before/after
benchmark in the handoff. Measure the affected path against an identified baseline
with identical inputs, dependencies and settings; include small-input controls,
repeated samples, absolute results, percentage changes and regressions. Keep the
runner and evidence reproducible, and label host generation, native codec time,
IC instructions and artifact sizes separately. If measurement is blocked or not
applicable, state why; never substitute an asymptotic claim for measured savings.
Documentation-only changes do not require benchmarks.

Every protocol-affecting change must include an inline explanation beside the
changed encoding/decoding or contract logic: the old and new behavior, wire-byte
or acceptance-rule difference, and compatibility/consumer action. Also summarize
that change in the changelog and final handoff, with a concrete before/after byte
or schema example when applicable. Cover framing, identities, payload encodings,
canonicalization, hashing, limits and malformed-input acceptance; explicitly state
when a batch leaves the protocol unchanged. Preserve the pre-1.0 rules below.

Keep limits checked before allocation, malformed-input failures typed, and
one production Serde/Postcard codec. The finite Wire API is retired; alternate
codec/derive strategies remain research. Native metadata uses no field IDs and follows Serde
declaration order; changes to that order require coordinated consumers. Preserve
exact retained retry bytes and encoder hash inputs. Decode validates
primitive Postcard representations in one pass; collection/custom-Serde
normalization is consumer-owned and re-encoding need not reproduce input bytes.
The maintained pre-1.0 contract
must not create v2/legacy lanes; same-release retries remain required. Do not
introduce automatic migration or compatibility shims.

Update the open changelog and compact handoff for meaningful batches. Keep
proposed syntax explicitly labeled until it compiles and passes its owning tests.
