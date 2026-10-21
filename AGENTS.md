# IC Wire contributor instructions

Read docs/status/current.md first, then README.md, docs/design.md and
 docs/evaluation.md. The Postcard runtime is qualified for bounded
Rust integration; see docs/runtime.md for its implemented contract.

Apply the reviewed [Shared Tooling baseline](DRAGGINZGAME.md) and its referenced
rules, vendored at `85aded191d85ac4a8c0e5deab207f6be18895862` (0.25.9).
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
- Keep runtime code Wasm-safe and independent of platform SDKs and host tooling.
- Keep all workspace dependency roles limited to codec qualification. Do not add
  endpoint SDKs, simulators or browser transports; integration belongs to consumers.
- Use one selected wire encoding, not a configurable codec framework.
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
Alternative-codec comparison suites are retired; retained raw Postcard probes are
diagnostic controls, not production modes. Siblings are read-only unless separately
authorized.

Use focused checks for changed code. Check for an active build sharing target/
before compiling. Do not run broad Canic or sibling suites. Keep compiler, target,
profile, input identity and all codec parameters with measurements. Distinguish
native time, IC instructions, linked code size and compressed deployment size.
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
wire identity independent of incidental Rust declaration order. Preserve exact
retry bytes and canonical hashing semantics. The maintained pre-1.0 contract
must not create v2/legacy lanes; same-release retries remain required. Do not
introduce automatic migration or compatibility shims.

Update the open changelog and compact handoff for meaningful batches. Keep
proposed syntax explicitly labeled until it compiles and passes its owning tests.
