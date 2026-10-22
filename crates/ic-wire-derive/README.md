# IC Wire metadata derive

`Wire` describes ordinary Serde layouts for IC Wire’s optional host-side
TypeScript generator. It generates no serializers and requires no field IDs.
Use the re-export from `ic-wire` alongside ordinary `Serialize` / `Deserialize`.
Concrete generic instantiations follow Serde declaration order. Unsupported Serde
attributes fail explicitly; custom serializers require verified manual metadata.
Field-level `#[serde(borrow)]` and `#[serde(borrow = "'a")]` are supported:
they constrain deserialization lifetimes without changing the serialized layout.

This replaces the former serializer-generating `Wire` derive; the new derive
requires ordinary Serde serialization and generates only metadata. See the
[runtime guide](https://github.com/dragginzgame/ic-wire/blob/main/crates/ic-wire/README.md)
for migration, host feature gating, bounds and consumer responsibilities.

Licensed under the MIT License; see the packaged LICENSE.
