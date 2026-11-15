//! Generated external consumer contract, using its public application API.
use super::run_typescript;
use ic_wire_consumer::{
    Receipt, clients, decode_receipt, decode_upload, encode_receipt, encode_upload, sample_upload,
};
use serde_json::json;
#[test]
fn standalone_consumer_exchanges_generated_clients() {
    let mut paths = Vec::new();
    let (module, frame) = clients("./runtime-adapter.ts", |report| {
        eprint!("{report}");
        paths.extend(report.warnings.into_iter().map(|warning| warning.path));
    })
    .unwrap();
    assert_eq!(
        paths,
        [
            "/types/Upload/fields/tags/list/u16",
            "/methods/upload/request/fields/tags/list/u16"
        ]
    );
    let request = sample_upload().unwrap();
    let reply = Receipt {
        revision: 129,
        accepted: true,
    };
    let output = run_typescript(
        &json!({"module":module,"frameModule":frame,
        "request":encode_upload(&request).unwrap(),"response":encode_receipt(&reply).unwrap()}),
        include_str!("consumer_exchange.mjs"),
    );
    let [request_bytes, reply_bytes]: [Vec<u8>; 2] = serde_json::from_value(output).unwrap();
    assert_eq!(decode_upload(&request_bytes).unwrap(), request);
    assert_eq!(decode_receipt(&reply_bytes).unwrap(), reply);
}
