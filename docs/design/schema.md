# Schema contract

The optional `typescript` feature admits explicit JSON definitions into
`ValidatedRegistry`, then renders type aliases and exports schema JSON
from its private typed nodes.
The same module and serde_json dependency are enabled on native and Wasm.
Codec-only builds omit both.

The destination contract is one admission boundary and one immutable authority.
Rendering does not read JSON or revalidate shapes. The initial caller is explicit
metadata for the representative record, exercised by `tests/schema.rs`. Manual
metadata remains a trusted claim about consumer Serde; validation cannot prove
that arbitrary serialization code matches it.

## Admitted subset and compatibility

Supported schema kinds are unit, bool, signed/unsigned integers, text, bytes,
finite float, packed integer/opaque words, list, array, tuple, result,
option, newtype, ref, record and enum. Field and variant order,
supplied names, numeric widths, bounds and enum IDs are retained. Imported names
are not regenerated or renamed. TypeScript constraints are informational comments, not
runtime enforcement. References render as names; shared graphs never expand into
root-to-leaf paths. Options preserve `null | { readonly some: T }`.

Unknown keys and unsupported shapes reject explicitly. This subset does not admit
every possible Serde projection. Scalar bounds and named custom
runtime checks are supported. Method identities and payload policies are supplied
through the separate generation APIs. It is an experimental
API for a hard cut, without a legacy compatibility requirement.

The reviewed source is `crates/ic-wire/src/typescript/metadata/mod.rs` at
source revision `2dc434b10bec2fca92345d940dbfedc05993e9d5`. Reuse is
limited to metadata grammar, identifier restrictions/reserved words and emitted
type representations, under the retained repository license. The importer,
typed storage, alias worklist and bounded renderer are destination implementations;
serde_json handles the import/export boundary.

## Admission and output bounds

`SchemaLimits` bounds named definitions plus inline nodes, retained name bytes,
inline depth (at most 64), and emitted UTF-8 bytes. Inline syntax descends only
within that bounded depth. Named references are resolved once and do not descend.
An iterative linear alias walk rejects cycles made entirely of aliases/references;
productive records, lists, options and enums may recurse. Field/variant arrays
consume child-node allowances. Duplicate names and enum IDs reject. Errors have
two fixed categories, without unbounded path diagnostics.

JSON parsing and its allocations happen before this API and belong to the caller.
The limits bound logical graph size, not exact heap capacity or process memory.
The shared output writer checks each append, including escaped labels, before
extending the string. The byte allowance applies separately to each declaration
or metadata artifact. Failure exposes no partial artifact. BTree maps/sets add logarithmic
lookup cost; syntax validation and rendering visit each stored node once, without
enumerating shared paths. Field-bound constraints stay at their individual uses.

## Focused evidence and remaining gates

`make test-host` covers a literal representative declaration, escaped names,
input mutation isolation, invalid metadata, duplicate fields/IDs, missing refs,
alias cycles, productive recursion, exact host allowances and output refusal.
A 1,000-definition diamond stays below 100 KB of declarations; adding an unrelated
definition preserves existing output. This is structural evidence, not a host
performance comparison. `make lint-host`, `make doc-host` and
`make check-wasm-host` check the optional module and its target boundary.
Codec-only native and Wasm builds omit `serde_json`. Enabling `typescript`
selects it on both targets. Generated closed enum projections use Serde derives in the build-time dependency graph.

`metadata_json()` exports only admitted graph nodes, never the original input.
Round-trip tests preserve all admitted kinds, signedness, bounds, IDs, recursive
references and escaped/prototype-like names. It returns JSON data, not an object
literal or executable module; consumers must parse it as JSON. No versioned runtime
pair is claimed by this schema registry alone.

The [cross-language qualification](../../qualification/README.md) is its first
runtime consumer. Rust exchanges representative and recursive/shared records
in payload, request and response form with the current bundled TypeScript runtime
under Node. It verifies literal bytes, both decode directions, selected spans,
owned bulk storage, node/depth boundaries and cyclic-value refusal. Schema sharing
does not become decoded-value aliasing. Archive consumers and a Chromium smoke
check are described in the [release guide](../release.md); these checks do not
establish full client-generation cost or cross-browser coverage.

