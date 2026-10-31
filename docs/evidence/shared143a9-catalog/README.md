# Latest Shared Tooling complete-catalog adoption

2026-10-31. Frozen source is **143a9abb9ae81feae9400ff3844b03b1ddfdd75a**,
verified against remote HEAD and checked out clean in an isolated clone. Its
committed VERSION is **0.27.21**. The latest commit/changelog calls the layout
follow-up 0.27.22 without changing VERSION; the canonical snapshot accurately
records 0.27.21 and the exact commit. No independent annotation or payload edit
was made. The incoming uncommitted 0.27.20 / Platform / Testkit adoption is retained.

## Consumer changes

[Impact](impact.log) and [refresh](refresh.log) identify the canonical 119-payload
selection. The changed consumers are root Cargo metadata, release-policy aliases
and maintained documentation. No sibling checkout, package source/path/version,
release ledger location, registry dependency selection, Git index or release ref
was modified.

The [catalog conversion](catalog.patch) removes the duplicate header map. Each
Core/Endpoint group appears once as a contiguous root dependency block with a
short title and separate changelog line. All six workspace members have real
path/full-version declarations, including the three private qualification packages.
The Core alias roster now lists all five of its packages, so release preparation
updates unused entries too. External dependency blocks distinguish runtime,
native schema, host proc-macro implementation and test/tooling usage. Existing
bare version operators and grouped formatting are retained.

Unused workspace catalog declarations add no dependency edges. The
[preservation proof](preservation.json) records **full resolved Cargo metadata
equality and byte-identical Cargo.lock** before/after this conversion. This is
stronger than checking only package versions: feature selections, targets and
dependency edges are unchanged. Unrelated dirty work is retained.

## Focused checks

[Integrity/tool log](integrity.log) retains canonical snapshot verification,
dependency pin/inheritance admission, Cargo boundaries, grouped formatting and
offline tool checks. Host/Cargo catalogs and prepared Build 0.73.1 tools do not
change. [Local Make/hook and both release-group fixtures](adapters.log) qualify their real caller
wiring, metadata preparation, exact custody, refusal, receipt recovery and resume
in disposable repositories. Expensive validation and registry uploads remain
substituted; no real release or publication runs here.

Own versions remain Core **0.13.0** / Endpoint **0.2.0**, with pending patch notes
**0.13.1 / 0.2.1**. No protocol, codec/schema bytes, budgets, admission, malformed
acceptance or retry/hash inputs change. Runtime/generator sources and resolved
inputs are unchanged; a new performance benchmark is not applicable. Earlier
[Platform qualification](../platform0152-testkit074/README.md) and
[Testkit 0.75 qualification](../shared02720-testkit075/README.md) retain their
identified runtime, Wasm, MSRV, installed and published-consumer evidence. No fresh
full local CI, hosted/macOS matrix or Candid performance matrix was run.

## Reproduction

From the completed working source, with the already selected tools and registry
cache prepared:

```sh
make shared-tooling-check check-pins check-boundary fmt-check tools-check
make check-tooling check-release
make check-doc-links
```

The adapter fixtures operate only on disposable repositories. For snapshot
reproduction, obtain a clean clone of Shared Tooling at the recorded commit,
retain the exact canonical remote `https://github.com/dragginzgame/shared-tooling`,
then run its `scripts/distribution/refresh-consumer.sh --impact --source SOURCE
--consumer CONSUMER` and inspect the preview before the same command without
`--impact`. Do not manually copy or patch owned payloads. Original graph/input
snapshots and the isolated source are retained under
`target/adoptions/shared-tooling-refresh/`; essential proof is copied here because
explicit Cargo cleanup may remove that target directory.
