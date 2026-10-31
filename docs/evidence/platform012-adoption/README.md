# Platform 0.12 integration qualification

2026-10-31; isolated Wire contribution from release
`c9d4e7b45b9ba6d841d7e2a664877e3bea035a5b` (0.10.0).
Only the optional Platform requirement, its three locked packages and migration
documentation change. Own package versions stay 0.10.0; notes target 0.11.0.

Published Wire 0.10 plus direct Platform 0.12 fails E0277 for native Principal
and Account schema bounds, because Wire implements the trait for Platform 0.11
identities. The selected Platform 0.12 supplier fixes those bounds without new
schema implementations or changes to generated shapes, frames or acceptance.
[#34 — Supply platform-types schemas compatible with published Platform 0.12](https://github.com/dragginzgame/ic-wire/issues/34)
tracks publication. Consumers align direct Platform to 0.12 and public Wire
types to the supplying minor. [Tooling #274 — Align Testkit typed Wire calls](https://github.com/dragginzgame/ic-tooling/issues/274)
tracks the corresponding typed native client release.

## Reproduction and results

Prepare the documented Rust 1.99.1 development compiler, Rust 1.91.0 minimum,
their Wasm targets, Node 24.21.0 and TypeScript 7.0.2. Tests admit prepared
inputs offline. Run from this topic checkout:

```sh
bash scripts/ci/check-platform-types.sh /absolute/prepared/typescript/bin/tsc
bash scripts/ci/check-endpoint.sh
make check-boundary check-pins check-doc-links
git diff --check
```

The standalone consumer uses the checkout's Wire source and exact registry
Platform 0.12.0, without a patched supplier. Results:

- Schema-only and combined endpoint consumers pass Rust 1.91 host/Wasm checks
  and strict Clippy. Four schema-only and five combined tests pass.
- Principal Rust/TypeScript frames and length refusals match. Account bytes
  preserve absent, zero and nonzero subaccounts; malformed request/response
  refusals agree. For anonymous owner, absent payload `01 04 00` remains distinct
  from `01 04 01` followed by 32 zero bytes.
- Schema-only graphs contain no runtime or Candid; core-only graphs contain no
  Platform. Combined endpoint graphs select the matching macro/runtime guard;
  Wasm schema selection contains no host serde_json.
- Endpoint schema tests (3), automatic method identity control (1), 11 expected
  compiler diagnostics and passing control succeed. Endpoint/Wire Clippy,
  optimized Wasm build, Rust 1.91 Wasm check and dependency graphs pass.
- Dependency declarations/boundaries, changed-document links and diff checks pass.

Run artifacts are retained in Auth's
`target/platform-adoption-20261031.306Irl/`: `published-consumer.log`,
`platform-types.log`, `endpoint.log` and the isolated `upstream/ic-wire/` checkout.
The schema script owns a disposable target; the endpoint run inherited Auth's
repository-local Cargo target directory. No full CI, macOS, installed PocketIC
execution or performance study was run for this topic.

No runtime or generator cost change is claimed. Recursive byte comparison of
the registry packages' `src/` trees shows Platform, macros and runtime guard
0.12.0 identical to 0.11.0; Wire Rust source is unchanged. The new package
identity and optional gzip dependency are the supplier changes. Existing cost
studies remain evidence for their own selections, not new measurements here.
