# IC Wire contributor instructions

Read docs/status/current.md first, then README.md, docs/design.md and
 docs/evaluation.md. The Postcard runtime is qualified for bounded
Rust integration; see docs/runtime.md for its implemented contract.

Apply the reviewed [Shared Tooling baseline](DRAGGINZGAME.md) and its referenced
rules, vendored at `879eb2b021522252a67e908dbc217a0f4efb2a6f` (0.15.4).
[.shared-tooling.snapshot](.shared-tooling.snapshot) records the exact selection,
hashes and modes. This file is the IC Wire overlay. Do not infer policy from a
moving sibling checkout or patch snapshot-owned files in place.

## Maintainer decisions

- ic-wire and ic-wire-derive were first published at 0.3.0.
  Keep the evaluation package `publish = false`. Subsequent publication,
  remote creation, pushes and releases require explicit authorization.
- Establish a reusable Rust developer experience and a substantial measured win
  before broad adoption. Postcard is selected for controlled RPC and owned user
  endpoints. Preserve the explicit runtime contract in docs/runtime.md; no Candid
  mode is part of the production API.
- Keep runtime code Wasm-safe and independent of Candid, CDK and host tooling.
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
releases to the maintainer. The accepted next task is the bounded evaluation in
 docs/evaluation.md, not cross-repository adoption. Siblings are read-only unless
separately authorized; prototype consumer-shaped fixtures here instead.

Use focused checks for changed code. Check for an active build sharing target/
before compiling. Do not run broad Canic or sibling suites. Keep compiler, target,
profile, input identity and all codec parameters with measurements. Distinguish
native time, IC instructions, linked code size and compressed deployment size.
Do not turn symbol attribution into a savings claim.

Keep limits checked before allocation, malformed-input failures typed, and
wire identity independent of incidental Rust declaration order. Preserve exact
retry bytes and canonical hashing semantics. The maintained pre-1.0 contract
must not create v2/legacy lanes; same-release retries remain required. Do not
introduce automatic migration or compatibility shims.

Update the open changelog and compact handoff for meaningful batches. Keep
proposed syntax explicitly labeled until it compiles and passes its owning tests.
