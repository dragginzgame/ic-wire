# Detecting public Wire API changes

[IC Wire #3](https://github.com/dragginzgame/ic-wire/issues/3) investigates a development-time contract snapshot. The opt-in `api-manifest` Cargo feature provides `ValidatedRegistry::api_manifests`, which emits one JSON document per public endpoint. A normal Git diff is sufficient to flag drift and locate the affected method. Compatibility decisions still require the application's supported client contracts.

## Inputs and output

Use the application's deployed endpoint name, its existing Wire `Method` descriptor and its `ValidatedRegistry`. The deployed name and the generated client export can differ. The descriptor already supplies the routing ID, request and response roots, and separate client payload limits. Store the full u64 ID as a decimal string. Byte limits exclude the frame header. Node and depth limits describe browser traversal. Server-side allocation, authorization and application behavior are outside this snapshot.

The [example document](../../qualification/ic-wire-compatibility/tests/manifest_cases/system_get_user_authority.json) is a synthetic authority lookup. Its reply contains an `authority` field referring to the `Authority` enum. Reordering the derived Rust variants changes these lines in that endpoint's file:

```diff
-      {"name":"User","id":0,"value":{"k":"unit"}},
-      {"name":"Admin","id":1,"value":{"k":"unit"}}]},
+      {"name":"Admin","id":0,"value":{"k":"unit"}},
+      {"name":"User","id":1,"value":{"k":"unit"}}]},
```

The routing ID stays the same. Swapping two fields moves their entries in the record's ordered `fields` array. Adding or removing an endpoint adds or deletes its document. Changing a shared type updates each affected endpoint's file. Unrelated registry roots produce no changes.

The exporter writes directly from the admitted graph with the same metadata renderer as `metadata_json()`. It visits each reachable definition once per endpoint, choosing the lexically first pending name. It preserves field and variant order and emits a trailing newline. The registry’s `output_bytes` limits the combined document bytes, including whitespace. An empty inventory returns an empty map. Explicit public root names make reviews easier. Nested names synthesized by `SchemaRegistry` depend on compiler type spelling, so a module rename or compiler update can also produce a diff. Pin the generator and toolchain when comparing releases. The document's `format` identifies the exporter format independently of the crate and protocol versions. A future grammar change should update that marker and regenerate baselines in a dedicated review.

The [standalone upload consumer](../../qualification/ic-wire-consumer/README.md) exercises this workflow with the method descriptor used by its generated TypeScript client. Run `make check-api-manifest` for its baseline diff.

## CI in a consuming application

1. Have the endpoint declaration or mount generator provide the actual exported endpoint name and the typed descriptor used for client generation. Collect the descriptors for the selected canister, role and production feature set. For a mixed-protocol service, the same declarations must identify which exports use Wire.
2. Independently enumerate the final Wasm module's public IC functions (exports named `canister_query …`, `canister_composite_query …` or `canister_update …`). Compare that inventory with the declarations, then check that each Wire declaration produced one document. Account for non-Wire methods from their owning declarations. Fail on missing or unexpected methods. Wire cannot infer a DTO or a protocol from an exported Wasm function name. An application supplying only a hand-built slice of descriptors can detect changes within that slice but cannot prove coverage.
3. Seed the checked-in baseline from the supported release or published client bindings and verify independent literal bytes. In each PR, export into a fresh directory and compare the entire directory, including added and removed files. Use an injective filename encoding for arbitrary endpoint names, or enforce safe single-component identifiers. The application supplies its own export command. The comparison itself can use `git diff --no-index --exit-code -- api/baseline "$generated_dir"`.
4. Review the method, direction and changed schema or limits. A reordered enum can silently invert meaning, and a smaller reply allowance can reject previously valid data. Exercise the old client against the candidate and test custom Serde or public/storage conversions against literal bytes. Accept intentional changes explicitly. Where old clients must continue to work, retain their contract through rollout.

## Recommendation and qualification

Start with the per-method Git diff. Defer a semantic diff engine until there are real changes reviewers cannot interpret. The consumer receives a map of endpoint names to JSON strings and owns safe filenames and atomic directory replacement. Test coverage against a real application's release artifact before promising automatic endpoint discovery.

The private tests cover registration and descriptor order, unchanged output against the checked-in fixture, recursive references, added/removed methods, limits, field order and the nested tag reversal. They also check both authority encodings against independent literal bytes. Run from the repository root:

```sh
cargo test --manifest-path qualification/Cargo.toml --locked -p ic-wire-compatibility \
  --features typescript --test typescript manifest_cases
```

## Implementation tracking

[IC Wire #4](https://github.com/dragginzgame/ic-wire/issues/4) owns the exporter.
[IC Tooling #16](https://github.com/dragginzgame/ic-tooling/issues/16) owns the protocol-independent artifact inventory, including method kind.
[Toko Tooling #401](https://github.com/dragginzgame/toko-tooling/issues/401) tracks a shared CI command after two consumer trials.
