# Native compatibility qualification

For release preparation, `make check-packages` verifies extracted archives using
this workspace's standalone consumer. `make check-browser` executes the generated
clients in an explicitly selected Chromium binary. Neither command publishes;
see [preparation checks and delivery boundaries](../docs/release.md).

The [standalone consumer](consumer/README.md) owns a small application contract,
independent of sibling recodes. `make test-consumer` checks its literal frames
against each Rust supplier; `make check-consumer-wasm` builds and executes both
standalone Wasm artifacts. Its generated clients participate in the TypeScript
exchange harness. These are integration checks, not size/instruction measurements.
The separate `make measure-consumer-wasm` target supplies a matched uncompressed
size comparison; [both profiles pass for this consumer](../docs/design/consumer-measurement.md).
It does not measure IC instruction counts.
`make measure-consumer-ic` supplies the separate installed counter experiment,
using an explicitly prepared PocketIC server and [isolated native driver](ic-instructions/README.md).
The first run identifies request-encoding regressions at both profiles and a
small opt-3 response-encoding regression; its performance criterion remains unmet.

`make bench-codec` measures 13 native workload families with ordinary/bulk controls,
public codec phases and paired suppliers. `make bench-codec-allocations` measures
allocation counts in a separate build; `make profile-codec` prepares a selected
operation for perf call-stack sampling where host permissions allow it. See
[the codec investigation guide](../docs/design/codec-benchmarks.md).

`make bench-schema` runs the separate bounded host timing probe. Its frozen
workloads, phase boundaries and first results are recorded in
[schema measurement](../docs/design/schema-measurement.md). The first run found
small-registry regressions; it does not establish a candidate performance win.

This independent, unpublished workspace compares candidate `ic-wire 0.23.9`
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

## Candidate Rust / candidate and reference TypeScript exchange

`make test-typescript` selects the private `typescript` qualification feature.
The harness imports candidate metadata into its typed graph, exports JSON from
that graph, and feeds it to both candidate and published `ic-wire 0.22.4`'s `typescript::RUNTIME`. Generated clients run against each source and their returned results must agree.
That source is emitted only into a unique temporary test directory and removed
when the subprocess finishes. No sibling checkout is required. The published source comparison above covers this runtime source too.
The reference's matching `ic-wire-derive 0.22.4` is selected only by this optional
qualification feature; it is not a candidate product dependency.

Prerequisite: explicitly installed Node supporting `--experimental-transform-types`.
This check passed on Linux with Node 24.21.0 and Rust 1.100.0. It does not install
Node or TypeScript, invoke npm, or claim browser/macOS proof.
`make lint-typescript` lints the focused Rust harness. Ordinary native qualification
continues to run without this optional feature or a Node prerequisite.

For static checking, install the private pinned compiler with
`npm ci --prefix qualification/typescript --ignore-scripts --no-audit --no-fund`,
then run `make check-typescript`. The lock selects TypeScript 5.9.3 with registry
integrity verification; it is a qualification-only dev dependency. This target
runs the same exchange tests and additionally checks the candidate runtime and
generated payload/frame clients with `strict`, `noUncheckedIndexedAccess`, and
`exactOptionalPropertyTypes`, targeting ES2022 ES modules. Positive and expected-error
consumer fixtures check Number/BigInt, options, enum payloads, packed arrays,
decode result types and distinct request/response shapes. The check does not
download tools automatically. `make test-typescript` still requires only Node.

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
browser-specific behavior and full schema coverage remain unqualified.

The 0.23.1 checkpoint also exchanges a two-link recursive record and a pair of
fields sharing the same named record schema. Literal payloads are respectively
`01 01 80 01 00` and `07 00 07 00`. Both are decoded in Rust and the reference TS
runtime. TypeScript admits them at node/depth allowances of 6/4 and 7/3 and rejects
one below either allowance. Shared schema references do not alias decoded values;
mutating the left field leaves the right field unchanged. A cyclic JavaScript
input reaches the configured depth allowance and returns `WireError('limit')`.
These are selected recursive-value checks, not arbitrary graph or stack-safety
qualification. Native Rust retains independently sufficient budgets here.

The 0.23.2 checkpoint adds 11 literal scalar/text vectors through the same
published runtime harness: u64 zero/128/maximum, i128 zero/-1/minimum/maximum,
empty text, a multibyte character, a retained BOM and a supplementary character.
Rust decodes the TypeScript-produced bytes as well as checking its own encoding.
Both runtimes reject selected invalid UTF-8, including overlong encodings,
surrogates and values beyond Unicode's range. TypeScript enforces UTF-8 byte
bounds, BigInt inputs and integer range limits. Redundant `80 00` remains accepted
as zero for u64/i128; re-encoding emits `00` while retained input stays unchanged.
These selected vectors do not qualify every scalar width, JavaScript surrogate
normalization, or all malformed/noncanonical encodings.

## Generated payload client

From 0.23.3 the representative payload exchange imports the actual TypeScript
module emitted by `ValidatedRegistry::payload_module`; the harness no longer
constructs that baseline payload codec by hand. The generated module exports one
frozen codec object, keeps its parsed metadata private and checks `schemaFormat`
before invoking the runtime factory. The test observes the actual supplied policy
and rejects a deliberately incompatible runtime before its factory can execute.

