# Prepared endpoint macros without manual IDs

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

This is an **isolated implementation trial**, following the
[fingerprint investigation](contract-fingerprint.md). The main
production macro, dependencies and codec remain unchanged. The supplying Account
adapter is still a separate prepared cut. No publication or sibling edits.

The prepared macro accepts:

```rust,ignore
#[ic_wire_endpoint::query(contract = ECHO, request = LIMITS, response = LIMITS)]
pub fn echo(request: Request) -> Request { request }
```

`id = ...` is rejected. `contract = ECHO` names the typed metadata declaration;
it is not a numeric revision. The eight-byte frame identity remains, supplied by
the generated contract artifact. Low-level native framing still takes a key.
The server has no old-key reader, runtime schema traversal or runtime hashing.

## Working build boundary

1. A host export build uses `IC_WIRE_SCHEMA_EXPORT=1`. The macro emits the actual
   typed declarations without Wire entrypoints; all exported methods must expose
   metadata for the inventory. Both endpoint attributes and framework-only
   `contract_key!` hooks refuse this mode on Wasm. It is not a host RPC mode.
2. The private producer projects those declarations through the existing `Wire`
   trait, generates fingerprints and browser/Rust constants, and records ordinary
   Rust/Cargo source hashes. It compares source snapshots before and after export.
3. The normal build selects the JSON artifact with `IC_WIRE_CONTRACTS`. Macro
   expansion verifies its source receipt, complete digests, method coverage, mode
   and all four request/reply policy values. Literal file/environment dependencies
   invalidate cached expansions. Builtin macros use fully qualified paths.
4. Regenerating with `--check` compares the artifact against current typed
   projections, including a deliberately altered layout with a recomputed digest.
   Final artifact/target qualification remains required.

The framework key hook reads the same artifact. The qualification mapper retains
historical IDs only in host test controls; no compatibility mapper enters Wasm.
The inventory covers all 17 real fixture exports, including framework composition.

For the intended hard cut, the old frame header for echo is
`49 57 07 00 00 00 00 00 00 00 00`; the generated header is
`49 57 d3 84 48 66 a4 c7 78 66 00`. Payload bytes and the 11-byte header size
are unchanged. Consumers regenerate bindings together, refuse old keys before
payload/effect work, and resolve retained/in-flight operations without rewriting
their original bytes. Unframed persisted data needs its own migration decision.

## Measured costs

Baseline: the preceding fingerprint prototype with explicit generated ID
expressions, frozen at `/tmp/ic-wire-contract-fingerprint-20261102/endpoint-after`.
Candidate: `/tmp/ic-wire-contract-artifact-20261102`. Rust **1.99.1**, wasm32,
fat LTO, one codegen unit, abort, stripped, same inputs and runtime supplier
versions. Only the prepared macro/host dependencies change. This stage keeps all
17 fingerprints identical to the preceding prototype.

| Profile | Total uncompressed Wasm, before → after | Code section, before → after |
| --- | ---: | ---: |
| opt-z | 100,853 → 100,853 | 86,922 → 86,922 |
| opt-3 | 94,343 → 94,343 | 81,366 → 81,366 |

All **356 case/profile pairs** match in every instruction, cycle and page-cost
sample. There are 178 cases per profile, one excluded warmup and eight measured
unique-caller samples: **2,848 new measured calls**, compared with 2,848 retained
baseline calls. All 3,204 requests including warmups have matching request hashes.
The instrumented server is actual version **16.1.0**, SHA-256
`b930e8bb2d5a1dbed7281a6a8717565c80576fca4c857b5b0a2cabcfc56842ff`.
Testkit 0.80.1 owns admission/lifecycle. Runtime/macros/guard remain 0.18.1,
Types 0.17.0, facade 0.18.0, Build 0.74.0 and Host 0.74.1.

Late host-export/hygiene fixes were rebuilt from the final source in a fresh
`qualified-target`; both guests are byte-identical to the measured binaries.
The Wasm runtime graph excludes SHA2, JSON, Candid and simulator dependencies;
SHA2/JSON are compiler-process dependencies of the endpoint procedural macro.
No compressed-size objective, new Candid matrix or native codec timing claim.

Host verification is currently expensive because it repeats the source scan for
each lookup. Eleven alternating samples in one release-profile unit binary,
with warmed filesystem buffers and 115 captured source files, measure this
**key-resolution component**, not full macro expansion or complete builds:

| Lookups | Old name-expression tokens | Checked artifact tokens | Added host time |
| --- | ---: | ---: | ---: |
| 1 | 0.00160 ms | 20.160 ms | 20.158 ms |
| 17 | 0.02632 ms | 343.886 ms | 343.860 ms |
| 100 | 0.15554 ms | 1,993.843 ms | 1,993.687 ms |

This is a substantial host regression, not runtime overhead. The 100-lookup
control cycles the 17-method inventory; it is not 100 separately derived Wasm
types. Sharing verification within a compiler invocation is a next experiment,
before adopting this boundary broadly. Raw samples and percentage changes remain
in Retired raw output (see retention notice).

## Checks and limits

- Incremental ordinary builds refuse missing artifacts, stale request types,
  manual IDs, altered digests, missing methods, policy mismatches and another
  source root. Restoring environment/source inputs rebuilds correctly.
- Field reordering changes the regenerated key; restoring source restores every
  key. A source-only comment changes the receipt but leaves all keys unchanged.
- Typed generation rejects a rehashed wrong layout; the producer refuses a
  concurrent source edit. Three fixture tests, strict Clippy, caller macro
  shadowing, Rust/TypeScript's 16 frame vectors and installed old-key/admission/
  borrowing/async/rollback checks pass. Both framework-only and attribute export
  modes refuse Wasm.
- The unchanged research fingerprint format still needs a stable specification
  and explicit 64-bit collision policy before activation. Current inventory
  collisions reject; this is not authentication or a collision-free identity.
- Endpoint projections remain finite; recursive graphs refuse. Qualify registry
  projection/recursive identity before making this the default.
- Receipts cover ordinary Rust/Cargo files under the selected root and require
  immutable inputs during compilation. They do not attest arbitrary external or
  generated inputs, feature-dependent or target-dependent layouts. Mutable path
  dependencies need explicit ownership beyond this fixture root. Registry inputs
  are locked; consumers still need paired target/byte qualification.
- Manual `Wire` metadata is still an assertion about custom Serde. A custom
  serializer with stale metadata can retain its key. Hashing source for freshness
  cannot turn false metadata into a true schema. Keep byte/refusal proofs.
- The private producer is fixture-specific, not a new supported build tool. A
  reusable consumer workflow and migration of the old compiler diagnostic suite
  remain before activating this API. This trial does not claim full CI passes.
