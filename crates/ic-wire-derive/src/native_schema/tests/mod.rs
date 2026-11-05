#[test]
fn rejects_unsupported_attributes_and_shapes() {
    for source in [
        "#[serde(untagged)] enum E { A(u8) }",
        "#[serde(tag = \"kind\")] enum E { A }",
        "#[serde(rename_all = \"snake_case\")] struct A { a: u8 }",
        r#"struct A { #[serde(rename(serialize = "a", deserialize = "b"))] a: u8 }"#,
        r#"struct A { #[serde(alias = "b")] a: u8 }"#,
        r#"struct A { #[serde(rename = "b")] a: u8, b: u8 }"#,
        r#"enum A { #[serde(rename = "B")] A, B }"#,
        r#"enum A { #[serde(rename = "a", rename = "b")] A }"#,
        "struct A { #[serde(skip)] a: u8 }",
        "struct A { #[serde(flatten)] a: u8 }",
        "struct A { #[serde(with = \"custom\")] a: u8 }",
        "struct A { #[serde(serialize_with = \"custom\")] a: u8 }",
        "struct A { #[serde(deny_unknown_fields)] a: u8 }",
        "#[serde(deny_unknown_fields, deny_unknown_fields)] struct A { a: u8 }",
        "#[serde(crate = 1)] struct A { a: u8 }",
        "#[serde(crate = \"serde\", crate = \"serde\")] struct A { a: u8 }",
        "struct A { #[serde(crate = \"serde\")] a: u8 }",
        "enum A { #[serde(crate = \"serde\")] B }",
        "struct A { #[serde(with = \"serde_bytes\", with = \"serde_bytes\")] a: Vec<u8> }",
        "struct A { #[serde(with = \"serde_bytes\", deserialize_with = \"custom\")] a: Vec<u8> }",
        "#[serde(borrow)] struct A { a: u8 }",
        "enum A { #[serde(borrow)] B }",
        "enum A { #[serde(borrow)] B(u8, u8) }",
        "enum A { #[serde(borrow)] B { a: u8 } }",
        "struct A { #[serde(borrow, borrow)] a: u8 }",
        "struct A { #[serde(borrow = 1)] a: u8 }",
        "struct A { #[serde(borrow, with = \"custom\")] a: u8 }",
        "struct A { #[wire(id = 0)] a: u8 }",
        "#[serde(transparent)] struct A(u8, u8);",
        "enum E {}",
    ] {
        assert!(
            super::expand(syn::parse_str(source).unwrap()).is_err(),
            "{source}"
        );
    }
}

#[test]
fn crate_paths_emit_the_same_metadata_implementation() {
    for source in [
        "struct A { value: u32 }",
        "struct A;",
        "struct A(u16);",
        "enum A<T> { Empty, Value(T) }",
    ] {
        let plain = super::expand(syn::parse_str(source).unwrap()).unwrap();
        for path in ["serde", "::owner::reexport::serde"] {
            let attributed = format!("#[serde(crate = {path:?})] {source}");
            let projected = super::expand(syn::parse_str(&attributed).unwrap()).unwrap();
            assert_eq!(plain.to_string(), projected.to_string());
        }
    }
}
