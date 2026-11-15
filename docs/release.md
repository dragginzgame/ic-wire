# Crates.io release

The user authorized public GitHub delivery and crates.io publication of both
product packages at **0.24.0** on 15 November 2026. The repository is now
`dragginzgame/ic-wire`; `ic-wire-legacy` retains the previous implementation
privately. Qualification crates remain unpublished.

The crates.io sparse index checked for this release lists 0.22.4 as the latest
version of both product names; 0.24.0 is available. Direct first-party selections
are current stable non-yanked versions: Platform Types 0.20.4, Platform Runtime
0.22.5, Testkit 0.106.0 and Toko Tooling 0.44.3. Simulator transitive first-party
selections are also current: Platform 0.33.7 and Host Artifacts/FS/Process 0.79.0.
No supplier refresh is needed.

Publication follows the ordered procedure below. Registry state, rather than
these notes or a dry run, establishes whether each upload has completed.

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

The final performance changes remove the representative request-encode
regression. Opt-z response encoding remains seven instructions above reference;
wide integers, float decoding and single-root host costs remain disclosed. See
[performance qualification](design/performance.md). Native timing and Wasm size
are separate evidence; no universal speed improvement is claimed.

## 0.24.0 validation

Final-version Clippy gates, native/host/derive/qualification tests, strict
TypeScript exchange, Rustdoc and Wasm checks passed. Fresh extracted archives
passed the all-feature core tests, literal-frame/platform consumer tests, Wasm
execution and generated-client Chromium smoke. The coordinated registry dry run
verified both final packages and aborted uploads as expected.

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
also passed. That was historical preparation evidence; 0.24.0 requires fresh archive and
registry checks. Ordinary dry runs need registry
access even when all selected dependency sources are already cached.

The browser check compiles the archive-generated TypeScript to ES2022 modules
with strict checking and rewritten `.js` imports, then executes a local page.
It passed in Chrome for Testing 156.0.8078.4 on Linux: literal payload/request/
response bytes, BigInt, UTF-8, typed arrays, selected input spans, owned decode
buffers, field limits and malformed-input refusal. This is one browser-engine
smoke test, not a Safari/Firefox support matrix or a browser speed benchmark.
The headless test disables Chromium's sandbox for this controlled local fixture;
that is not an application/browser deployment recommendation.

## Authorized publication procedure

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

The ordered Cargo commands are:

```sh
cargo publish -p ic-wire-derive --locked
# After the derive version is available:
cargo publish -p ic-wire --all-features --locked --dry-run
cargo publish -p ic-wire --all-features --locked
```

There is no automatic release runner, consumer migration or tag creation in this
preparation. Registry-dependent core verification remains a delivery-time check.
