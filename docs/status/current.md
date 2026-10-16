# Current handoff

Updated 2026-10-16. The maintainer requested this local ic-wire scaffold after
investigating removal of Candid from Canic-owned traffic. The name must remain
unreserved on crates.io until the direction, usability and benefit are accepted.

## Accepted scope

- Shared Rust codec ownership outside host-only libraries.
- Postcard selected for controlled inter-canister calls and owned user endpoints;
  wire contract and public API remain unfrozen.
- Canic, IC Relay, IcyDB and native clients are intended consumers.
- Developer ease comparable to Candid, strong documentation and significant
  measured benefits are requirements, not established properties.
- External Candid contracts remain supported through their owning adapters.
- No source adoption in sibling repositories is authorized by this setup.

## Implemented

A virtual Cargo workspace, dependency-free no_std crate with no public API,
publication disabled, initial license and design/evaluation documents.
No production runtime codec, derive, endpoint adapter or wire contract is implemented.
An evaluation-only example compares four encodings on seven synthetic fixtures;
see [the payload screen](../codec-screen.md) for results and reproducibility.
The production library's codec dependencies are development-only. A separate,
unpublished evaluation package owns matched canister and PocketIC fixtures.
The 0.1.1 batch added `make evaluate-journey`: a native typed balance
request/reply experiment with explicit layouts, identity/direction checks and
exact-byte retry retention. See [developer experience](../developer-experience.md).
It is not an installed IC endpoint or an accepted runtime API.
The pending 0.1.2 batch generates scalar request serialization from an explicit
schema declaration, distinguishes retained request/reply types and adds four
compiler rejection checks. The existing six-byte request vector is preserved.
The library manifest stays at 0.1.1; its stale lockfile entry is synchronized.
The same pending batch now includes the
[matched canister experiment](../canister-comparison.md): three scalar/blob
inputs, four installed artifacts, real management calls, Wasm sizes and separate
strict-decode/handler/encode instruction-counter evidence. Postcard improves both
measured codec regions here; complete Wasm reduction falls from 88.2% to 6.83%
when the fixture retains Candid management traffic. This is not an application
savings projection. These results preceded the maintainer's Postcard selection.
The same pending batch adds the [browser trial](../browser-interop.md): generated
JavaScript/TypeScript bindings, a bounded typed client, and three signed Chromium
updates against local PocketIC. Rust vectors match exactly, including large u64
values; 4,264 truncated replies and wrong-root certificate verification reject.
Generated decoder allocation and number/bigint behavior require the wrapper.
Shared Tooling 0.15.4 is vendored at reviewed committed revision
`879eb2b021522252a67e908dbc217a0f4efb2a6f`, with its manifest, common Make
interface, setup catalogs/installers and formatting hook. Local Make targets and
Linux/macOS CI cover scaffold checks, Wasm, MSRV and contributor tooling.
See [development](../development.md). Maintainer-operated release entrypoints now
use the shared runner and pinned Cargo metadata adapter for direct origin/main
Git delivery. Package publication remains disabled. Source-bound validation and
prepared-byte receipts protect interrupted delivery; release execution is separate
from ordinary agent work.

## Tooling validation (2026-10-16)

The 0.1.2 prototype passes eleven example tests and four compile-fail cases on
Linux. Its generated schema remains limited to nonempty scalar records.
The matched canister fixture passes 36 typed replies, 60 malformed-call rejections
and two real management calls in local PocketIC. Host/Wasm checks pass on Rust
1.99 and 1.91; the production library's normal dependency graph remains empty.

On Linux x86_64, explicit `make install-tools` and offline `make tools-check`
passed for the complete common toolset. Snapshot integrity, local documentation
links, native formatting and the local Make/hook fixture passed. The fixture
qualifies actual Cargo target routing with substitutes, shared release routing,
native hook formatting, unrelated edit preservation and partial
staging rejection in disposable Git repositories. ShellCheck and actionlint
passed for the consumer script and workflow.

Native/Wasm library checks passed on Rust 1.91.0; the development Wasm check
passed on Rust 1.99.0. These remain empty-scaffold compilation evidence.
No full local `make ci` or sibling suite was run. Initial hosted Ubuntu CI passed;
macOS arm64 failed the Make fixture's temporary-path comparison. The failure was
reproduced with `TMPDIR=/tmp/` on Linux and corrected by resolving the fixture's
physical path. Hosted macOS qualification of the fix is still outstanding.

At the maintainer's request, initialized local Git on `main`, activated the
formatting hook and created the private
[dragginzgame/ic-wire repository](https://github.com/dragginzgame/ic-wire).
`origin` points to that repository, whose description identifies this as an
unpublished investigation. Earlier Git-dependent checks used disposable exports;
the initial Shared Tooling snapshot came from clean disposable Git checkouts.
Package publication remains disabled. The local release fixture qualifies Git
version/tag/push and recovery with substituted setup/validation gates; no real
repository release was executed while wiring the adapters.

## Next action

The [shared-codec assessment](../shared-codec-evaluation.md) reviewed pinned
IcyDB source and keeps sharing optional. Bounded message/schema serialization
has useful overlap; row projection, ordered keys and authenticated/recovery
envelopes retain distinct requirements. The subsequent payload screen gives
Postcard the smallest bytes for all seven fixtures, but demonstrates its implicit
field-order hazard. CBOR exceeds Candid on three fixtures. The fixed comparator
does not beat Postcard. The subsequent installed-canister comparison supplies
bounded fixture evidence. Postcard is selected; no sibling changes have been made.

The browser trial passes on Chromium 156, Node 24.21.0 and SDK 6.1.1. Its generated
bindings cover only the scalar/blob fixture; query endpoints, login delegation,
frontend rollout and general generated bounds remain unqualified. Historical
canister size evidence retains its original source/lock hashes and was not
remeasured after adding browser binding generation.

Continue docs/evaluation.md by extending installed measurements to nested authority
and schema shapes, and qualifying generated-layout ergonomics plus intended
Canic/client integration. Explicit tuple conversions demonstrate declaration-order
independence locally; schema identity and general bounds remain unresolved.
Include one local IcyDB-shaped schema artifact to test the sharing hypothesis.
Keep all candidates unpublished and compare equal-behavior Wasm builds; IcyDB
adoption is not a prerequisite for choosing IC Wire's encoding.

Read docs/evidence.md before citing size numbers. Historical symbol attribution
suggests a substantial opportunity but establishes neither net savings nor speedup.
The existing Relay delivery fixture already uses bounded binary bytes; its runtime
remains a scaffold. IcyDB storage and ordered-key encoding are outside scope.

The scaffold is not ready for publication or consumer adoption. Initial validation
is recorded separately in docs/scaffold-validation.md.
