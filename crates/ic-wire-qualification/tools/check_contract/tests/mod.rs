use super::*;
use serde_json::json;

fn snapshot() -> Snapshot {
    serde_json::from_value(json!({"types":{"Request":{"k":"record","fields":[
        {"name":"a","value":{"k":"u","bits":8}}, {"name":"b","value":{"k":"i","bits":8}}
    ]}}, "methods":[{"name":"read","mode":"query","id":"7","request":{"k":"ref","name":"Request"},"response":{"k":"unit"},
    "requestBytes":64,"responseBytes":64,"requestNodes":64,"responseNodes":64,"requestDepth":8,"responseDepth":8}],
    "vectors":{"request":[73,87,7,0,0,0,0,0,0,0,0,255,1]}})).unwrap()
}
#[test]
fn unchanged_and_reordered_declarations_are_compared_by_name() {
    let mut before = snapshot();
    let mut second = before.methods[0].clone();
    second["name"] = json!("write");
    second["id"] = json!("8");
    before.methods.push(second);
    let mut after = snapshot();
    after.methods = before.methods.iter().rev().cloned().collect();
    before.validate().unwrap();
    after.validate().unwrap();
    assert!(changes(&before, &after).is_empty());
}
#[test]
fn same_id_layout_changes_limits_and_original_bytes_require_review() {
    let before = snapshot();
    let mut after = snapshot();
    after.types.get_mut("Request").unwrap()["fields"]
        .as_array_mut()
        .unwrap()
        .reverse();
    after.validate().unwrap();
    assert_eq!(changes(&before, &after), ["types/Request"]);
    after = snapshot();
    after.methods[0]["requestNodes"] = json!(63);
    assert_eq!(changes(&before, &after), ["methods/read"]);
    after = snapshot();
    after.vectors.get_mut("request").unwrap().push(0);
    assert_eq!(changes(&before, &after), ["vectors/request"]);
    after = snapshot();
    after.methods.clear();
    assert_eq!(changes(&before, &after), ["methods/read"]);
}
#[test]
fn duplicate_ids_bad_references_and_unknown_top_level_keys_refuse() {
    let mut input = snapshot();
    let mut method = input.methods[0].clone();
    method["name"] = json!("other");
    input.methods.push(method);
    assert_eq!(input.validate(), Err(typescript::GenerationError::Schema));
    input = snapshot();
    input.types.clear();
    assert_eq!(input.validate(), Err(typescript::GenerationError::Schema));
    assert!(serde_json::from_value::<Snapshot>(json!({"method":[]})).is_err());
}
