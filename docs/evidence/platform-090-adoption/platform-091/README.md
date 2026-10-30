# Platform 0.9.1 follow-up

Adopt published Platform/macros 0.9.1, reviewed against upstream release commit
`8e21a5e` and the registry package sources. The incoming lockfile already selected
0.9.1; the workspace requirement now selects that patch explicitly. Retain the
pending Wire 0.8.0 notes and all earlier work. No Wire package version is changed.

This fixes Cargo dependency admission: Platform's executor singleton `links`
identity moves into an empty, optional `ic-platform-runtime-guard =0.9.1` package.
Native consumers without executor/runtime no longer claim an executor owner.
The actual endpoint graph still selects the guard, with no external executor or
IC0 supplier; the core codec graph still excludes Platform. Testkit keeps its
separate native-only Platform 0.8.3 selection. No new codec, runtime implementation,
Wire bytes, schemas, admission limits or acceptance rules are introduced.

[Registry source parity](source-parity.json) verifies all 119 facade source files
and all five macro source files are byte-identical to the qualified 0.9.0 packages.
The [upstream packaging diff](upstream.patch) records the guard and manifest change.
The [locked dependency graph](Cargo.lock) preserves unrelated selections.

Matched endpoint artifact builds use the previous study's Rust 1.99.1, wasm32,
LTO, one codegen unit, panic abort and stripped opt-z / opt-3 profiles:

| Profile | 0.9.0 total / code bytes | 0.9.1 total / code bytes | Size change |
| --- | ---: | ---: | ---: |
| opt-z | 99,939 / 86,068 | 99,939 / 86,068 | 0 / 0 |
| opt-3 | 93,510 / 80,630 | 93,510 / 80,630 | 0 / 0 |

[Artifact hashes and section comparisons](artifacts.json) retain exact results.
The files are not byte-identical: opt-z differs in its data section; opt-3 differs
in its function, element, code and data sections. Equal size is not an assertion
of identical Wasm or newly measured instruction parity. No speed claim is made.
Runtime/generator algorithm changes are absent, so a new timing/IC benchmark is
not applicable to this packaging-only patch; the earlier physical measurements
remain explicitly scoped to Platform 0.9.0.

Focused validation passes: locked offline metadata, declaration boundaries, core
and endpoint graphs, core/derive and endpoint strict Clippy, native benchmark host
Clippy, endpoint schema tests, both Wasm builds and Rust 1.91 Wasm checking.
Full CI, installed IC measurements and macOS were not rerun. No sibling edits,
commits, release or publication were performed.

To reproduce, use the retained lock and compile the endpoint fixture with
`cargo build --locked --offline -p ic-wire-endpoint-fixture --lib --target wasm32-unknown-unknown --profile evaluation`
and then `--profile benchmark-speed`, in a separate Cargo target from the
[0.9.0 baseline](../README.md). Compare total bytes and the Wasm code section;
source parity lists each registry source file's SHA-256.