## Generated payload modules

`payload_module(export, root, runtime_module, PayloadLimits, acknowledgements, diagnostics)` renders an actual
typed client from the same graph. It admits an existing root and an independent
codec export name, rejects names that collide with generated types or shadow
`globalThis`, and requires explicit browser byte/node/depth allowances. Zero bytes
is valid for unit payloads; nodes/depth must be positive and all limits fit exact
JavaScript integers. This does not infer browser policy from Rust allocation or
claim equal traversal accounting.

The imported runtime must export `schemaFormat = "ic-wire-sparse-v1"` and
`createPayloadCodec(metadata, registry)`. That factory uses the exported JSON
grammar and returns encode/decode functions. A mismatch throws before factory
creation. The marker identifies an interface, not cryptographic supplier identity;
applications must select and qualify their runtime implementation. Qualification
uses a private observation adapter that forwards the runtime's actual marker.

The module exports type aliases and a frozen codec object. Its JSON registry is
parsed into private module storage, preserving prototype-like keys without exposing
mutable metadata to callers. Generated codecs delegate to one runtime interpreter;
no per-type byte writers are emitted. Runtime module specifiers are escaped strings
selected by the caller, not resolved or downloaded by Wire.

The total source shares the configured output allowance, including declarations,
quoted metadata and client code. JSON string escaping uses one additional bounded
metadata buffer, so this is not a single-buffer memory guarantee. The generated
client is now the payload caller in the representative Rust exchange; format
mismatch and actual supplied policy are checked at runtime. Automatic Serde
projection beyond the strict subset below remains separate implementation work.
The bundled interpreter is exported as `typescript::RUNTIME`; its scope and
reuse contract are recorded in [browser runtime](browser-runtime.md).

## Generated method modules

`method_module(&[Method], runtime_module, acknowledgements, diagnostics)` emits request/response codecs using the
same name/policy admission, runtime-format prelude, declarations, JSON export and
bounded writer as payload modules. The supplied runtime exports `createCodec`.
All methods share one private metadata registry; each exported frozen object has
typed encodeRequest/decodeRequest/encodeResponse/decodeResponse functions. No
endpoint executor, router or transport is generated.

Each Method supplies an explicit u64 ID, export name, request/response root names
and separate PayloadLimits. Byte limits exclude the header at this API; generation
adds exactly 11 and refuses totals beyond JavaScript's safe-integer range. Zero
payload bytes remains valid for unit schemas. Node/depth limits are unchanged.
IDs are decimal strings interpreted by the runtime as BigInt, never JS numbers.
Callers may explicitly use Rust `method_id` for name-derived identities; renaming
an exported codec alone does not silently change its routing identity.

Duplicate exports and IDs within a module reject before rendering. Root lookup is
binary search over admitted names; duplicate detection uses bounded sets, with
method count capped by the source-byte allowance before allocating them. All
declarations, shared metadata and method wrappers count toward one output ceiling.
Independent separately generated modules are not a global routing registry.

## Strict Rust metadata projection (0.23.5)

Use the included `Wire` derive on shared DTOs. On native and Wasm targets
the schema trait is available under the same name.
`ValidatedRegistry::from_type::<T>("Root", limits)` projects an inline root and
feeds it through the existing JSON admission boundary. The derive is a producer
of metadata, not another schema validator or serializer; the sealed graph remains
rendering's sole authority. The representative payload client now uses this path,
with its independently authored JSON retained as a qualification oracle.

