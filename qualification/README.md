# Native compatibility qualification

This independent, unpublished workspace compares candidate `ic-wire 0.23.1`
with crates.io `ic-wire =0.22.4`. It contains real integration tests; product
packages do not depend on it. Run from the repository:

```sh
make test-compatibility
make test-allocation
make lint-compatibility
make fmt-check
make deps-list
```

The root Makefile checks this workspace's tooling declarations and lock checksum
against the same verified installed snapshot. Formatting and dependency commands
include both maintained workspaces. Each owns its lock, catalog and lint policy.
On a clean host prepare dependencies with
`cargo fetch --locked --manifest-path qualification/Cargo.toml`; checks are offline.

## Reference identity

Published archive SHA-256 (also locked by Cargo):
`12f9ea1376af8b628c513c838043106a79d0c94c2946fe4a79b8fefbc595c250`.
Its `.cargo_vcs_info.json` records packaging commit
`b310e3beb8deaf75b4bc6396925c428141806a59`, not the design's source pin
`2dc434b10bec2fca92345d940dbfedc05993e9d5`.

All 28 `src/` paths were compared in both directions with the pinned Git tree,
and their bytes compared with `git show`. The extracted source was also compared
directly with the checksum-identified archive. All matched. This establishes
library-source equivalence, not whole-repository or release-system equivalence.
The tests use no sibling working tree or mutable Git dependency. This source
check is a recorded adoption observation, not a Git-dependent test prerequisite.

## Current evidence and limits

Eight native tests compare:

- All 65,536 u16 and all 65,536 i16 encodings; selected wider-integer boundaries.
- All two-byte u16 and boolean inputs, plus empty/single-byte cases and options.
- A representative record against independent literal payload/request/response
  bytes, using separate candidate and reference field types.
- 1,748 budget settings in each of three encoding modes, for encoding and
  decoding: bytes 0–64, allocation 0–1536, nodes 0–128, depth 0–16. These are
  one-axis sweeps, not all combinations.
- Every single-byte mutation of the 28-byte request under three allowance
  configurations, plus every truncated prefix.
- Reusable/slice output agreement, empty reused output on refusal, untouched
  short slices and encoded lengths.
- Borrowed span custody and accepted noncanonical char behavior.

All passed on Linux x86_64 with Rust 1.100.0. This does not establish universal
compatibility, native macOS execution, Wasm runtime budget equality, opt-z/opt-3
performance, host-schema behavior or Rust/TypeScript interoperability.

## Allocation qualification

The dedicated `allocation` test executable has `harness = false` and runs eight
groups serially in one process. Published `stats_alloc =0.1.10` wraps the system
allocator; the workspace retains `unsafe_code = "forbid"`. This is a private
qualification dev-dependency and is absent from the product graph. It owns
instrumentation only, not product allocation or admission policy.

Regions surround only codec calls. Fixture construction, result assertions and
printing occur outside the measured region. Outputs remain live until after
statistics are read. `black_box` plus successful-allocation controls check that
optimized execution still performs the measured work. No harness worker thread
or concurrent test in this executable can contaminate process-wide counts.

| Controlled case | Observed allocations/reallocations |
| --- | --- |
| Scalar sizing and each failed preflight allowance | 0 / 0 |
| Owned u16 payload / frame | 1 / 0, requesting 2 / 13 bytes respectively |
| Serializer phase probe | No allocation before first pass; output allocation before second pass |
| Sufficient reused Vec, sufficient slice, short slice | 0 / 0; short slice untouched |
| Byte bound, odd/oversized/truncated packed span, refused first sequence seed | 0 / 0 |
| Owned Bytes / PackedI16 success | 1 / 0, requesting 3 / 4 bytes respectively |
| Borrowed bytes or empty owned bytes | 0 / 0 |
| Second-pass growth or serializer failure | One output allocation, no reallocation, output freed on failure |

