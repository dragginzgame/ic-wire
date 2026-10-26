# Current handoff

## Released 0.7.14; pending 0.7.15

The checkout started clean at `5cfd762` (Release 0.7.14). Adopted Shared Tooling
0.26.18 at `37652c087fff16f55f83d18d27ba17c83f34021c` through the canonical
exporter after reviewing its impact report. AGENTS.md and the 107-file snapshot
record the selected revision. Pins, compilers, Cargo dependencies and own package
versions remain unchanged. No commit, push, release or publication was performed.

## What the update fixes

Normal releases start fresh from current committed metadata instead of scanning
historical release plans. Explicit `release-resume VERSION=X.Y.Z` still owns
recovery and checks the saved source, validation receipt, refs and destination.
The local adapter fixture now includes malformed historical plans and verifies
that normal release delivery succeeds while preserving that historical evidence.
Its dirty-source assertion follows the new runner's earlier refusal diagnostic.
All release effects in qualification are confined to disposable repositories and
local bare remotes.

Cargo publication helpers did not change between the adopted snapshots. This
update does not make an already-uploaded version replaceable or supply a missing
derive dependency. The checkout is 0.7.14, so a runtime-only retry requires derive
0.7.14 to be available. Select both for a new version:

```sh
make publish PUBLISH_PACKAGES='ic-wire-derive ic-wire'
```

The previous non-uploading Cargo dry run for that exact selection passed. A
runtime-only retry applies to the same already partially published version;
advancing the workspace version requires publishing its matching derive too.
Actual registry visibility and successful uploads still need maintainer execution.
See [publication recovery](../development.md#first-publication).

## Validation and scope

Snapshot integrity, local Make/hook/publication routing, disposable release and
resume/receipt checks, formatting, documentation links and whitespace pass.
Tool catalogs/pins are unchanged, and existing tool admission passed without
installation. No production codec/generator code changed, so runtime benchmarks
are not applicable. Full CI and fresh macOS qualification were not run.

The new RPC-budget rule keeps policy with consumers and codec semantics with Wire;
no default limits or runtime policy were introduced by snapshot adoption.
The GitHub description was corrected from "unpublished investigation" to the
published bounded Serde/Postcard contracts and generated TypeScript scope, as
required during baseline adoption. The sibling Shared checkout stayed unchanged.
The released [method-ID qualification](../evidence/method-ids-0713/README.md) and
[unsigned evidence guide](../unsigned-magnitude.md) retain their existing scopes.