Supported derives are named/unit/newtype/tuple structs and ordinary enums
with unit, single-value, tuple or named-field variants. Raw identifiers lose only the
Rust `r#` syntax. The default crate path is `::ic_wire`; renamed dependencies use
`#[wire(crate = "wire")]`. Type parameters gain a Wire bound; lifetimes,
const parameters, defaults and existing where clauses are preserved. Concrete
instantiations project their own field widths and bounds, without TypeId or a
`'static` requirement. The derive currently requires Wire for every type parameter,
including parameters that a custom wrapper might otherwise leave unconstrained.
Field `#[serde(borrow)]` and explicit borrow-lifetime lists are accepted because
they do not change bytes. Version 0.23.6 also accepts symmetric
`#[serde(rename = "name")]` on named fields and enum variants. Renames affect
generated JavaScript keys/tags while preserving Postcard field order and numeric
variant indices. Projected name collisions reject at the existing graph boundary.
Directional rename forms, rename_all, container renames, unnamed-field renames,
other Serde attributes except the sealed standard `with = "serde_bytes"` and
qualified `ic_wire::serde_packed::{bools,finite_f32,finite_f64,nonzero_i64,heap_u64}`
field projections, unsupported Wire attributes, explicit enum discriminants and
unions reject. Packed adapter paths follow the selected Wire dependency name.
This conservative subset does not silently approximate rename/flatten/skip/tagged
or custom serializer layouts. Serde remains the only payload implementation.

Native trait implementations cover fixed-width integers, finite float wrappers,
bool/unit, strings/Cow text, Option/Result, tuples through arity four, fixed arrays,
slices, Box/references, ordinary Vec/List, explicit Bytes/Text/Packed and
published ByteBuf/ByteArray plus borrowed byte/text adapters. Ordinary Vec<u8>
remains a list projection. The optional `platform-types` feature adds
Principal/Account projections without redefining supplier identities or Serde.

SchemaContext bounds nested projection calls and active Rust type depth. Active
recursive type labels reject in inline mode, and a failed nested call invalidates the context
so manual code cannot swallow failure and return an apparently admitted graph.
Construction work counts Rust projection calls, not exact JSON nodes. Final graph
admission independently enforces its exact node/name/depth limits. Producer JSON
allocations occur before that admission; arbitrary manual projection code remains
trusted and can do work outside the construction allowance. This is not an exact
heap cap. Nested types are inline: no new generated-name algorithm or type
identity promise is introduced in this inline API.
Use the named collector below for recursive/shared types.

The macro is the `ic-wire-derive` product package, authored for this strict contract.
Its selected syn/quote/proc-macro2 versions match the already locked suppliers.
Default codec builds omit the Wire macro. With `derive` enabled, the macro runs
on the build host and emits the same schema trait implementation for native and
Wasm consumers. Both can construct a registry and render TypeScript in memory.

`make test-derive` checks macro rejection and exact graph projections, construction
limits, recursive refusal and sticky failure. `make test-typescript` compiles and
executes the renamed-dependency derived client against literal bytes. Focused
`make check-wasm-host` executes a derived Platform schema on Wasm. Neither a full Serde attribute
surface nor standalone publication/browser/typechecker proof is claimed.

Generic qualification includes distinct u16/u64 instantiations, independent const
field limits, generic enum payloads, defaults/where clauses and a type parameter
named Result to check generated-path hygiene. A borrowed generic record uses a
non-static source lifetime and verifies decoded string/byte pointers into its
input. The generated-client exchange now starts from `Record<u8, 3>` and retains
the same independent schema and byte oracles.

## Named Rust collection (0.23.6)

Version 0.23.7 scale controls register 1,024 public scalar aliases in opposite
orders and require identical JSON/declarations with exactly one shared scalar
definition. Graph admission succeeds at 2,050 named-plus-inline nodes and rejects
2,049. A separate 1,024-root recursive fixture preserves every baseline shared
definition and adds only root aliases. These are bounded correctness checks;
they do not measure full generation or host allocation costs.

`SchemaRegistry::new(limits)`, `register::<T>(alias)` and consuming `finish()`
collect concrete Rust roots into the same JSON import boundary and return the
same immutable ValidatedRegistry. SchemaContext owns both inline and named
production; derives emit one projection implementation. Every nested concrete
Rust type receives one definition and subsequent occurrences emit references.
A reserved name breaks recursive expansion before its definition is complete.
Explicit aliases remain independent of discovery order. No TypeId or static
borrow requirement is introduced.

