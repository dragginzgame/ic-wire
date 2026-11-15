# Changelog

## [0.23.9] - 2026-11-15

GitHub evaluation patch requested by the user; not published to crates.io.
The intended first public hard-cut release remains 0.24.0.

### ✨ Added

- Simplify the checked packed-word conversion so opt-z can eliminate a per-word
  panic-helper call. Keep one generic decode path and all preallocation bounds.

- Add private native codec hotspot benchmarks with selected ordinary/bulk
  workloads, separate timing/allocation builds, paired supplier samples and a
  symbol-bearing profiling loop. No benchmark dependency enters product crates.
  Extend investigation to i32/u64 varint boundaries and nested request shapes.

- Consolidate packed fields into `Packed<T, N>` with an optional element bound
  and one fallible constructor. Remove `PackedI16` and its special schema/runtime
  branch; every supported word kind now uses the same bounded representation.

- Prepare both product crates for registry publication with package guides,
  repository metadata, extracted-archive consumer checks and a Chromium smoke
  test of the shipped TypeScript runtime. The 0.23.8 coordinated publish dry run passed;
  publication remains explicitly deferred.

## [0.23.8] - 2026-11-15

Versioned evaluation checkpoint; not published to crates.io.

### ✨ Added

- Add scalar bounds and named synchronous browser validators through the shared
  schema graph. Keep one current, unversioned schema/runtime interface for the
  authorized hard cut; legacy compatibility is not a release requirement.

- Add explicit bool, finite-float, nonzero-i64 and heap-u64 bulk field adapters,
  sealed derive projections and matching browser validation. Qualify bytes,
  admission boundaries, allocation ordering and heap normalization against the
  reference; lint advice now names the available bool/float adapters.

- Add rooted bulk advice over the validated graph, with exact acknowledgements,
  stale-path detection, bounded recursive traversal and bounded diagnostic display.
  Payload/method client generation requires a diagnostic handler; the standalone
  consumer prints warnings and supports `--deny-warnings` before artifact output.

- Preserve standard Result, fixed-array, tuple, slice and Cow text schema layouts,
  including tuple structs and multi-field enum variants, across Rust and both
  TypeScript runtimes. Fixed arrays and tuples carry no sequence-length prefix.
- Expose published ByteBuf/ByteArray types and sealed standard serde_bytes field
  projection, retaining ordinary-array versus bulk-byte distinctions.
- Add optional host schemas for published Platform Types 0.20.4 Principal and
  Account, preserving absent versus explicit-zero subaccount bytes.
- Add finite float wrappers and browser projection, preserving finite IEEE-754
  bits and refusing nonfinite values and inexact browser f32 inputs.
- Add explicit packed integer and 16/32-byte identifier fields over the shared
  bounded-i16 bulk path, with owned decoding and partial-word rejection.

These are local evaluation additions, not a production migration or performance
acceptance. Browser/distribution and matched performance qualification remain;
see [the current parity ledger](docs/design/parity.md). The standalone Wasm size
gates passed at both optimization levels after sharing the packed implementation;
that measurement predates the subsequent lint, domain-adapter and validation work.

### 🐛 Fixed

- Keep qualification helper modules out of Cargo's standalone integration-test
  discovery; the three explicit test binaries retain their existing ownership.

### ▶️ Try it

Run `make check-packages`, then use the generated client directory it prints:
`make check-browser CLIENT_DIR=/absolute/path/to/run/client BROWSER=/absolute/path/to/chrome`.
These commands verify local archives and clients; they never publish. See
[release preparation](docs/release.md) for prerequisites and later delivery steps.

## [0.23.7] - 2026-11-15

Versioned evaluation checkpoint; not published to crates.io.

### ⚡ Improved

- Combine visit-budget checks and apply measured Wasm meter inlining, preserving
  admission behavior. The standalone consumer is smaller than the reference at
  both opt-z and opt-3; record the opt-z cost relative to the initial candidate.
- Append ordinary UTF-8 spans together in the bounded JSON/TypeScript writer,
  preserving escaping and exact output admission. The unchanged host probe shows
  lower export cost; single-root total-cost regressions remain unresolved.

### ✨ Added

- Add installed IC instruction qualification with a separately locked native
  simulator driver and optional guest counter. Record reproducible encoding
  regressions while preserving exact frame and independent size evidence.
- Add fixed-profile, fresh-build Wasm comparison with artifact execution, source
  hashes and total/code-section sizes. Retain the initial opt-3 regression and
  final passing results for the standalone consumer.
- Add an independent qualification consumer with a frozen sample-upload contract,
  candidate/reference supplier builds, generated TypeScript clients and runnable
  SDK-free Wasm artifacts. Qualify literal frames and typed refusals.
- Add a bounded paired host-schema timing probe for collection, finalization and
  JSON export. Record initial small-registry regressions without accepting a
  performance tradeoff or claiming replacement readiness.
- Qualify named collection with 1,024 shared scalar and recursive roots, including
  deterministic registration order, finite shared definitions and exact graph
  admission boundaries.

## [0.23.6] - 2026-11-15

Versioned evaluation checkpoint; not published to crates.io.

### ✨ Added

- Add 24 paired packed-i16/ordinary-list qualification cases through 16,384
  elements, preserving distinct wire representations and traversal allowances
  across Rust and generated TypeScript clients.
- Qualify generated byte-buffer clients through 1 MiB against Rust and both
  TypeScript runtimes, covering writer growth, length prefixes, byte limits,
  frame identity and owned-buffer custody.
- Add strict TypeScript qualification for the runtime and generated clients using
  a private pinned compiler. Check valid consumer calls and compile-time rejection
  of incorrect field types, decode results and request/response shapes.
- Add one candidate TypeScript interpreter for the admitted schema subset, exported
  as `typescript::RUNTIME`. Run generated payload/frame clients against candidate
  and published runtimes; compare primitive acceptance, typed failures and budgets.
- Add bounded named Rust schema collection for shared and recursive types.
  Retain reference-generated names, recheck encountered projections, and reject
  duplicate roots, name collisions, unstable projections and alias-only cycles.
  Generate the recursive framed-client fixtures from derives.
- Project explicit symmetric Serde field/variant renames into generated clients.
  Preserve Postcard bytes and reject duplicate projected names; qualify renamed
  client exchange and prototype-like field labels. Directional renames and other
  unsupported layout attributes remain explicit errors.

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