A test-only adapter exports `schemaFormat = "ic-wire-json"` alongside the
published 0.22.4 payload factory. This explicitly qualifies that interpreter for
the tested metadata subset; the marker alone does not authenticate arbitrary
runtime implementations. The published Cargo source pin supplies runtime identity.
No adapter or reference dependency enters the product graph. Scalar and
boundary-sweep controls still exercise the supplier directly. Node transforms emitted TypeScript;
static checking is supplied separately by `make check-typescript`; neither target
constitutes browser execution evidence.

From 0.23.4, baseline request/response exchange also imports generated method
codecs. The adapter records both directions' actual factory policy. A second
generated method uses u64::MAX, a recursive request and a distinct shared-schema
response, with exact payload byte/node/depth allowances 5/6/4 and 4/7/3. Header
addition yields 16 and 15 byte frames. Independent literal vectors and Rust decode
prove the full eight-byte identity, direction and layouts. Wrong method/direction
and mismatched runtime modules reject. Duplicate export names/IDs, missing roots,
header-inclusive JS integer overflow and total output limits have host tests.

Version 0.23.5 switches the representative payload module's schema source to
`#[derive(candidate::typescript::Wire)]` on its Rust Record and Choice types.
The renamed dependency is explicit with `#[wire(crate = "candidate")]`.
The representative record is now generic; `Record<u8, 3>` supplies the concrete
integer type and byte-field bound to generation. The derived metadata must exactly
match the independently authored JSON fixture
before generation; the generated client then runs the same byte/custody tests.
Version 0.23.6 framed controls use SchemaRegistry to collect their generic,
recursive and shared Rust types. The qualification
feature now enables the candidate host derive, while the reference's macro stays
its separately selected published supplier.

Version 0.23.6 adds a generated-client exchange for explicitly renamed fields
and variants. Independent JSON and literal bytes prove the selected names and
unchanged Postcard order/tags; JavaScript must use the projected names. A field
named `__proto__` survives JSON parsing and decodes as an own property on a
null-prototype record. This is an opt-in fixture contract, not a rename or
migration of any production consumer.

Named recursive collection is compared directly with the published reference's
SchemaRegistry using the same concrete Chain/Shared Rust types and independent
reference trait implementations. The complete JSON maps, including generated
names and wrapper references, match. Root-order stability, duplicate and generated
name collisions, identical diagnostic labels for distinct Rust types, inconsistent
manual projections, alias-only cycles and sticky failure have host tests. Named
borrowed projection also compiles without a static lifetime bound. These are
focused contract checks, not large-registry performance qualification.

## Large-byte generated-client qualification

The 0.23.6 `bulk_cases` fixture adapts the workload from
`crates/ic-wire/examples/typescript_conformance/large_bytes/{mod.rs,checks.mjs}`
at the pinned reference commit above. The source SHA-256 values are respectively
`6033c1eb16dcb25eb3fba0b0bddfa7fffb8850a26e90b4e2a8035a3a7151ae07` and
`a9e511850e6325164d948d31e1ca5283618dc514df9e31a4134d0cb351a063e7`.
The retained repository license applies. Reuse is limited to the 1 MiB byte
workload, its `i % 251` pattern and boundary-check ideas; no legacy generator,
benchmark driver, executor or release machinery is imported.

The destination is the existing generated-client exchange harness. Independent
literal length prefixes cover 0, 63, 64, 127, 128, 16,383, 16,384 and 1,048,576
bytes. Candidate and reference TypeScript encoders match those prefixes and the
byte pattern in payload/request/response form; Rust independently encodes and
decodes the returned payloads. Checks cover writer growth, exact/short payload
byte allowances, field overflow, selected spans, owned decoded buffers, retained
output, malformed missing bodies, trailing bytes and frame identity/direction.
The same generated modules also pass strict TypeScript checking.

Reproduce this focused case with:

```sh
IC_WIRE_CHECK_TYPESCRIPT=1 cargo test --manifest-path qualification/Cargo.toml --features typescript --test typescript bulk_cases --locked --offline
```

This is correctness evidence at selected sizes, not throughput, JS heap-allocation,
browser, Wasm-size or IC-instruction evidence.

## Paired packed and ordinary signed buffers

`bulk_cases::packed` adapts the four value distributions from
`qualification/ic-wire-benchmarks/benches/fixtures/packed_i16/mod.rs` at the same
pinned reference commit. Its SHA-256 is
`e2f873f88d0e190652ae43e51ff88507689f42843b85aca0d6bb3202fd102f22`.
The retained repository license applies. Only workload sizes/value patterns are
reused; historical measurements, Candid controls and benchmark orchestration are
not imported.

The 24 paired cases combine 0, 1, 64, 128, 1,024 and 16,384 elements with small
positive, maximum, small negative and mixed boundary values. Independent literal
prefixes and word/varint tables check little-endian packed bytes versus ordinary
Postcard ZigZag varints. Generated request/response clients run against both TS
runtimes; returned payloads also match Rust encoding and decode to the original
values. Exact/short byte and node allowances, selected unaligned input spans,
owned decoded results, odd packed lengths, field overflow and redundant empty
lengths are checked. Generated modules pass strict TypeScript checking.

Browser policy counts packed data as one value and ordinary lists as one plus
their element count. Rust uses a separately sufficient weighted Serde allowance;
these tests do not imply equal cross-language budget accounting. Native endian
conversion on a big-endian host and matched performance measurements remain
outstanding. Run this subset by replacing `bulk_cases` in the command above with
`bulk_cases::packed`.