The naming algorithm was adapted from
`typescript/native_schema/registry/mod.rs` at
`2dc434b10bec2fca92345d940dbfedc05993e9d5`: a sanitized 48-character stem plus
FNV-1a-64 of the full compiler type spelling. Only this naming function is reused;
collection integrates with the destination context and sealed-graph admission.
The retained repository license applies. These names are compiler-source labels,
not guaranteed unique Rust type identities or wire/method IDs.

Each encountered projection callback is retained within the construction call
allowance. Finalization replays those callbacks with known children resolving to
references and compares the result with its collected definition. This catches
conflicting concrete types sharing a diagnostic spelling and nondeterministic
manual projections. It is not a proof against arbitrary malicious custom code.
Failed registration is sticky; finish cannot expose partially admitted metadata.
The existing graph validator rejects unresolved references and alias-only cycles.

Construction and verification share a finite call allowance; generated/root
names consume a separate name-byte allowance before insertion. The graph import
then independently enforces its exact nodes, labels and inline depth bounds.
References do not recursively expand during verification or rendering. JSON
producer allocations and manual work retain the limitations described above;
no exact heap cap or supplier-identical construction budget is claimed.

Focused evidence includes recursive Chain/Shared schemas, generated framed exchange from
actual derives, root-order stability, diagnostic-name and explicit-root collisions,
unstable projections, failure poisoning, alias cycles and non-static borrowed
roots. Full generation/allocation cost remains unmeasured. Unsupported Serde
attributes intentionally reject; full legacy surface parity is not a hard-cut
release requirement.

The [API manifest exporter](api-manifest.md) supports development-time drift detection using the admitted registry and method descriptors.

### Application-scoped bulk acknowledgements

An exporter can share one `BulkPolicy` across the reports from every canister
and method in an application. Register each decision with a stable advisory ID,
an exact declaration and a nonempty reason. For example, a typed application
generator could register the following decisions before its first method:

```rust,ignore
use ic_wire::typescript::{BulkAdvisory, BulkPolicy, BulkSite};

let mut policy = BulkPolicy::default();
policy.acknowledge(
    BulkSite::list_element::<ic_db::Ulid>(),
    BulkAdvisory::PackedList,
    "Keep typed ULIDs in the public request and reply.",
)?;
policy.acknowledge(
    BulkSite::field::<LaunchPolicy>("unlocked_rarity_tiers"),
    BulkAdvisory::OpaqueBytes,
    "Each element is a numeric tier.",
)?;
```

Apply the policy in the existing diagnostics callback and check stale decisions
once after processing the whole application:

```rust,ignore
let module = registry.method_module(&methods, runtime, &exact_paths, |mut report| {
    policy.apply(&mut report);
    eprint!("{report}");
})?;
// After the final method:
for (site, advisory, reason) in policy.stale() {
    eprintln!("stale {} acknowledgement for {site:?}: {reason}", advisory.as_str());
}
```

A generic host such as Canic can load decisions from application configuration.
Parse advisory IDs such as `packed-list` and `opaque-bytes` with
`str::parse::<BulkAdvisory>()`. For a JSON schema inventory, construct
`BulkSite::ListElement(label)` or `BulkSite::Field(record_label, serialized_field)`.
The application can obtain labels using `SchemaRegistry::type_label::<T>()`
with its build toolchain. Carry the exact named definitions through inspection.
Inline, anonymous elements have no declared type identity to match. A list of
another newtype with the same wire layout keeps its own advisory. Record fields
match across root aliases and endpoints, including whenever the same DTO is nested.

The per-module exact-path map still runs first and reports its own stale paths.
A declaration rule matches only remaining warnings of its specified kind.
Keep decisions alongside exporter inputs. They affect the diagnostic report
alone and are absent from generated clients, manifests and wire data.

For issue #7, a local Canic 0.264.0 exporter build applied these two rules
through the callback during `canic app export-wire` on Toko's seven-canister
fixture. The baseline emitted 51 advisories. The configured run removed 25
ULID and two rarity-tier advisories, retained all 24 cursor advisories, and
produced the same 157 files byte for byte. The released Canic 0.264.0 CLI
requires downstream integration to load an application policy.
