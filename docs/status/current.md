# Current handoff

## Scope

The maintainer selected a codec-only repository on 2026-10-19. Endpoint SDKs,
simulators, installed canister drivers, browser transport, the private adapter
crate and format-comparison evidence are retired. Native codec controls,
compiler diagnostics and standalone Rust/TypeScript byte conformance remain.
Published runtime APIs, wire bytes, limits and package versions are unchanged.
The pending compatible notes remain 0.4.5; release and publication were not run.

Source starts at `a3e6577` with the pending license-metadata repair preserved.
Shared Tooling remains at reviewed 0.24.2; its generic scripts/catalogs and guides
are intact. Product setup no longer selects its platform executable bundle.
The full pre-change source, retired reports, browser dependencies and previous
build artifacts are preserved outside this checkout at
`/tmp/ic-wire-format-removal.hbmMHv/`.

## Maintained qualification

The Blob adoption hard cut is delivered independently in
[PR #90](https://github.com/dragginzgame/ic-blob-storage/pull/90), commit `2967298`,
from `/tmp/ic-blob-wire-cut`. All owned methods/lifecycle use Wire; installed
client/browser and recovery checks pass against local substitutes. Wire gains 1 MiB finite/native
Rust and generated TypeScript conformance cases, with exact frame equality,
truncation, identity/direction, trailing bytes, noncanonical lengths and allocation
refusal. These focused cases pass; the consumer uses published Wire 0.4.4 without
new runtime functionality. Provider transports and persistent records remain
consumer contracts.

See [local development](../development.md), [evaluation scope](../evaluation.md),
[runtime contract](../runtime.md) and [native timing controls](../performance.md).
The evaluation package stays unpublished. IC Build retains ownership of compiler
diagnostic execution. Private raw Postcard/bitcode/wincode controls retain their
different guarantees; they are not production modes or installed measurements.

Focused Linux checks pass: 22 runtime/native tests and seven doctests, native
raw-codec roundtrips, strict evaluation Clippy and Rust 1.91 host checking,
twelve compiler-diagnostic subjects/control, ninety TypeScript vectors and
17,524 differential mutations, runtime Wasm/floor checks, formatting, pins,
dependency boundaries, snapshot integrity and documentation links. Disposable
Make/release fixtures and common-tool admission pass without platform executables.
All-feature/all-target dependency inspection retains 79 original package
identities and prunes 234 packages. Both offline archives retain the exact MIT
LICENSE; archive preparation used `--no-verify` and uploaded nothing.

The native timing driver runs all 63 case/lane pairs and refuses existing output
without overwriting it. Its final report is `target/native-performance-verified/report.json`;
lock/case/driver hashes match the checked source. Earlier draft failures and
focused logs remain under `/tmp/ic-wire-removal-*`. No full local CI, installed
endpoint, native macOS, compressed size or IC instruction qualification is claimed.

## Preserved contributor work

All packages inherit only the MIT SPDX license. Publishable crates include the
root LICENSE as identical package-local regular files. The formatting hook refuses
tracked symlinks; the local fixture now preserves file types so it exercises that
admission rule. Earlier locked metadata and offline
archive checks passed without duplicate-license warnings; both archives contain
the exact root text. Existing packaging warnings concern excluded tests/examples.

The local Make fixture selects authored crate inputs through Git, including
untracked modules and excluding ignored output. Shared Tooling owns the formatter
and hook implementation. The fixture confines all Git/index/release-routing
effects to disposable repositories.
