# Current handoff

## Released 0.7.19; pending 0.7.20 IC Build repair adoption

HEAD remains `793134aa4476b5cf7501004d6aab271687c7235f` / `v0.7.19`.
Own package versions remain 0.7.19. The compatible 0.7.20 changelog draft records
adoption of IC Build 0.69.6 for the library and managed release editor. Leave
staging, commits, releases, publication and pushes to the maintainer.

The root Build minimum, Make editor pin and setup docs now select 0.69.6.
At inspection, incoming Cargo.lock work had already selected Build/Testkit/host
0.69.6 and Platform 0.7.3. This entire incoming lockfile is preserved byte-for-byte
against `target/ic-build-adoption-0696/incoming.Cargo.lock`; no dependency update
command was needed. Testkit/Platform declaration minimums remain unchanged.
Shared Tooling remains 0.27.3 at `7ee5c83744aa822ecd070703dbf0a7d95076d5f6`.
No runtime, generator, protocol bytes, identities or admission rules change.
Build remains a qualification/host dependency, outside published runtime crates.

## macOS correction and qualification boundary

Published IC Build 0.69.6, source `cba072ee8fe288eb92bb4a4ddf9753b8abc4b580`,
contains the fix for [IC Tooling #262](https://github.com/dragginzgame/ic-tooling/issues/262).
Its source inventory keeps native `Mode` values through permission masking and
serializes `mode.bits()` into the existing numeric JSON projection. It removes
0.69.5's Darwin u16-versus-u32 assignment while retaining the evidence format.
This is source inspection of the published package, not a local downstream patch.

The producer's [0.69.6 optimizer run](https://github.com/dragginzgame/ic-tooling/actions/runs/40688824204)
passes Ubuntu, macOS ARM64 and macOS Intel. Producer general CI was still running
at inspection; no complete producer-matrix pass is claimed. Wire's own new-source
macOS qualification has not run yet. No workflow was dispatched or rerun here.

The historical [Wire 0.7.19 CI](https://github.com/dragginzgame/ic-wire/actions/runs/40683192769)
has Ubuntu success and both macOS failures during installation of the old 0.69.5
release editor, before Wire validation. Preserve its failure artifacts. Once this
adoption is delivered, use the latest delivered Wire source/CI for acceptance;
do not require obsolete 0.7.19 jobs to rerun with different dependencies.

## Focused local checks

Linux evidence is under `target/ic-build-adoption-0696/`:

- Explicit editor preparation reuses its verified 0.69.6 cache; offline
  executable/receipt admission passes through `make release-editor-check`.
- The real release adapter's disposable Git fixture passes source refusal,
  failure retention, metadata/tag/push, resume and receipt checks. No release
  effects occurred in this checkout.
- Strict Clippy passes for the diagnostic example; IC Build 0.69.6 checks nine
  compile-fail cases plus a passing control. The example compiles on Rust 1.91.
- Both standard Wire/Candid roundtrip tests pass with the preserved incoming
  graph. The Testkit admission example passes strict Clippy on that graph.
- Dependency pins, external-IC boundaries, grouped formatting and documentation
  links pass.

No runtime/generator cost changed, so no new performance experiment is applicable
to this host-tool adoption and no speed/Wasm saving is claimed. Full Wire CI,
macOS execution and a fresh full Candid performance matrix were not run locally.

## Delivered Wire issues

[#29](https://github.com/dragginzgame/ic-wire/issues/29) and
[#30](https://github.com/dragginzgame/ic-wire/issues/30) are implemented in published
Wire/derive 0.7.19. Their registry archives identify the release commit above.
The standalone registry consumer under `/tmp/ic-wire-0719-acceptance.pyWIBU/`
passes direct/re-exported Serde crate paths, schema/frame checks, caller-slice
output and generated TypeScript payload encode/decode with trailing-byte refusal.
Issue #30 also has actual IcyDB consumer confirmation. Keep both producer issues
open until current required native qualification is complete; consumer adoption
is separately owned and is not another producer closure gate.

The [SDK recipe](../sdk-integration.md) retains transport, scalar conversions,
buffer/async custody and rollback ownership. There is one Serde/Postcard codec;
payload APIs explicitly omit framing, and framed identities remain intact.
Allocation admission retains its eightfold ledger. The earlier 22-byte example
at `bytes: 64` needs allocation 832 for encode and 704 for decode.

The [caller-storage study](../evidence/caller-storage-0719/README.md) preserves its
frozen 0.68.12/0.6.21 measurements and regressions; it does not qualify this new
dependency graph or an installed SDK. Historical
[runtime](../evidence/runtime-0716/README.md) and
[encoder](../evidence/encoder-followup-0716/README.md) studies retain their inputs.
