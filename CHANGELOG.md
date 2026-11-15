# Changelog

## [0.23.0] - Unreleased

### ⚠️ Breaking

- Evaluate the next minor version after published ic-wire 0.22.4 with root-level
  codec APIs and explicit Encoding selection. This incomplete replacement remains
  unpublished; consumer migration and release qualification are still pending.

### 🛠️ Tooling

- Initialize the virtual Rust workspace with compiler 1.100.0 and shared lint
  policy. Install Toko Tooling 0.44.3 and expose its dependency list/update and
  tool setup commands. The real core package and lockfile enable Cargo admission.

### ✨ Added

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
