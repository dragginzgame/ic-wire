# Architecture and contracts

This document describes the current 0.24 implementation. It replaces the initial
replacement proposal and its superseded execution sequence. Detailed contracts,
source provenance and checkpoint evidence remain in the linked topic documents.
Historical measurements qualify only their recorded inputs.

## Scope and ownership

Wire owns bounded Serde/Postcard encoding, framing, explicit field adapters and
optional native schema tooling. It has two product packages in the root virtual
workspace: `ic-wire` and `ic-wire-derive`. Private contract tests and consumer packages
live directly under `qualification/` and share one workspace and lockfile, including
the private instruction driver. Product packages do not depend on qualification.

| Responsibility | Owner |
| --- | --- |
| Primitive representation and acceptance | Published Postcard, with the qualified one-byte integer shortcut |
| Traversal admission and complete-input checks | Portable Wire codec |
| Header and routing identity | Portable Wire framing |
| Byte, bounded and packed field semantics | Explicit Wire/Serde field adapters |
| Layout, custom Serde work and collection normalization | Consumer |
| Metadata projection and validation | Optional host schema module and strict derive |
| Declarations, client generation and bulk advice | Projections of the validated graph |
| Browser byte operations | One bundled TypeScript interpreter |
| Principal/Account identity and Serde | Published Platform Types |
| Execution, routing, authentication, retries and deployment | Downstream application/platform owners |
| Native simulator lifetime | Published Testkit managed-server in private qualification |

The codec is `no_std` with `alloc`, forbids unsafe Rust and remains SDK-independent.
Host modules and optional schema/derive/Platform dependencies are excluded on
Wasm even when their features are forwarded. There is no endpoint or executor.
Keep one production Serde/Postcard path; additional abstractions, specializations
or package splits need a concrete caller or measured justification.

## Current contract

The current API, metadata and runtime form one contract. Legacy API, wire, schema,
acceptance and budget compatibility are not requirements. There are no versioned
compatibility lanes, fallback readers or automatic rewrites of consumer Serde.
Clients and runtime must be regenerated together after contract changes.

Qualification uses independent literals, allowance boundaries, allocation probes
and Rust/TypeScript exchange. Narrow source provenance remains with the owning
contracts; no historical implementation is a dependency or test oracle.

Changing a package version does not make retained data safe to reinterpret.
Consumers own retained requests, signatures, hashes, browser rollout and persisted
state. Actual migration, publication and deployment require separate authority;
this design is not authorization for them.

## Framing and portable API

A frame has an 11-byte header followed directly by its Postcard payload:

| Bytes | Meaning |
| --- | --- |
| 0–1 | ASCII `IW` |
| 2–9 | Method ID, little-endian u64 |
| 10 | Request `0` or response `1` |
| 11 onward | Payload |

`Encoding::Payload` selects bare payloads; `Encoding::Frame { method, direction }`
selects frames. `method_id` hashes UTF-8 `ic-wire:method:` plus the exact exported
name with FNV-1a-64. This is routing identity, not authentication or a schema hash.
Host method generation rejects duplicate IDs and export names.

The root API exposes `encoded_len`, `encode`, `encode_into`, `encode_to_slice`,
`decode` and `validate_request_header`. Field/variant order, Serde attributes and
adapter selection are protocol inputs. Accepted redundant varints and Postcard
char behavior are covered by explicit tests; decode is not a canonicality proof.

- Owned encoding returns a complete buffer after sizing/admission.
- Reusable Vec output is empty on error; capacity retention is not promised.
- Slice encoding rejects short storage before writing. Other failures may modify
  storage; only a successful returned prefix may be published.
- Decode accepts exactly one value and rejects trailing input. Borrowed views
  refer to the selected input and retain its lifetime.
- Fixed-header validation checks identity/direction only, without payload,
  authentication or resource validation.

