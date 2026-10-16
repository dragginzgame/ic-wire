# Current handoff

Updated 2026-10-16. The maintainer requested this local ic-wire scaffold after
investigating removal of Candid from Canic-owned traffic. The name must remain
unreserved on crates.io until the direction, usability and benefit are accepted.

## Accepted scope

- Shared Rust codec ownership outside host-only libraries.
- Canic, IC Relay, IcyDB and native clients are intended consumers.
- Developer ease comparable to Candid, strong documentation and significant
  measured benefits are requirements, not established properties.
- External Candid contracts remain supported through their owning adapters.
- No source adoption in sibling repositories is authorized by this setup.

## Implemented

A virtual Cargo workspace, dependency-free no_std crate with no public API,
publication disabled, initial license and design/evaluation documents.
No runtime codec, derive, endpoint adapter or wire contract is implemented.
An evaluation-only example compares four encodings on seven synthetic fixtures;
see [the payload screen](../codec-screen.md) for results and reproducibility.
All external codec dependencies are development-only.
Shared Tooling 0.15.4 is vendored at reviewed committed revision
`879eb2b021522252a67e908dbc217a0f4efb2a6f`, with its manifest, common Make
interface, setup catalogs/installers and formatting hook. Local Make targets and
Linux/macOS CI cover scaffold checks, Wasm, MSRV and contributor tooling.
See [development](../development.md). Release entrypoints reject before the
shared runner; no release delivery adapter is configured.

## Tooling validation (2026-10-16)

On Linux x86_64, explicit `make install-tools` and offline `make tools-check`
passed for the complete common toolset. Snapshot integrity, local documentation
links, native formatting and the local Make/hook fixture passed. The fixture
qualifies actual Cargo target routing with substitutes, release refusal before
the runner, native hook formatting, unrelated edit preservation and partial
staging rejection in disposable Git repositories. ShellCheck and actionlint
passed for the consumer script and workflow.

Native/Wasm library checks passed on Rust 1.91.0; the development Wasm check
passed on Rust 1.99.0. These remain empty-scaffold compilation evidence.
No full `make ci` or sibling suite was run. Native macOS execution and hosted CI
are not observed here.

At the maintainer's request, initialized local Git on `main`, activated the
formatting hook and created the private
[dragginzgame/ic-wire repository](https://github.com/dragginzgame/ic-wire).
`origin` points to that repository, whose description identifies this as an
unpublished investigation. Earlier Git-dependent checks used disposable exports;
the initial Shared Tooling snapshot came from clean disposable Git checkouts.
Package publication and release delivery remain disabled.

## Next action

The [shared-codec assessment](../shared-codec-evaluation.md) reviewed pinned
IcyDB source and keeps sharing optional. Bounded message/schema serialization
has useful overlap; row projection, ordered keys and authenticated/recovery
envelopes retain distinct requirements. The subsequent payload screen gives
Postcard the smallest bytes for all seven fixtures, but demonstrates its implicit
field-order hazard. CBOR exceeds Candid on three fixtures. The fixed comparator
does not beat Postcard. No speed, IC instruction or Wasm-size win is established;
no sibling changes or codec selection have been made.

Continue docs/evaluation.md with one complete typed endpoint/client journey. Resolve stable
field identities, canonical bytes and decode limits before freezing a format.
Include one local IcyDB-shaped schema artifact to test the sharing hypothesis.
Keep all candidates unpublished and compare equal-behavior Wasm builds; IcyDB
adoption is not a prerequisite for choosing IC Wire's encoding.

Read docs/evidence.md before citing size numbers. Historical symbol attribution
suggests a substantial opportunity but establishes neither net savings nor speedup.
The existing Relay delivery fixture already uses bounded binary bytes; its runtime
remains a scaffold. IcyDB storage and ordered-key encoding are outside scope.

The scaffold is not ready for publication or consumer adoption. Initial validation
is recorded separately in docs/scaffold-validation.md.
