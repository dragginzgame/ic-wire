# Generated contract fingerprint investigation

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

**Research only; no production identity, encoding, dependency or default changes.**
Prepared on 2 November 2026 from Core/derive 0.15.1 and Endpoint 0.5.0 at
`aa308c9c859687f505cdde33a5d47c96c99ac601`. Preserve the unrelated primary lock
and Types-readiness work. This is one existing Serde/Postcard codec with different
generated constants in isolated endpoint fixtures, not another codec or legacy lane.

## What works

The host prototype exports actual typed `Contract` schemas from all 17 maintained
fixture endpoints, normalizes finite named references, hashes a versioned exact
contract description and emits Rust constants and TypeScript bindings. The
canonical input includes the real export name/mode, ordered request/response
schemas, field/variant labels, bounds, four native request/reply policies and
an encoding-rules revision. Rust type/alias names, registry alias spelling,
current IDs, compiler/package versions and build paths are excluded.

Research encoding: deterministic sorted-key JSON, SHA-256 on the host, first
eight digest bytes as a little-endian u64, carried as a decimal string in JS
metadata. The full digest and descriptor remain in the host manifest. This is
not a reviewed permanent fingerprint specification. Current-generation collisions
and overlap with the prior fixture's keys are refused; 64 bits is not collision-free
or an authentication mechanism. Payload validation remains mandatory.

Generated constants occupy the existing eight-byte identity field. Header magic,
direction, payload encoding, framing length (11 bytes), bounds and runtime
admission code remain unchanged. Clients must regenerate for these new keys;
there is no old-key reader. The existing endpoint `id` expression is only the
prototype's internal attachment point: users would not choose numbers in the
proposed production workflow.

For example, echo's empty-value request header changes from
`49 57 07 00 00 00 00 00 00 00 00` to
`49 57 d3 84 48 66 a4 c7 78 66 00`; the following Postcard payload is identical.
These research bytes are not a new published encoding or an old-key fallback.

The semantic checks cover borrowed/owned equivalence, type aliases, renamed Rust
fields retaining Serde labels, renamed named definitions, reordered fields
(including two equal-typed fields), enum order, generic arguments, field bounds,
response types, method names/modes, policies and encoding-rules revisions. The
Account `Option<[u8;32]>` → optional standard byte adapter changes the key.
Regenerating from a server already containing generated keys produces exactly
the same constants: the current key is deliberately not hashed into itself.

Strict Rust 1.99.1 host lint and generation pass, including 18 semantic checks.
Generated TypeScript compiles
strictly and agrees with Rust on 16 request/reply frames covering u64 zero/max
and empty/full bounded blobs. Both old/new clients refuse the other key; only
header bytes 2–9 change, and frame/payload lengths remain identical. Public
binding metadata preserves hyphenated export names and composite-query mode;
JS declaration labels are separate from routing names.

## Important limits

- **Metadata cannot inspect arbitrary Serde code.** A concrete test changes a
  serializer to the byte adapter while deliberately returning its old manual
  metadata. The fingerprint stays the same although serialized bytes differ.
  The fingerprint protects agreement on a declaration; audited manual metadata,
  Rust–TypeScript byte/refusal vectors and target qualification remain required.
  Hashing a Rust type name, source tokens or a lockfile does not repair this.
- **Recursive graphs are refused by this prototype.** Finite aliases expand
  deterministically under bounded work/depth. Recursive graph normalization needs
  a separately qualified policy; silently using registry/type names would make
  keys depend on aliases. There is no name-only fallback.
- **Policy is conservative.** All four request/reply budgets participate, so
  even a budget-only adjustment invalidates clients. Response changes also
  invalidate requests, stopping stale callers before their handlers can run.
  Freeze the intended policy before publishing an identity specification.
- The schema is host metadata, not proof of target-specific custom serialization
  or Rust memory layout. Host/Wasm qualification and actual admission tests stay.
- Runtime keys are not stored-data migrations, authorization or retry recovery.
  Unframed Account storage and original retained/signature/hash bytes remain
  consumer-owned, as in the Account release preparation.

## Build integration

Use one generated contract artifact for server constants, Rust clients and TS
bindings. CI regenerates it from actual typed signatures, checks its committed
form and byte fixtures, and fails when stale. Deployment builds the Wasm and SDK
from that same admitted artifact. A mismatched key is refused before payload
interpretation; it does not provide Candid's schema evolution/subtyping.

There is a real staging constraint: today's endpoint procedural macro sees type
paths, not nested type definitions or separately implemented serializers. The
existing `Wire` trait and JSON schema projection are host-only. A build script
cannot depend on the application it is currently compiling to obtain its schemas.
The working prototype therefore uses a host contract-export step followed by
Wasm compilation. Previously generated constants let that host step compile,
but generation/checking must be enforced: bare `cargo build` with stale files is
not automatically safe.

For consumers already generating SDK/build evidence, put this step there. For
standalone users, qualify a clear generate/check/build workflow before changing
defaults. Fully transparent single-pass `cargo build` would require moving or
adding cross-target layout metadata and changing trait bounds/derives, including
recursive/custom types. That is more invasive than this host-only approach and
is not implemented or claimed qualified here. No second schema trait is proposed.

The research preparation adds host metadata constants for bare/composed fixture
exports and uses ordinary module discovery. A source rewriter and host-only old→new
mapping adapt this controlled fixture; these are experiment mechanics, not the
proposed consumer API or a compatibility reader in the guest. The actual guest
accepts one key per endpoint and uses the unchanged production codec.
Production therefore also needs a complete endpoint inventory/export step:
optional `Contract` declarations do not currently provide automatic discovery of
every bare or framework-composed export. The research source rewriter is not a
production Rust macro or a proposed consumer API.

## Measurement scope

The paired study uses independent frozen sources and Cargo targets, identical
locks, Rust 1.99.1, wasm32-unknown-unknown, opt-z/opt-3 with fat LTO, one codegen
unit, abort and stripped symbols. The maintained endpoint runner discovers all
17 exports and measures all 178 workloads, including small/large controls,
malformed input, borrowing, async/self-call, guards and rollback. Each case has
one excluded warmup plus eight samples with distinct callers. Physical trace
module hashes, instructions, page costs and observed cycle debits are verified.
The paired inputs intentionally differ in contract-key bytes; payloads, policies,
outcomes and lengths remain matched. These are endpoint guests, not application
linker savings or a new Candid comparison. Sizes are uncompressed total and code
section only; gzip is not an objective.

Use the previously qualified instrumented PocketIC binary, SHA-256
`b930e8bb2d5a1dbed7281a6a8717565c80576fca4c857b5b0a2cabcfc56842ff`,
whose actual version string is **16.1.0**, admitted through Testkit 0.80.1.
The standard cached 16.0.0 server refuses the trace endpoint with HTTP 404;
that failed attempt is retained and excluded, without weakening trace checks.
An initial archive-provenance setup refusal is also retained; no performance
result came from it. Archived input commits are recorded explicitly because
the source copies have no HEAD of their own.

Full raw runs remain in `/tmp/ic-wire-contract-fingerprint-20261102`.