Custom serializers must be deterministic across passes. Second-pass writes stay
within admitted storage even if trusted code changes behavior. There is no cached
prepared-encoding token. A scratch/callback helper remains excluded without a
named caller needing its distinct lifetime contract.

## Admission and fields

`Limits` specifies bytes, conservative allocation charges, nodes and depth.
The private meter owns sticky refusal and checked accounting. Encode sizing,
writing and decode have distinct visit sequences; their allowances are not
interchangeable with browser budgets. The [admission ledger](design/admission.md)
owns exact accounting and failure precedence, including frame transformations.
Typed errors are `Limit`, `Allocation`, `Malformed`, `Trailing` and `Identity`.

Preflight precedes codec output allocation. Field adapters admit their span and
bounds at their own boundary; this does not promise whole-document validation
before every allocation. Trusted custom Serde may allocate/work independently,
and native type layout affects charges. [Allocation qualification](../qualification/README.md#allocation-qualification)
records positive controls and limits of those claims.

`Bytes`, `Text`, `List`, borrowed byte/text views, `ByteBuf` and `ByteArray` preserve
explicit field choices. `Packed<T, N>` has an optional element bound and a fallible
constructor; its prefix counts bytes and its words are little-endian. Ordinary
vectors retain ordinary Serde element encoding. Bool, finite-float, nonzero-i64
and heap-u64 adapters have separate domain checks. Nonzero validation follows
packed allocation; heap reconstruction may change re-encoded ordering. No unsafe
unaligned casts or automatic field normalization are introduced.

## Host graph and browser clients

`SchemaRegistry` collects named Rust `Wire` projections and finishes into
`ValidatedRegistry`. `ValidatedRegistry::from_metadata` imports explicit JSON
through the same admitted graph. Input JSON allocation and manual projection work
are caller-owned. The graph has private indices and immutable structure; failed
collection cannot be reused as an apparently valid registry.

The [schema contract](design/schema.md) covers shared/recursive shapes, concrete
generics, borrowed projections, naming, collisions and resource limits. Unresolved
references, unsupported attributes and alias-only cycles reject. Derive projects
supported Serde declarations without generating serialization. Manual projections
remain trusted claims requiring independent byte vectors.

Rendering and rooted advice consume the graph. Use-site bounds remain distinct
from reusable type structure. Diagnostic traversal, retained paths and displayed
output are bounded; acknowledgements require exact paths and review reasons.
Payload/method generation requires a diagnostic handler; consumers display the
report or enforce `is_clean()`.

Generated modules use private metadata and require the `ic-wire-json` interface
marker. This is an unversioned tooling marker, not a wire-version discriminator
or proof of arbitrary runtime compatibility. Ship the bundled `RUNTIME` with the
generated clients. One interpreter handles all admitted kinds; there are no
per-record byte writers or separate npm runtime.

Browser byte/node/depth budgets do not represent Rust allocation or JS heap.
Wide integers use BigInt; selected input spans, strict object shapes, UTF-8 and
owned bulk results are qualified separately. Named synchronous checks and
getters/proxies are trusted work outside the traversal budget. Consumer Serde
owns matching Rust domain constraints.

## Qualification and performance limits

Use focused tests for the affected owner: literal byte/frame vectors, malformed
input and allowance boundaries, custody, schema rejection, Rust/TS exchange,
standalone consumers and optional feature graphs. [Qualification](../qualification/README.md)
identifies commands and prerequisites. [Release preparation](release.md) adds
archive and browser verification; old recorded passes do not prove new inputs.

Opt-z total uncompressed Wasm is the primary size measure; opt-3, code sections,
IC instructions, allocation behavior, payload bytes and native/host timings are
reported separately. Preserve ordinary small-value controls and count consumer
glue. Do not optimize gzip or silently relax a failed experiment threshold.

[Performance qualification](design/performance.md) describes current-only Wasm
size and installed instruction measurements. Correctness and allocation checks
do not establish speed. Browser evidence remains a Chromium smoke check, not a
cross-engine matrix; actual big-endian host execution is not established.
