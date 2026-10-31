# Build/Host 0.73.2, Testkit 0.75.1 and Metrics 0.24.1

2026-10-31. Adopt the published suppliers in [resolved selections](selections.json)
and align the managed Build CLI editor at 0.73.2. The incoming Cargo.lock already
selected these releases and is preserved byte-for-byte. Root requirements now
require Build 0.73.2 / Testkit 0.75.1; Testkit supplies arithmetic-only Metrics
0.24.1. No unused Host/Host Tools dependency is added: Wire consumes the Host
foundation through its actual Build/Testkit owners.

Own versions stay Core 0.13.1 / Endpoint 0.2.1; only Core pending notes select
compatible 0.13.2. Endpoint's production package has no dependency change requiring
another release. No staging, commits, publication, release or sibling checkout
edits occurred. Shared and Platform/Types selections remain unchanged.

## Supplier review and graph

[Source inventory](source-review.json) compares released registry sources. Build,
Artifacts, FS and Testkit src files are identical. Process changes one test;
Metrics changes application documentation relative to 0.23.2, and its src is
identical to 0.24.0. New package/requirement metadata does not change the selected
runtime/generator implementation. The upstream
[Foundation notes](https://github.com/dragginzgame/ic-tooling/blob/a930be3305b8dcd9477ac045308f1072e4e3142b/releases/foundation/CHANGELOG.md)
and [integration notes](https://github.com/dragginzgame/ic-tooling/blob/7898f58fc0ee0a909484c192a81cf454683120b4/releases/integration/CHANGELOG.md)
distinguish the catalog/test update from Metrics dependency alignment.

[Preservation proof](preservation.json) checks the incoming lock and own versions;
the [production endpoint Wasm graph](normalized-endpoint-wasm-graph.txt) equals
the delivered source after path normalization. No production dependency edge or
protocol change is introduced. Codec/schema bytes, budgets, malformed acceptance,
retry/hash inputs and endpoint rollback remain unchanged. No new runtime cost,
Wasm-size or build-time saving is claimed; a new performance benchmark does not
apply to this host-tool/metadata selection change.

## Focused qualification

[Logs](checks/) retain the actual callers:

- Build's diagnostic runner verifies nine expected diagnostics plus its passing
  control. Ordinary qualification passes four tests, strict Clippy and Rust 1.91;
  its fuzz/profiler default/native/Wasm graph guard also passes.
- Default Wire/Candid corpus roundtrip/refusal tests pass (three tests).
  Benchmark targets with wire,candid,ic check on Rust 1.91.
- Installed endpoint limits/malformed refusals and transaction/reply rollback
  pass with Testkit 0.75.1 and the already prepared PocketIC server. No sibling
  or broad simulator suite is run.
- The managed 0.73.2 CLI is prepared and admitted offline. Real group-manifest
  preparation/checks operate on isolated Core/Endpoint copies, updating all
  selected member versions and root aliases, including unused/private entries,
  while preserving the other group. These checks perform no Git/release effects.
- A registry-only Rust 1.91 consumer composes all four native Metrics 0.24.1
  schemas, Principal/Account, Wire 0.13.1, Endpoint 0.2.1 and Testkit 0.75.1 typed
  Limits/Error plus query/update signatures. Exactly one Metrics/Core/Types
  package resolves; no patch or source-aligned supplier is used. Unlike the older
  0.75.0 pairing, native Metrics and Testkit now share Metrics 0.24.1.
- Formatting, pins/boundaries, documentation links and runner syntax pass.

Execution host is Linux x86_64, Rust 1.99.1 / LLVM 23.1.1; minimum checks use
1.91.0. Installed execution explicitly selects the prepared
`/home/adam/.cache/canic/pocket-ic-server-16.0.0-pocket-ic-x86_64-linux/pocket-ic`.
No fresh full local CI, new hosted/macOS matrix or Candid performance matrix is
claimed. Older release/benchmark evidence retains its original selections.

## Reproduction

With the locked registry sources/compilers and selected tools already prepared:

```sh
bash docs/evidence/tooling0732-metrics0241/qualify-consumer.sh /tmp/wire-tooling0732-consumer
bash docs/evidence/tooling0732-metrics0241/qualify-editor.sh /tmp/wire-tooling0732-editor
make release-editor-check check-typed-diagnostics check-qualification
cargo test --locked --offline -p ic-wire-benchmarks --features wire,candid --test roundtrip
cargo +1.91.0 check --locked --offline -p ic-wire-benchmarks --all-targets --features wire,candid,ic
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic make check-endpoint-ic
make fmt-check check-pins check-boundary check-doc-links
```

The editor runner freezes baseline revision plus supplier-only inputs; run it with
the adopted source's managed Build 0.73.2 editor selected. Both scripts require a
fresh output directory and install nothing. Full temporary outputs remain under
target/adoptions/tooling-metrics-latest; essential evidence is retained here so
normal explicit Cargo cleanup can remove those target outputs.
