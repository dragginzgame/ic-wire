# Crates.io preparation

The next intended public release is **0.24.0**, selected by the user while
performance investigation continues. Manifests now select the user-requested
0.23.9 GitHub evaluation patch; historical archive
evidence describes 0.23.8. At release preparation, update
both product versions and their dependency requirements together, check registry
availability again, and repeat archive verification for the final inputs.
The future release target does not trigger publication during this investigation.
The subsequent packed-word performance fix has focused native/Wasm/IC evidence;
the archive checks below predate that fix and must be refreshed for 0.24.0.

Publication is explicitly **not authorized** for this preparation. Nothing here
uploads packages automatically. The product roster is `ic-wire-derive` and
`ic-wire`, version 0.23.9. Qualification crates remain private. The crates.io
sparse index checked on 15 November 2026 lists 0.22.4 as the latest version of
both names, leaving 0.23.8 available. Both owner lookups return `borovan`.
A successful dry run does not establish that a credential has upload permission.

## Prepared product

Both manifests select crates.io and contain package README and repository metadata.
Archives carry the complete licence. The derive package's local licence matches
the repository licence; packaging copies the root licence into the core archive.
The bundled TypeScript runtime is included by `include_str!`, and the archive
consumer verifies it is available when generating clients. No npm runtime is
published or required. Rust 1.100.0 remains the supported compiler floor.

The [package guide](../crates/ic-wire/README.md) documents features, output custody,
limits, TypeScript delivery and the breaking move from `native` entrypoints to
root-level operations with `Encoding`. The derive is separately enabled with
`derive`, and its re-export lives under `ic_wire::typescript`. There is one current
schema/runtime contract; regenerate clients together with the runtime.
The pre-publication hard cut also replaces `PackedI16<N>` with `Packed<i16, N>`.
All packed constructors return Result, and the i16-only metadata kind is removed.
Legacy runtimes do not enforce the new generic packed bounds.
After that consolidation, the refreshed archive tests, Chromium smoke and
coordinated non-publishing Cargo dry run pass. The focused TypeScript suite now
has 24 tests, including independent generic-bound cases beyond legacy coverage.

The user chose simplicity over speculative specialization. The measured request
encode regressions (+5.30%/+9.46% at opt-z/opt-3), opt-3 response encode regression
(+0.87%) and single-root host cost are disclosed tradeoffs for this preparation.
They do not become speed improvements or disappear because size is smaller.
See [performance qualification](design/performance.md). Extra domain-adapter
benchmarks and a scratch/fallback helper are not first-publication requirements.

## Preparation checks

```sh
make check-packages
# Use the client directory printed by that command and an explicitly prepared
# Chromium binary. Install the private TypeScript compiler via npm ci in
# qualification/typescript if it is not already prepared.
make check-browser CLIENT_DIR=/absolute/path/to/run/client BROWSER=/absolute/path/to/chrome
```

`check-packages` assembles both crates, extracts fresh copies under `target/`,
checks README/licence inclusion, and tests the core archive with all features.
It copies the real qualification consumer into that scratch directory, uses the
extracted core and derive, and tests literal frames, metadata generation and
published Platform Types identities. It builds and executes the codec-only Wasm
consumer and checks all-feature forwarding on Wasm. Product source files are not
rewritten: only an empty scratch `[workspace]` and a local derive patch isolate
the archives before their coordinated registry publication. This is archive
proof, not proof that crates.io already serves these packages.

Cargo 1.100.0's offline workspace package verifier encountered an internal
`no hash listed for ic-wire-derive v0.23.8` error in its temporary registry.
The explicit archive checks avoid that offline verifier path. Subsequently,
`cargo publish --workspace --dry-run --all-features --allow-dirty` with registry
access successfully packaged and verified both crates using Cargo's temporary
registry, then explicitly aborted both uploads. The standalone derive dry run
also passed. No real publish command has run. Ordinary dry runs need registry
access even when all selected dependency sources are already cached.

The browser check compiles the archive-generated TypeScript to ES2022 modules
with strict checking and rewritten `.js` imports, then executes a local page.
It passed in Chrome for Testing 156.0.8078.4 on Linux: literal payload/request/
response bytes, BigInt, UTF-8, typed arrays, selected input spans, owned decode
buffers, field limits and malformed-input refusal. This is one browser-engine
smoke test, not a Safari/Firefox support matrix or a browser speed benchmark.
The headless test disables Chromium's sandbox for this controlled local fixture;
that is not an application/browser deployment recommendation.

## Later publication, only when requested

1. Freeze the intended source and review the final package lists, versions and
   dependency selection. Commit/push the prepared changes when authorized.
   Recheck registry availability and owner credentials at delivery time.
2. Reuse unchanged focused runtime evidence; re-run affected checks if package
   contents or dependencies changed. Keep final archive verification fresh.
3. Publish `ic-wire-derive` first. Wait until the selected release version is visible in the registry
   index, then verify the core archive against that actual registry dependency.
4. Publish `ic-wire`, then confirm both versions and build a fresh consumer using
   registry dependencies, without path patches. A partial publication must be
   resumed from observed registry state; never blindly repeat or change a version
   that has already been uploaded.

The ordered Cargo commands, for a separately authorized publication, are:

```sh
cargo publish -p ic-wire-derive --locked
# After the derive version is available:
cargo publish -p ic-wire --all-features --locked --dry-run
cargo publish -p ic-wire --all-features --locked
```

There is no automatic release runner, consumer migration or tag creation in this
preparation. Registry-dependent core verification and actual upload authority
remain delivery-time checks, not a reason to publish early during preparation.
