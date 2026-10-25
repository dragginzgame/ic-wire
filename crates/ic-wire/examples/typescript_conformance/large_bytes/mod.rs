//! Large-buffer boundary fixtures for the production Serde codec.
use ic_wire::{Bytes, Limits, native};
use std::{fs, path::Path};
pub(super) fn generate(directory: &Path) -> String {
    let limits = Limits {
        bytes: 1024 * 1024 + 21,
        allocation: 16 * 1024 * 1024,
        nodes: 32,
        depth: 8,
    };
    let id = 0x6c61_7267_6500_0001;
    let value = Bytes::<1048576>::new((0..1024 * 1024).map(|i| (i % 251) as u8).collect()).unwrap();
    let schema = ic_wire::typescript::native_schema::<Bytes<1048576>>().unwrap();
    let method = serde_json::json!({"name":"codec","mode":"query","id":id.to_string(),"request":schema,"response":schema,"requestBytes":limits.bytes,"responseBytes":limits.bytes,"requestNodes":limits.nodes,"responseNodes":limits.nodes,"requestDepth":limits.depth,"responseDepth":limits.depth});
    fs::write(
        directory.join("large.ts"),
        ic_wire::typescript::generate_registry(
            &Default::default(),
            &[method],
            "./wire-runtime.js",
            &Default::default(),
            |report| eprint!("{report}"),
        )
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("large-request.bin"),
        native::encode_request(id, &value, limits).unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("large-response.bin"),
        native::encode_response(id, &value, limits).unwrap(),
    )
    .unwrap();
    include_str!("checks.mjs").to_owned()
}
