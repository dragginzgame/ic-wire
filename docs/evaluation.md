# First implementation batch

## Outcome

The initial [payload screen](codec-screen.md) implements steps 1–2 with bounded
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

Produce a reproducible codec comparison and one complete typed developer journey.
Do not adopt a codec or promise savings solely from source size or symbol names.
The maintainer must accept the resulting direction before registry publication.

The [shared-codec assessment](shared-codec-evaluation.md) treats reuse by IcyDB
as an optional hypothesis. Include an IcyDB-shaped schema artifact in the local
fixtures and evaluate its bounded binary approach as a candidate alongside the
existing-codec control. Sharing must earn its place through measured benefit and
reduced consumer implementation; it is not a selection requirement. Database
storage and ordered-key formats remain outside initial implementation scope.

## Work sequence

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
