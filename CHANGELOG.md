# Changelog

## [0.23.5] - 2026-11-15

Versioned evaluation checkpoint; not published to crates.io.

### ✨ Added

- Add optional strict metadata derivation for ordinary structs/enums and field
  adapter projections. Generate the representative payload client directly from
  its Rust type, admitting the result through the existing validated graph.
  Support concrete type/const generics and borrowed fields without a static
  lifetime requirement. Reject unsupported Serde layouts and recursive projection;
  preserve the explicit named-metadata path for recursive schemas.

## [0.23.4] - 2026-11-15

Versioned evaluation checkpoint; not published to crates.io.

### ✨ Added

- Generate typed request/response codecs with duplicate export/ID admission,
  exact u64 method identities and separate directional policy. Share module
  metadata and runtime admission with payload generation. Exercise emitted
  frames, asymmetric request/response schemas and the maximum method ID in Rust
  and the reference TypeScript runtime.

## [0.23.3] - 2026-11-15

Versioned evaluation checkpoint; not published to crates.io.

### ✨ Added

- Generate executable typed payload clients from the validated host graph, with
  private metadata, explicit browser limits, bounded total source and runtime
  format admission. Exercise the emitted client in representative Rust exchange,
  including policy observation and mismatch refusal before codec creation.

## [0.23.2] - 2026-11-15

Versioned evaluation checkpoint; not published to crates.io.

### ✨ Added

- Qualify wide integer and UTF-8 exchange with the published reference TypeScript
  runtime. Cover literal extremes, BigInt/range refusal, BOM preservation,
  malformed UTF-8, byte-counted text bounds and retained noncanonical integers.

## [0.23.1] - 2026-11-15

Versioned evaluation checkpoint; not published to crates.io.

### ✨ Added

- Qualify recursive records and shared named schemas through the candidate graph
  and published reference TypeScript runtime, with literal bytes, value custody
  and recursive depth-limit refusal.

## [0.23.0] - 2026-11-15

Initial evaluation checkpoint; not published to crates.io.

### ⚠️ Breaking

- Evaluate the next minor version after published ic-wire 0.22.4 with root-level
  codec APIs and explicit Encoding selection. This incomplete replacement remains
  unpublished; consumer migration and release qualification are still pending.

### 🛠️ Tooling

- Initialize the virtual Rust workspace with compiler 1.100.0 and shared lint
  policy. Install Toko Tooling 0.44.3 and expose its dependency list/update and
  tool setup commands. The real core package and lockfile enable Cargo admission.

### ✨ Added

- Export bounded reference-format JSON from the admitted host graph. Qualify
  representative payload/request/response byte exchange between candidate Rust
  and the published reference TypeScript runtime, including selected-span custody,
  truncation categories and byte/node/depth boundaries. Check request and response
  allowances independently without changing codec policy.
- Add optional host-only metadata admission into an immutable typed registry and
  bounded TypeScript declaration rendering. Validate references, alias cycles,
  names and constraints once; retain productive recursion and shared references.
  This initial subset does not yet generate a browser codec or collect Rust types.
- Start the portable core with method-ID derivation and fixed request-header
  validation, preserving reference identities and typed rejection behavior.
  Add literal contract vectors and native/Wasm development checks.
- Add bounded Postcard decoding through one payload/frame API, preserving
  borrowed input, reference traversal accounting and primitive acceptance.
  Check resource boundaries, hostile counts and typed failure precedence.
- Add shared encoding preflight for owned, reusable and slice output, with
  Postcard primitive delegation and failure-custody tests. Preserve frame bytes
  and reference traversal charges without private integer or enum-tag writers.
- Add bounded bytes, UTF-8 text, ordinary lists and input-backed borrowed views.
  Preserve field error semantics and test literal record encodings, ownership,
  borrow lifetimes and refusal before decoding excess list elements.
- Add explicit bounded packed i16 fields and a native representative record
  with ordinary-vector controls. Check little-endian bytes, element bounds,
  unaligned decoding, owned custody and bulk admission without unsafe code.
- Add a separate native compatibility workspace against published ic-wire
  0.22.4, with exhaustive short-primitive checks, representative budget sweeps,
  frame mutations and output-custody comparisons.
- Add isolated allocator qualification for preflight/output ordering, bounded
  field rejection and borrowed decoding, with successful-allocation controls and
  explicit coverage of incremental decoding and custom Serde allocations.
