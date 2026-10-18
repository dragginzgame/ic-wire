# Evaluation scope and historical plan

Candid comparisons, `evaluate-runtime` and the old `evaluate-canisters` experiment
are retired. Reports below preserve their original source/build identities.
Current `evaluate-browser` and `evaluate-macros` test IC Wire endpoints only.
The maintainer-requested [performance experiment](performance.md) uses private
raw Postcard/bitcode/wincode controls, without adding production codec modes.
Durable receipt qualification moves to Relay; IC Wire retains browser transport
and codec byte conformance without receipt caching or peer-delivery policy.

## Outcome

The [Postcard runtime](runtime.md) and [installed adoption proof](runtime-adoption.md)
now qualify a controlled Rust-to-Rust adoption path. Historical comparisons below
retain their original bytes and build identities; their savings are not measurements
of every current runtime or consumer. The [recorded runtime comparison](runtime-comparison.md)
measured its selected envelope and bounds on a mixed bounded schema. Optional
TypeScript generation and installed browser correctness are also qualified.

The initial [payload screen](codec-screen.md) implemented steps 1–2 with bounded
synthetic fixtures and partial rejection evidence. It does not complete the typed
developer journey. The [matched canister experiment](canister-comparison.md)
now covers installed scalar/blob calls, linked Wasm and IC instructions for both
internal-only and management-enabled artifacts. The [nested authority extension](nested-authority.md)
adds finite delegate/grant trees and optional expiry, including maximum collection
inputs. Installed schema artifacts and production developer ergonomics remain
outside that measured subset. The subsequent
[browser trial](browser-interop.md) qualifies one generated TypeScript update
client against the installed Postcard fixture. Postcard is now the selected
engine for controlled RPC and owned user endpoints; adoption remains bounded
by the unresolved protocol requirements.
The historical [protected-command trial](command-replay.md) qualified bounded stable
receipts for synchronous local effects, including trap rollback and replay after
an identical Wasm upgrade. Delegated auth and external effects remain separate.

The sequence below records the original selection experiment, completed before
the first publication. It is not a request to reintroduce candidate codecs or
consumer replicas. Current work preserves the selected runtime and reproducible
comparisons without deriving savings from source size or symbol names.

The historical [shared-codec assessment](shared-codec-evaluation.md) investigated
IcyDB reuse. Its consumer-specific SQL projections have since been retired here;
IcyDB owns actual contracts, scalar validation and endpoint qualification. IC Wire
keeps generic finite/native conformance fixtures. Database storage and ordered-key
formats remain outside this repository.

## Historical work sequence

1. Freeze representative local fixtures: a small request/result, a bounded nested
   Canic-shaped authority payload, a byte-heavy payload and a relay batch/ACK.
   Record the source schema and value identities. Keep fixtures consumer-neutral;
   do not add full Canic or IcyDB dependency graphs to this runtime crate.
2. Compare current Candid with a compact schema-driven candidate and bounded CBOR.
   Inspect existing implementations and their licenses before choosing candidates.
   Keep benchmark-only Candid dependencies out of the library's normal graph.
3. Prototype derives and a shared typed method binding for the best viable
   candidate. Show all real annotations; assess ergonomics against Candid.
4. Build matched Wasm fixtures with identical compiler, optimization, endpoints,
   application work and external management functionality. Compare the complete
   linked artifacts, not a codec function removed in isolation. Include a pure
   internal fixture and a management-enabled fixture to distinguish codec retention.
5. Measure encode/decode IC instructions, payload bytes and code-section sizes.
   Record native timings separately; they are not IC instruction evidence.
   Record optional postprocessed/compressed sizes separately. Do not subtract
   unrelated historical builds to manufacture savings.
6. Exercise malformed lengths, nesting/element budgets, invalid tags, truncation,
   trailing bytes, canonical hash vectors and exact-byte retries. Use PocketIC
   for installed canister/endpoint behavior, not native mocks of management.
7. Document the selection and tradeoffs. If the result is not a meaningful net
   improvement with an equally simple developer experience, report that outcome
   and revise or stop the proposal rather than publishing a weak abstraction.

## Acceptance evidence

| Property | Required evidence |
| --- | --- |
| Usability | Compiling shared types, endpoint adapter and typed client; no manual codec plumbing |
| Wasm | Matched full-artifact results with remaining external Candid held constant |
| Execution | Encode/decode IC instruction results for each representative shape |
| Bounds | Typed rejection before oversized allocation or unbounded recursion |
| Determinism | Published test vectors and repeated byte-identical encodings |
| Independence | Codec normal dependency graph excludes Candid, CDK and host tooling |
| Documentation | Working quickstart, supported types, wire rules, limits and change policy |

No numeric improvement threshold has been accepted yet. Report absolute and
relative results per fixture, including regressions; do not invent an acceptance
percentage. The current Candid attribution is motivation, not the target saving.

The codec must not own transport retry policy, database storage, auth or fleet
control. Keep delivery invariants with Relay and business invariants with the
consumer. Do not add a generalized framework or extra crates to complete a table.
