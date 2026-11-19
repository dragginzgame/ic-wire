# IC Wire derive

Host-only schema projection for IC Wire. Prefer the coordinated re-export:

```toml
[dependencies]
ic-wire = { version = "0.25.2", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
```

```rust
#[derive(serde::Serialize, serde::Deserialize, ic_wire::typescript::Wire)]
struct Request { sequence: u64 }
```

The macro describes supported Serde layouts for the validated host graph; Serde
remains the only Rust serialization implementation. Unsupported attributes fail
explicitly. Named/unit/newtype/tuple structs, ordinary enum variants, generics,
borrowed fields, symmetric field/variant renames, standard `serde_bytes` and
qualified IC Wire packed adapters are supported. Renamed dependencies use
`#[wire(crate = "dependency_name")]`.

The same derive works on native and Wasm DTOs. It checks the declaration on
both targets and emits the schema implementation only for native builds.
Requires Rust 1.100.0. The derive and runtime crate are released together.
See [IC Wire](https://github.com/dragginzgame/ic-wire) for usage and qualification.
Licensed under the included Toko Studios Network Licence (`LICENSE.md`).
