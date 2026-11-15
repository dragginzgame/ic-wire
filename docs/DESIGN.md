# Architecture

Status: proposed architecture and evaluation contract, 15 November 2026. This replaces the supplied Contract and qualification boundaries and Replacement evaluation as the active design proposal. The migration and qualification plan below defines execution gates.

Reference: [ic-wire at 2dc434b10bec2fca92345d940dbfedc05993e9d5](https://github.com/dragginzgame/ic-wire-legacy/tree/2dc434b10bec2fca92345d940dbfedc05993e9d5), Core 0.22.4. All repository observations below refer to that commit.

## Current decision: hard cut

The user explicitly authorized a hard cut on 15 November 2026. Legacy ic-wire
API, schema, wire, acceptance and budget compatibility are not release requirements.
This supersedes preservation and migration-approval gates below wherever they
exist solely to maintain legacy compatibility. Keep one current implementation,
with no v1/v2 lanes, fallback readers or legacy-client support. Generated clients
and runtime ship together. Differential tests remain useful evidence, not an
obligation to reproduce every legacy behavior. Retained safety, admission, domain
validation and cross-language correctness contracts still need qualification.
The user reaffirmed that there are no consumer compatibility constraints before
publication. Remove legacy-only types and metadata forms rather than retain aliases.
This decision does not deploy consumers, reset stored state or publish packages.

## Release preparation

The user selected simplicity over speculative specialization and authorized
package preparation, initially excluding publication. The user subsequently
authorized renaming the public repository to `ic-wire` and publishing both
product crates as 0.24.0. After the expanded matrix,
the user requested fixing the important losses. A shared one-byte integer shortcut
now has exact-byte/admission proof and measured native/IC benefit; wider varints
remain delegated. The small Wasm-size and opt-z response tradeoffs are recorded in
[encoding](design/encoding.md). Single-root host costs remain disclosed. Zero-regression research criteria
below remain experiment results rather than absolute publication blockers.
Correctness, admission and package/browser verification remain required.

## 1 Decision and purpose

Evaluate a smaller implementation with one portable codec, one admission policy and one validated host schema graph. Define the current bytes, accepted input behavior, field representations, resource allowances and consumer type identity. Remove duplicate interpretation and orchestration rather than changing the wire protocol to make implementation easier.

Compare two candidates: targeted consolidation of the reference and a minimal fresh implementation of these same contracts. Select the simpler qualified outcome. A new repository, fewer crates or fewer source lines is not evidence that replacement is better.

Zero future technical debt cannot be guaranteed. The release target is no known unresolved correctness obligation, one authoritative representation per responsibility, and no unmeasured performance claim.

This design does not authorize resetting stored data, migrating consumer state, publishing new packages.

## 2 Source findings and evidence limits

The sibling checkout was unavailable in this workspace. Source was read from public GitHub URLs fixed to the reference commit, never from a moving main branch. No tests or benchmarks were executed during this review, and the supplied reference.sha256 manifest was not available for verification.

| Verified source fact | Consequence |
| - | - |
| Two public packages exist, ic-wire and ic-wire-derive; the core is no_std with alloc | Keep the portable boundary; do not add packages solely to mirror conceptual responsibilities |
| Core already shares prefixed encoding and frame decoding helpers | Do not describe those existing shared paths as new architectural progress |
| Encoding preflights using Postcard sizing, then writes with bounded Serde traversal | One-pass output allocation is not an equivalent substitute |
| Decoding delegates primitive acceptance to Postcard and no longer reserializes the result | Successful decoding does not establish canonical original bytes |
| Limits reserve the configured byte allowance, then charge weighted traversal and buffers | Allocation is an admission model, not measured live heap |
| Host metadata uses serde_json::Value; collection, validation, rendering, advice and the TS runtime each interpret schema structure | A typed validated graph is a concrete consolidation candidate |
| The named registry already supports sharing, recursion, generic shapes and order-independent names | Preserve those capabilities; replacing them with an inline tree would regress the design |
| The TypeScript runtime executes one schema interpreter with trusted metadata and custom checks | Do not introduce a generated execution function for every schema |
| Release roster validation already delegates to Shared tooling | Do not recreate the Platform-specific Make-regex problem in this design |

Sources: [core manifest](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/Cargo.toml), [core API](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/src/lib.rs), [codec orchestration](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/src/native/mod.rs), [schema registry](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/src/typescript/native_schema/registry/mod.rs), [schema rendering](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/src/typescript/metadata/mod.rs), [TS runtime](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/src/typescript/runtime.ts), [release roster](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/scripts/ci/wire-validation-roster.pl).

The workspace declares Rust 1.100.0. Its manifest permits Platform Types from 0.20.1 and Postcard from 1.1.1, while the committed lock resolves Platform Types 0.20.4, Postcard 1.1.3 and Serde 1.0.229. These resolved sources, not the lower manifest bounds, define the initial comparison cohort. [Manifest](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/Cargo.toml), [lockfile](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/Cargo.lock).

Historical reports may describe other complete runs. The user's stopped 29/39-cohort matrix remains partial evidence; neither it nor a different historical report qualifies the replacement.

## 3 Ownership and package boundaries

| Responsibility | Sole owner |
| - | - |
| Postcard primitive representation and acceptance | Selected Postcard supplier |
| Serde traversal admission and complete-input checks | Portable Wire codec |
| Frame header and method identity derivation | Portable Wire framing |
| Explicit bounded/byte/packed field semantics | Portable Wire field adapters |
| Consumer layout and collection normalization | Consumer Serde declarations |
| Metadata projection and schema graph validation | Optional host schema module |
| TypeScript module rendering and field advice | Consumers of the validated host graph |
| Browser byte operations | One current TS runtime |
| Principal and Account semantics | Published Platform Types |
| Execution, routing, authentication and retry | Platform and application owners |
| Build and simulator custody | Host, Build and Testkit in private qualification |

Start with the current two-package boundary as a hypothesis: core plus optional metadata derive. Host modules remain target-gated; codec-only Wasm must not select serde_json or macro machinery. A host package split requires dependency or release evidence and is not an initial task.

Preserve the current ability to forward the TypeScript feature to Wasm while excluding host runtime modules. Do not bundle an unrelated native-on-Wasm policy break into the codec evaluation. Verify normal, build and proc-macro closures separately.

Do not make the derive generate Rust serialization. Serde remains the sole Rust payload description. Wire metadata must either correctly project the supported Serde declaration or reject it; it may not invent a second layout.

No endpoint crate, transport wrapper, stable-memory subsystem, lifecycle manager, generic plugin registry or alternative binary format belongs here.

## 4 Current protocol contract

The frame is exactly 11 header bytes followed by the existing Postcard payload:

| Byte range | Meaning |
| - | - |
| 0 through 1 | ASCII IW |
| 2 through 9 | Method ID as little-endian u64 |
| 10 | Request 0 or response 1 |
| 11 onward | Payload, with no implicit extra envelope |

A payload API never adds or strips a frame. Method IDs use the existing explicit value or FNV-1a-64 over UTF-8 `ic-wire:method:` followed by the exact exported method name, with the reference constants and wrapping arithmetic. This is routing identity, not authentication or a schema hash. Duplicate numeric IDs must fail host registry admission. [Header](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/src/contract/mod.rs), [method ID](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/src/native/mod.rs).

Field and variant declaration order, Serde attributes and explicit adapter selection are protocol inputs. Sorting fields, switching vectors to byte buffers, or renaming an exported method is not a harmless cleanup.

Preserve Postcard's accepted noncanonical forms. The reference explicitly documents redundant varint `80 00` decoding as zero and char payload `02 61 62` decoding through upstream behavior. Do not add canonicality checks under the label of malformed-input hardening. Any intentional acceptance change needs an explicit current contract and focused tests, not legacy compatibility approval.

Decode/re-encode may change maps, sets, heaps or custom values. Signatures, hashes, stored requests and retry records use the retained original byte sequence, not a regenerated approximation.

Rust and TS must agree on exact emitted bytes for supported corresponding values and agree on the admitted wire-language cases defined by their cross-language contract. Their internal allocation/node accounting and error enums need not be identical; differences must be named and tested.

## 5 Portable API and output custody

Use one explicit encoding selection to remove the request/response/payload multiplication of orchestration wrappers. Proposed signatures, to compile in the first evaluation slice:

```rust
pub enum Encoding {
    Payload,
    Frame { method: u64, direction: Direction },
}
pub enum Direction { Request, Response }

pub fn encoded_len<T: Serialize + ?Sized>(
    value: &T, encoding: Encoding, limits: Limits,
) -> Result<usize, Error>;

pub fn encode<T: Serialize + ?Sized>(
    value: &T, encoding: Encoding, limits: Limits,
) -> Result<Vec<u8>, Error>;

pub fn encode_into<'out, T: Serialize + ?Sized>(
    value: &T, encoding: Encoding, limits: Limits,
    output: &'out mut Vec<u8>,
) -> Result<&'out [u8], Error>;

pub fn encode_to_slice<'out, T: Serialize + ?Sized>(
    value: &T, encoding: Encoding, limits: Limits,
    output: &'out mut [u8],
) -> Result<&'out [u8], Error>;

pub fn decode<'de, T: Deserialize<'de>>(
    input: &'de [u8], encoding: Encoding, limits: Limits,
) -> Result<T, Error>;

pub fn validate_request_header(
    method: u64, header: &[u8; 11],
) -> Result<(), Error>;
```

Retain the four explicit Limits fields and the typed Error categories Limit, Allocation, Malformed, Trailing and Identity. Rust path changes are permitted at the consumer cut; bytes and failure semantics are not silently changed.

Output contracts:

- Owned encoding returns only a complete buffer. Preflight finishes before codec-owned output allocation.

- Reusable Vec output is empty after an error. Capacity retention on failure is not promised. A retained retry frame must live elsewhere.

- Slice output refuses an undersized destination before any write. Other failures may leave modified storage; only the successful returned prefix may be published.

- Decoding consumes exactly one complete payload/frame and rejects a remaining suffix. Borrowed values may reference the original input.

- Header validation proves only fixed-header identity/direction; it does not authenticate the sender or validate payload limits/schema.

All encode destinations use one private sizing/admission orchestration and destination-specific writing. This is a consolidation of responsibilities, not permission to add a broad public Sink trait or dynamic dispatch framework.

Retain a synchronous scratch-or-owned helper only if a named consumer needs the reference's drop-before-callback contract. Implement it as orchestration over the same engine. Its callback borrow cannot escape into a future; it must not run on an encoding error.

Do not introduce a reusable PreparedEncoding token: arbitrary trusted Serialize code can observe interior mutation between passes. A cached length is not an immutable byte commitment. Keep the reference's deterministic custom-serializer obligation and bounded second-pass writes.

## 6 Admission model

One private meter owns nodes, depth, conservative allocation charges and sticky refusal. Sizing and writing each receive their appropriate fresh meter; decoding receives its own. They share policy definitions without pretending their Serde visit sequences are identical.

Preserve the reference accounting initially:

- Reserve the configured byte allowance against allocation, not the actual resulting length.

- Charge visited value sizes and buffer lengths using the reference weighting by eight.

- Count unit elements and zero-width values; small encoded size does not mean small traversal.

- Preserve actual encode/decode depth increments and wrapper visits.

- Frame accounting reserves header space through the same checked transformation as the reference.

- Any overflow or exhausted allowance fails closed and cannot be refunded by later successful operations.

These costs can depend on Rust type layout and target width. Qualify native and Wasm boundaries separately. Do not claim equal numeric thresholds across architectures or browser execution.

Create an operation-by-operation accounting table from the pinned encode/decode implementations before replacing their wrappers. A more elegant “one node per logical field” policy is a behavior change, not an internal simplification.

On decode, suppress attacker-controlled sequence/map size hints before a standard visitor can reserve from them. Admit visited values and buffer spans before the corresponding consumer allocation boundary that Wire controls. On encode, size/admit before reserving output.

Do not claim that a complete document is validated before every allocation. Incremental decoding may allocate a valid prefix before discovering malformed later data. Custom Serde can allocate or execute arbitrary work outside Wire visits. Traversal allowances neither sandbox that code nor guarantee fallible handling of every allocator failure. [Decode admission](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/src/native/decode/mod.rs), [allocation tests](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/tests/native/allocation/mod.rs).

Keep Postcard as the primitive engine initially. The current encoder includes specialized integer emission. Compare direct supplier delegation with each specialization; retain a specialization only with exact-byte/acceptance proof and a material measured benefit. Do not start by writing another full serializer or primitive parser.

## 7 Field representations and borrowing

One bulk word implementation may serve compatible packed adapters. Representation policy remains explicit at each consumer field.

| Representation | Required distinction |
| - | - |
| Vec\<u8> versus ByteBuf | Bytes may match, but traversal admission differs |
| [u8; N] versus ByteArray\<N> | ByteArray adds the byte-buffer length prefix |
| Ordinary integer vector versus Packed | Varint elements/count differ from fixed-width little-endian byte span |
| Packed opaque IDs | Prefix counts bytes, not ID elements; refuse partial IDs |
| Bool adapter | Byte values are exactly 0/1; do not implement Packed\<bool> through arbitrary-bit casting |
| Finite float adapter | Reject nonfinite values; preserve signed zero and subnormal bits |
| Nonzero integer adapter | Validate every word; no zero may become a nonzero typed value |
| Heap adapter | Rebuilding a heap does not preserve original serialization order |
| BytesRef and TextRef | Require borrowed input; cannot silently allocate an owned substitute |

Bounds distinguish element count from byte count, with checked multiplication. Validate span and width before copying. Bool/finite validation must occur before typed decoded-output allocation as required by their existing contract. Nonzero currently validates after bounded word decoding; any prevalidation improvement must preserve error/admission behavior and be measured rather than claimed as an existing universal guarantee.

Avoid a generic adapter policy language. Use a sealed set of admitted word representations, a shared endian/length implementation and small domain validators. No unsafe reinterpretation of unaligned input or bit patterns invalid for the destination type. Rust remains unsafe-code-forbidden.

Borrowed byte/text views point into the selected original input span and cannot outlive it. Packed owned decode must not accidentally become a borrowed typed cast. TypeScript byte/typed-array ownership must retain the reference's copy and selected-buffer semantics, including nonzero byteOffset and unaligned input.

Ordinary small-value vector controls remain in evaluation. Packed representations can be larger or slower for those values. Advice may recommend a review; it must never rewrite fields or regenerate a different wire contract automatically. [Representation contracts](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/docs/guides/runtime.md), [Packed implementation](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/src/packed/mod.rs).

## 8 One validated host schema graph

Replace repeated interpretation of loosely typed JSON inside host tooling with a private typed graph. JSON remains an import/export boundary when required by existing consumers; it is not the internal authority.

Proposed conceptual API:

```rust
pub struct SchemaBuilder { /* unfinished, bounded collection */ }
pub struct ValidatedRegistry { /* private immutable graph */ }

impl SchemaBuilder {
    pub fn register<T: Wire + ?Sized>(
        &mut self, exported_name: &str,
    ) -> Result<Root, GenerationError>;
    pub fn finish(self) -> Result<ValidatedRegistry, GenerationError>;
}

impl ValidatedRegistry {
    pub fn generate_typescript(&self, methods: &[Method])
        -> Result<String, GenerationError>;
    pub fn advise_fields(&self, roots: &[Root])
        -> Result<Advice, GenerationError>;
}
```

Root handles are scoped to the registry that created them; foreign handles fail admission. Numeric NodeIds are private indices, never wire identities.

Collection interns concrete definitions and reference edges. Finalization validates supported shapes, bounds, references, names, methods and recursion, and then seals the graph. No public mutable node access. Failed collection cannot be reused as an apparently valid registry.

The derive emits metadata only. Unsupported or ambiguous Serde attributes fail explicitly. Manual schema implementations remain trusted projections whose correspondence to Serialize/Deserialize requires independent vectors; a graph validator cannot prove arbitrary Rust code matches its claimed layout.

The reference already handles recursive/shared shapes. Preserve that ability with explicit worklists and bounded traversal, not recursive JSON expansion. Reject unresolved references, invalid aliases and unproductive reference-only cycles. Permit supported recursive value shapes subject to runtime value-depth/node limits.

Memoize context-independent validation by node. Do not cache path-sensitive depth, rooted advisory paths or occurrence budgets as though all visits were equivalent. Bound graph nodes, edges, traversal work, path depth, emitted source size, diagnostic count and diagnostic bytes. Report a bounded truncation summary instead of enumerating exponentially many root paths.

Do not collapse one use site's bounds into another's shared type definition. Separate reusable type structure from field/method-specific constraints.

Retain source-name stability within the supported build contract: root order and unrelated registrations cannot rename existing output. The reference derives generated names from compiler type spelling, which is not a stable protocol ID or guaranteed unique Rust type identity. Preserve its algorithm during compatible generation, retain collision/conflicting-projection checks, and make public root aliases the durable consumer-facing names. A new naming scheme requires a declared source migration.

Do not substitute TypeId without resolving borrowed/lifetime-parametric types. Do not silently hash a new graph structure and call it the old schema or method identity.

The first experiment must show that the typed graph reduces authoritative parsers and repeated traversal. If it only adds conversion layers beside the existing JSON walkers, reject the experiment.

## 9 TypeScript execution and generation

Keep one audited runtime interpreter plus generated types and immutable metadata. Do not generate per-record writers/readers in addition to the interpreter.

Host generation consumes only ValidatedRegistry. Generated artifacts identify their metadata/runtime format version and compatible runtime. A mismatched runtime must fail explicitly rather than guess a fallback interpretation. This version labels generated tooling artifacts; it does not add bytes to Wire frames.

Validate externally supplied metadata at the host boundary. The browser factory consumes generated trusted metadata, as the reference does. Do not broaden it into an untrusted-schema service without a named consumer. Keep emitted runtime data privately owned or immutable so mutation after validation cannot change traversal meaning.

Preserve numeric/BigInt boundaries, option/result/enum representations, UTF-8 behavior, primitive acceptance, declaration order, little-endian packed output, strict domain validators and trailing-input rejection. Test invalid numbers and large lengths before output allocation where the corresponding runtime boundary controls it.

Rust allocation charges cannot be copied into JavaScript and relabeled as heap bounds. The reference projects bytes/nodes/depth with different traversal semantics. Keep that distinction explicit and test cross-language byte behavior under separately sufficient budgets. [Limit projection](https://github.com/dragginzgame/ic-wire-legacy/blob/2dc434b10bec2fca92345d940dbfedc05993e9d5/crates/ic-wire/src/typescript/limits/mod.rs).

Retain the reference's TS error contract or explicitly migrate it. For example, Rust Trailing and TS malformed currently need an intentional mapping; byte interoperability does not require inventing equal error enum names.

## 10 Platform Types and consumer contracts

Platform Types owns Principal/Account validation, text, bytes, Serde and absent-versus-zero subaccount behavior. Wire's optional host integration owns only their schema projection.

The initial lock selects published Types 0.20.4. Verify package/source identity and frozen representations against the published supplier, not a renamed local copy or only the old Wire adapter. Check Principal boundaries, malformed encodings, Account None versus all-zero Some, and any supplier normalization separately.

Do not pull Platform Runtime, Protocol, crypto or native Agent into Wire to accommodate the parallel Platform redesign. A future Types cohort change is a separate dependency and consumer compatibility decision.

Before cutover, inventory all retained requests, response archives, hashes/signatures, browser clients and persisted Wire values. The 11-byte header has no new schema-version discriminator. Reusing the same method ID with an incompatible layout cannot safely distinguish old data.

Adoption is a hard cut with regenerated clients. Legacy method/version separation and old-client support are not required. Actual storage operations remain downstream actions. No fallback reader, implicit normalization, automatic reset or migration lives in Wire.

## 11 Deletions and reuse policy

Delete only after replacement proof:

- Redundant request/response/payload orchestration wrappers superseded by the Encoding API.

- Parallel JSON interpretation paths superseded by the sealed host graph.

- Duplicate packed/endian primitives where one implementation preserves the exact field contracts.

- Old code-generation paths and diagnostics walkers that no longer own an invariant.

- Primitive fast paths that fail the measured benefit test.

- Compatibility aliases after the coordinated Rust API cut.

- Obsolete benchmark machinery and active design history whose obligations are mapped elsewhere.

Do not delete ordinary vectors, borrowed APIs, bound checks, collision checks, independent golden cases or security/admission tests to meet a size target.

Reuse starts after the destination contract and independent tests exist. Each admitted unit records source commit, license, dependencies, invariant, removed alternatives and measurements where relevant. Whole-crate copying is not an initial milestone.

The design succeeds only if actual consumers use the new boundaries, fewer independent implementations interpret the same contract, and qualification shows no unapproved compatibility or performance loss. If targeted consolidation achieves that with less maintenance risk, choose it.

# Migration and qualification

Status: proposed evaluation and delivery sequence, 15 November 2026. Implements the architecture above. No code replacement, publication, persisted-state migration or benchmark resumption is authorized by this document alone.

## 1 Freeze the reference and the requirement ledger

Use commit `2dc434b10bec2fca92345d940dbfedc05993e9d5` throughout. In the implementation workspace, read committed bytes with:

```sh
git -C ../ic-wire show 2dc434b10bec2fca92345d940dbfedc05993e9d5:<path>
```

Do not copy the moving sibling working tree into the candidate. Verify reference.sha256 against committed bytes before relying on it; include the reference commit and hash-tool version with the result. If a file is unavailable, report the gap rather than substituting another revision.

Record compiler, Cargo.lock, selected package/source IDs, features, target and optimization settings. Initial comparison uses the reference's resolved supplier cohort, including Platform Types 0.20.4 and Postcard 1.1.3. Do not combine architectural evaluation with supplier upgrades.

Build a requirement ledger with:

| Field | Required content |
| - | - |
| Consumer | Actual repository/revision and call site or published contract |
| Representation | Serde type/order/attributes, byte/packed/domain selection |
| Identity | Explicit method ID or exact name-derived ID |
| Resources | Each caller allowance and each field bound |
| Custody | Borrowed, owned, retained retry, signed/hashed or persisted bytes |
| Browser | Generated exports, runtime version and supported retained clients |
| Proof | Independent bytes, refusal case, graph/installed test |
| Disposition | Preserved, intentionally retired, or downstream-approved migration |

Do not treat absent search results as permission to remove a public feature. Resolve indirect users, persisted values and client retention.

## 2 Establish byte and admission oracles

Create a small qualification workspace that depends on the pinned reference and the candidate as distinct packages for native comparison where feasible. Use separate Wasm artifacts for installed measurements. The harness must not become a production dependency.

Build independent literal vectors before candidate code. Freeze expected payload and full request/response bytes, lengths and method IDs.

Freeze an accounting table from the actual reference visitor behavior. Include encode and decode separately, frame/header transformations, root visits, compound visits, unit elements, buffers, wrapper sizes and target-dependent Rust layouts. Test exactly-at and just-below/above each relevant limit.

Preserve refusal precedence in intersecting failure cases: excessive frame size versus bad header, short header versus identity, budget exhaustion versus malformed payload, and trailing data after a valid value. Do not merely assert that all failures return some error.

The old suite supplies cases, not a passing result for the candidate. Expected byte fixtures must not be generated by the implementation being tested.

## 3 Build the first vertical slice

Use one representative record containing:

- A small integer and enum/option.

- A bounded byte field.

- An explicitly selected packed integer or opaque-ID field.

- A bounded ordinary vector control.

Cover empty, one-element, small-varint and representative large values. Add a borrowed byte/text projection and a caller-owned output buffer. Use the same consumer field choices for both candidates.

Implement only the required portable encode/decode/frame path, plus the minimum host metadata and TypeScript generation needed to exchange that record. No endpoint, router, transport or stable-memory work.

Compare targeted consolidation and a fresh implementation. Reuse Postcard as a supplier; do not copy the reference crate into the new workspace. An isolated primitive may be reused only after its contract and independent tests exist.

Exit gate:

- Independent expected bytes match.

- Rust encodes for TS and TS encodes for Rust.

- Owned, reused and slice destinations agree.

- All four allowances and field bounds have boundary cases.

- Ordinary vectors remain unchanged.

- No host dependencies appear in the portable graph.

Passing this slice permits continued evaluation, not selection of a whole replacement.

## 4 Qualify custody and hostile inputs

| Area | Required cases |
| - | - |
| Output allocation | Failed preflight performs no codec-owned output allocation; short slice refuses before writing |
| Reusable storage | Error leaves Vec empty; previous retained retry frame remains separate |
| Slice storage | Other failures do not expose a successful prefix; untouched storage is not falsely promised |
| Borrowing | Pointer/offset checks prove views refer to selected input; compile-fail cases reject escaping/mutating a live borrow |
| Owned results | Remain valid after input/scratch release and do not alias mutable caller storage unexpectedly |
| Counts | Huge length prefix, partial span, overflowed width multiplication, unit-element collections |
| Primitives | Invalid bool/UTF-8/tag, truncated varint, overflow, accepted redundant varint and reference char behavior |
| Domain fields | Partial words/IDs, nonfinite values, zero nonzero words, invalid bool bytes, signed zero and subnormals |
| Endianness | Exact little-endian bytes and unaligned/subview inputs; exercise the alternate-endian conversion logic |
| Normalization | Map/set/heap decode-reencode may differ; original signed/hash/retry bytes remain exact |
| Trusted Serde | Documented limits of custom allocation/work; inconsistent passes cannot write beyond admitted storage |

Use allocator instrumentation narrowly to prove controlled allocation ordering, not to claim a process-wide memory cap. Keep Rust native and Wasm layout-dependent accounting tests separate.

For TS, include nonzero byteOffset, bounded selected Buffer/Uint8Array spans, unsafe numeric values, BigInt boundaries and mutation/custody checks.

## 5 Prove the host graph before choosing replacement

Large registries are an architecture gate, not optional polish after a successful codec demo.

Exercise shared subgraphs, supported recursion, alias cycles, concrete generics, borrowed type projections, duplicate roots, generated-name collisions, conflicting manual projections and unresolved references.

Reordering roots or adding unrelated types must not rename existing generated outputs. Preserve the admitted naming algorithm until an explicit source migration is chosen. An object graph with a shared type used under different field bounds must retain both use-site constraints.

Measure graph construction, final validation, TS rendering and advisory traversal separately. Include adversarial diamond-shaped graphs and many rooted paths. Diagnostics must have bounded count, path length and output bytes; truncation must be explicit and deterministic.

Verify that memoization does not bypass depth limits or reuse a result under incompatible constraints. No declaration registry may be reported as valid after a failed registration.

Exit gate: the candidate has one validated internal authority, fewer schema interpretation paths, and no worse unapproved large-registry behavior. If a typed graph merely adds a conversion layer, stop that approach.

## 6 Verify suppliers and build boundaries

Build standalone codec-only consumers with derive absent. Build supported native host consumers with metadata/derive and optional Platform type schemas. Inspect normal, build and macro dependency graphs, not only source cfg attributes.

Test the current feature-forwarding behavior on Wasm and confirm that host runtime modules and serde_json remain absent. Verify no Platform Runtime, Testkit, Host, Build, Candid or simulator dependency reaches the product closure.

Against the selected published Platform Types supplier, verify:

- Exact Principal valid/invalid boundaries and bytes.

- Account absent versus explicit zero subaccount behavior.

- Serde representation, including borrowed/owned crossings where supported.

- Canonical package/source identity.

- Wire metadata correspondence without redefining supplier types.

Archive/package consumer tests must pass without hidden path patches. A local build is not registry distribution proof.

## 7 Measure a matched comparison

Use opt-z as the primary decision profile and opt-3 as the regression profile, matching the reference's LTO/codegen/panic/strip choices. Build fresh isolated target directories for every candidate/profile comparison and record source/dependency hashes.

Measure total uncompressed Wasm, code and other sections, IC instruction counts, encoded length, controlled allocation behavior, native throughput and host-generation cost. Keep application bodies and fields equivalent; disclose any required API adaptation.

Include ordinary small-value controls as well as bulk workloads. Do not average away regressions in tiny requests with gains on large blobs. Distinguish setup/conversion cost from codec work, and count consumer glue in maintenance cost.

Before measurements, freeze the selected workloads and numerical acceptance thresholds. Default byte/acceptance tolerance is zero. No size/instruction regression is accepted merely because source is shorter. Any allowed performance tradeoff needs an explicit, recorded consumer rationale; do not move thresholds after seeing results without recording the change.

Use paired repeated measurements where timing is noisy. Record exact installed environment versions for instruction evidence.

The stopped 29/39-cohort matrix is not resumed automatically. Select a new bounded experiment with a declared purpose. Historical reports, including other completed matrices, are context and test ideas only.

## 8 Choose the outcome

| Result | Decision |
| - | - |
| Consolidation meets contracts with less machinery than replacement | Adopt consolidation |
| Replacement demonstrably reduces independent invariants/paths and passes all required gates | Proceed to consumer migration |
| Codec slice passes but registry/advice or supplier boundaries remain unresolved | Continue limited evaluation; no replacement decision |
| Performance worsens outside agreed bounds or complexity simply moves | Rework or stop |
| Current Rust/browser contracts disagree | Fix the contract or implementation before cutover |

The decision record names the authoritative survivor for each consolidated responsibility, removed paths, measured results, remaining limitations and implementation/review cost.

A new repository is not a reason to prefer replacement. Old code is not a reason to exempt an abstraction from review.

## 9 Consumer migration and deletion

Use a breaking release and regenerate TypeScript with the current runtime. Qualify current browser clients; retaining legacy clients is not required.

The hard-cut decision permits incompatible field representations and layouts. Client rollout and persisted-state operations still belong to the consumer. Wire supplies no generic fallback reader or migration engine.

Record how exact original request bytes survive retries, signing and rollback. Changing a package version does not make old retained values safe to reinterpret.

After all relevant gates pass, delete superseded API wrappers, JSON walkers, unneeded specializations, compatibility aliases and retired fixture scaffolding. Every removed test gets an invariant disposition; independent failures remain covered.

Do not copy the entire historical benchmark tree or active design history into the new repository. Preserve immutable references and current proof inputs.

## 10 Release acceptance

Release requires:

- Identified consumers and retained-byte/persisted-state contracts.

- Exact required bytes, method identities, accepted language and failure categories.

- Qualified allocation ordering, borrowing and all four allowances.

- Explicit adapter behavior and unchanged ordinary-vector controls.

- Large-registry, generated-name and bounded-diagnostic proof.

- Rust/TS interoperability and a compatible generated-runtime pair.

- Published supplier identity and standalone native/Wasm/package proof.

- Matched opt-z and opt-3 results within the recorded acceptance decision.

- No temporary compatibility layer or correctness-critical unresolved obligation.

Wire already delegates roster admission and execution machinery to Shared. Preserve its source/lock/receipt/host/artifact obligations rather than importing another repository's release framework. Map the existing release stages to their replacement evidence; changing implementation does not authorize dropping stages silently.

The deliverable is a qualified reduction in independent mechanisms. Neither a copied crate nor a green subset of inherited tests completes this rewrite.
