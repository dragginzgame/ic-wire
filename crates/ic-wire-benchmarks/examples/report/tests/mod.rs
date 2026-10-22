use super::*;
use serde_json::json;

#[test]
fn code_size_counts_only_the_code_section_and_rejects_broken_framing() {
    let header = b"\0asm\x01\0\0\0";
    assert_eq!(code_size(header), Ok(0));
    let module = [header.as_slice(), &[0, 2, 1, b'x', 10, 1, 0]].concat();
    assert_eq!(code_size(&module), Ok(1));
    let padded = [header.as_slice(), &[10, 0x81, 0, 0]].concat();
    assert_eq!(code_size(&padded), Ok(1));
    for tail in [
        &[10][..],
        &[10, 2, 0],
        &[10, 0x80],
        &[10, 0xff, 0xff, 0xff, 0xff, 0x10],
        &[10, 0xff, 0xff, 0xff, 0xff, 0x80],
        &[10, 1, 0, 10, 1, 0],
    ] {
        assert!(code_size(&[header.as_slice(), tail].concat()).is_err());
    }
    assert!(code_size(b"invalid").is_err());
}

#[test]
fn artifact_uses_reported_paths_and_rejects_missing_or_ambiguous_outputs() {
    let message = json!({"reason":"compiler-artifact", "target":{"name":"codec-wasm"},
        "filenames":["/custom target/codec_wasm.d", "/custom target/codec_wasm.wasm"]})
    .to_string();
    assert_eq!(
        artifact(&message).unwrap(),
        PathBuf::from("/custom target/codec_wasm.wasm")
    );
    assert!(artifact("").is_err());
    assert!(artifact(&format!("{message}\n{message}")).is_err());
    assert!(artifact(&message.replace("codec-wasm", "another-target")).is_err());
}

#[test]
fn comparison_requires_pairs_and_rejects_duplicates_but_allows_wire_reuse() {
    let root = std::env::temp_dir().join(format!("ic-wire-report-test-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    assert!(comparison(&root, "wire-native").is_err());
    let run = root.join("evaluation-native-1");
    fs::create_dir(&run).unwrap();
    let file = run.join("samples.jsonl");
    fs::write(&file, "").unwrap();
    assert!(comparison(&root, "wire-native").is_err());
    let row = json!({"case":"unit", "operation":"encode-fresh", "codec":"wire-native",
        "input_sha256":"same-input", "encoded_bytes":11, "measurement":{"median_ns":20}});
    let wire = row.to_string();
    let mut row = row;
    row["codec"] = json!("candid");
    let candid = row.to_string();
    row["codec"] = json!("wire-native");
    row["operation"] = json!("encode-reuse");
    let reuse = row.to_string();
    for incomplete in [&wire, &candid, &reuse] {
        fs::write(&file, incomplete).unwrap();
        assert!(comparison(&root, "wire-native").is_err());
        assert!(!root.join("comparison.csv").exists());
    }
    let complete = format!("{wire}\n{candid}\n{reuse}\n");
    fs::write(&file, &complete).unwrap();
    assert_eq!(
        String::from_utf8(comparison(&root, "wire-native").unwrap())
            .unwrap()
            .lines()
            .count(),
        2
    );
    fs::write(&file, format!("{complete}{wire}\n")).unwrap();
    assert!(comparison(&root, "wire-native").is_err());
    let finite = complete.replace("wire-native", "wire-bounded");
    fs::write(&file, finite).unwrap();
    assert!(comparison(&root, "wire-native").is_err());
    assert!(comparison(&root, "wire-bounded").is_ok());
    assert!(comparison(&root, "unknown").is_err());
    fs::write(&file, &candid).unwrap();
    assert!(comparison(&root, "wire-bounded").is_err());
    fs::remove_dir_all(root).unwrap();
}
