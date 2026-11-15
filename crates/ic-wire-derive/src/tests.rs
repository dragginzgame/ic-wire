use super::expand;

#[test]
fn rejects_layout_attributes_and_unqualified_shapes() {
    for input in [
        "#[serde(transparent)] struct Record { value: u8 }",
        "struct Record { #[serde(skip)] value: u8 }",
        "struct Record { #[serde(with = \"custom_bytes\")] value: Vec<u8> }",
        "struct Record { #[serde(with = \"other::serde_packed::bools\")] value: Vec<bool> }",
        "struct Record { #[serde(with = \"ic_wire::serde_packed::unknown\")] value: Vec<bool> }",
        "struct Record { #[serde(with = \"ic_wire::serde_packed::bools\", with = \"serde_bytes\")] value: Vec<bool> }",
        "struct Record { #[serde(with = \"serde_bytes\", with = \"serde_bytes\")] value: Vec<u8> }",
        "enum Choice { #[serde(with = \"serde_bytes\")] Value(u8) }",
        "enum Choice { #[serde(rename(serialize = \"x\", deserialize = \"y\"))] Value(u8) }",
        "struct Record { #[serde(rename = \"x\", rename = \"y\")] value: u8 }",
        "#[serde(rename_all = \"camelCase\")] struct Record { some_value: u8 }",
        "enum Choice { #[serde(borrow)] Value(u8) }",
        "struct Record(#[serde(rename = \"ignored\")] u8);",
        "#[serde(untagged)] enum Choice { Value(u8) }",
        "struct Record<'a> { #[serde(borrow, skip)] value: &'a str }",
        "struct Record<'a> { #[serde(borrow = 1)] value: &'a str }",
        "enum Choice { Value = 7 }",
        "#[wire(unknown)] struct Record;",
        "#[wire(crate = \"one\", crate = \"two\")] struct Record;",
    ] {
        let input = syn::parse_str(input).unwrap();
        assert!(expand(&input).is_err());
    }
}

#[test]
fn supports_ordinary_shapes_and_renamed_dependency() {
    for input in [
        "struct Unit;",
        "struct Newtype(u16);",
        "struct Pair(u8, u8);",
        "struct Empty();",
        "struct Bools { #[serde(with = \"::ic_wire::serde_packed::bools\")] value: Vec<bool> }",
        "#[wire(crate = \"renamed\")] struct Floats(#[serde(with = \"renamed::serde_packed::finite_f32\")] Vec<f32>);",
        "struct Bytes { #[serde(with = \"serde_bytes\")] value: Option<Vec<u8>> }",
        "enum Choice { Pair(u8, u8) }",
        "struct Record { r#type: u8, value: Option<u16> }",
        "enum Choice { Empty, Value(u16), Record { flag: bool } }",
        "#[wire(crate = \"candidate\")] struct Record { value: u8 }",
        "struct Generic<T>(T);",
        "enum Choice { #[serde(rename = \"x\")] Value(u8) }",
        "struct Record<'a> { #[serde(rename = \"text-value\", borrow)] value: &'a str }",
        "struct Generic<'a, T: Clone = u8, const N: usize = 4> where T: Copy { value: T, text: &'a str, bytes: Bytes<N> }",
        "struct Borrowed<'a> { #[serde(borrow = \"'a\")] bytes: BytesRef<'a, 8> }",
    ] {
        assert!(expand(&syn::parse_str(input).unwrap()).is_ok());
    }
}