Reference controls also observe one allocation for successful owned u16 output
and zero for failed preflight or an oversized bounded byte field.

Negative guarantees are tested explicitly: a valid byte prefix allocates before
a later invalid boolean, Text allocates its bytes before rejecting invalid UTF-8,
and a custom serializer allocates during sizing independently of codec output.
Thus these measurements establish selected boundary behavior, not a global heap
cap or complete-document validation before any allocation. They do not inject
allocator failure or qualify every type, target or custom Serde implementation.

All eight groups pass in native debug and optimized release builds. Reproduce:

```sh
make test-allocation
cargo test --manifest-path qualification/Cargo.toml --test allocation --release --locked --offline
```

`make test-compatibility` runs both comparison tests and allocation qualification.
Release-profile allocation checks are not a matched opt-z/opt-3 Wasm benchmark.

## Candidate Rust / reference TypeScript exchange

`make test-typescript` selects the private `typescript` qualification feature.
The harness imports candidate metadata into its typed graph, exports JSON from
that graph, and feeds it to published `ic-wire 0.22.4`'s `typescript::RUNTIME`.
That source is emitted only into a unique temporary test directory and removed
when the subprocess finishes. No sibling checkout or copied product runtime is
required. The published source comparison above covers this runtime source too.
The reference's matching `ic-wire-derive 0.22.4` is selected only by this optional
qualification feature; it is not a candidate product dependency.

Prerequisite: explicitly installed Node supporting `--experimental-transform-types`.
This check passed on Linux with Node 24.21.0 and Rust 1.100.0. It does not install
Node or TypeScript, invoke npm, or claim TypeScript compiler/browser/macOS proof.
`make lint-typescript` lints the focused Rust harness. Ordinary native qualification
continues to run without this optional feature or a Node prerequisite.

The four literal cases cover empty, zero, large signed and mixed representative
records: small integer, enum, option, bounded bytes, packed i16 and ordinary list.
Each is exchanged in both directions as a payload and both 11-byte frame directions.
Rust and TypeScript both encode independently to the frozen literals; Rust also
checks the values decoded from the TypeScript-produced bytes.
The test checks nonzero input offsets, decoded bulk ownership, retained output,
all truncated prefixes, trailing bytes and out-of-range values. Each record is
tested below, at and above its byte/node/depth boundary for payloads and both
frame directions. Request and response policies vary independently; the opposite
direction must remain admitted. The reference JS minimum node counts are 8, 9,
9 and 11 respectively, with depth 3 for all four records. These count value visits,
including enum payloads, optional Some values and ordinary list elements.

The reference decoder returns null-prototype records. Packed-length admission
reports `limit` at selected truncated prefixes; other tested truncations and
trailing bytes report `malformed`. Those categories are asserted explicitly.
Rust retains its own `Malformed`/`Trailing` contract. This harness does not claim
identical cross-language error enums or allocation/node accounting.
The fixture's Rust policy supplies bytes/nodes/depth to both runtime APIs with
nonbinding baseline node/depth allowances; boundary probes narrow one resource
at a time. Native allocation stays Rust-only. Boundaries beyond these records,
version-mismatch refusal, browser-specific behavior, full
schema coverage and a candidate versioned runtime pair remain unqualified.

The 0.23.1 checkpoint also exchanges a two-link recursive record and a pair of
fields sharing the same named record schema. Literal payloads are respectively
`01 01 80 01 00` and `07 00 07 00`. Both are decoded in Rust and the reference TS
runtime. TypeScript admits them at node/depth allowances of 6/4 and 7/3 and rejects
one below either allowance. Shared schema references do not alias decoded values;
mutating the left field leaves the right field unchanged. A cyclic JavaScript
input reaches the configured depth allowance and returns `WireError('limit')`.
These are selected recursive-value checks, not arbitrary graph or stack-safety
qualification. Native Rust retains independently sufficient budgets here.
