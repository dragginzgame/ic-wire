# Host schema experiment

The optional `typescript` feature admits explicit JSON definitions into
`ValidatedRegistry`, then renders type aliases and exports reference-format JSON
from its private typed nodes.
It is native-only: forwarding the feature on Wasm excludes both the module and
serde_json. The codec-only graph is unchanged.

The destination contract is one admission boundary and one immutable authority.
Rendering does not read JSON or revalidate shapes. The initial caller is explicit
metadata for the representative record, exercised by `tests/schema.rs`. Manual
metadata remains a trusted claim about consumer Serde; validation cannot prove
that arbitrary serialization code matches it.

## Admitted subset and compatibility

Supported reference kinds are unit, bool, signed/unsigned integers, text, bytes,
packed-i16, list, option, newtype, ref, record and enum. Field and variant order,
supplied names, numeric widths, bounds and enum IDs are retained. Names are not
generated or renamed. TypeScript constraints are informational comments, not
runtime enforcement. References render as names; shared graphs never expand into
root-to-leaf paths. Options preserve `null | { readonly some: T }`.

Unknown keys and unsupported shapes reject explicitly. Unlike the reference,
this subset does not admit custom checks, all scalar/collection kinds, methods,
payload policies, or automatic Rust metadata collection. It is an experimental
API, not a replacement for the full reference host surface. No consumer source,
wire acceptance, byte, identity or persistence migration is approved here.

The reviewed source is `crates/ic-wire/src/typescript/metadata/mod.rs` at
`2dc434b10bec2fca92345d940dbfedc05993e9d5` in the read-only reference. Reuse is
limited to metadata grammar, identifier restrictions/reserved words and emitted
type representations, under the retained repository license. The importer,
typed storage, alias worklist and bounded renderer are destination implementations;
the reference generator and registry were not transplanted. serde_json 1.0.154
matches the reviewed reference lock and is used only for the import boundary.

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
These focused checks passed with Rust 1.100.0. Normal/build dependency trees for
the default native selection and TypeScript-enabled Wasm selection contain no
serde_json. Existing supplier proc macros remain; this is not a macro-free claim.

`metadata_json()` exports only admitted graph nodes, never the original input.
Round-trip tests preserve all admitted kinds, signedness, bounds, IDs, recursive
references and escaped/prototype-like names. It returns JSON data, not an object
literal or executable module; consumers must parse it as JSON. No versioned runtime
pair is claimed by this reference-format registry alone.

The [cross-language qualification](../../qualification/README.md) is its first
runtime consumer. Candidate Rust exchanges four representative records in payload,
request and response form with published Core 0.22.4's TypeScript runtime under
Node. It verifies literal bytes, both decode directions, selected spans and owned
bulk storage. This does not install the reference runtime into the product.
The 0.23.1 checkpoint adds recursive and shared named-record exchange, exact
reference TS node/depth boundaries and cyclic-value depth refusal. Schema sharing
does not become decoded-value aliasing. These cases use the same graph exporter
and runtime harness as the representative record.

Automatic collection, naming/collision qualification, method admission, executable
versioned metadata, matched browser runtime, TypeScript compiler validation,
advisory traversal and reference performance comparisons remain outstanding.
No Wasm size or IC instruction improvement is claimed.
