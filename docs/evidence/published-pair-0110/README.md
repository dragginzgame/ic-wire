# Published Wire 0.11 / Platform 0.12 consumer qualification

Issue [#34](https://github.com/dragginzgame/ic-wire/issues/34), checked 2026-10-31.
Wire 0.11.0, its derive, Platform 0.12.0 and Metrics 0.22.0 are published registry
packages. No Cargo patches, mirrored types or downstream working-tree edits.

The actual Blob 0.60.0 `wire_schema` example passes Rust 1.91 after changing
only three dependency selections in an isolated archive of released Blob commit
`7fe95703365d51ceda1c0b37d35ca42882ec812d`:

| Dependency | Released Blob selection | Qualified selection |
| --- | --- | --- |
| ic-wire | 0.10.0 | =0.11.0 |
| ic-platform | 0.11.1 | =0.12.0 |
| ic-metrics | 0.21.0 | =0.22.0 |

Metrics must align too: its native metadata implements the supplying Wire trait.
The first check retained Metrics 0.21 and failed its eight metadata trait bounds;
selecting published Metrics 0.22 resolves them without changing Blob Rust source.
The [successful check](blob-check.log) uses the real contracts crate and example,
not a copied DTO surrogate. This does not qualify Blob deployment or its Testkit
integration. [Tooling #274](https://github.com/dragginzgame/ic-tooling/issues/274)
separately owns the typed-client supplier release. Blob's normal checkout remains
unchanged; this proves the available adoption, not that Blob has adopted it.

The [standalone consumer](consumer-Cargo.toml.snapshot) passes all five
[Principal/Account and combined endpoint tests](rust-tests.log),
[Rust 1.91 Wasm compilation](wasm-check.log), and generated TypeScript checks:
exact request/response bytes, length refusals, malformed Account frames and
absent/explicit-zero/nonzero subaccounts. Principal stays bounded to 29 bytes;
Account stays owner then optional 32-byte subaccount. Codec bytes, budgets,
acceptance and retry/hash inputs are unchanged.

The [schema graph](schema-host-graph.txt) has no CDK/macros/runtime guard/Candid;
the [combined Wasm graph](endpoint-wasm-graph.txt) has one Platform runtime guard
and no host serde_json or Candid. Metadata confirms registry sources for all
Wire/Platform suppliers. No installed endpoint or performance experiment was
rerun: published source is the already-qualified 0.11 release, and this checks
consumer composition. Local Linux does not qualify macOS.

Reproduce with cached dependencies, Rust 1.91 native/Wasm and the prepared browser
TypeScript compiler:

```sh
bash docs/evidence/published-pair-0110/reproduce.sh /absolute/fresh/output /absolute/ic-blob-storage
```

The Blob repository is read only: the runner archives the exact released commit
into the new output directory. It checks only the named contracts example, not
sibling suites. Scratch run: `/tmp/wire-issue34-36/`. The earlier failed Metrics
selection remains in `blob-check.log` there; the aligned result is
`blob-aligned.log`. Transitive dependencies resolve from the cached index, so this
is a composition reproduction, not a pinned performance comparison.
