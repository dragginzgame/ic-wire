//! Generate real Rust bindings/vectors and serve a local installed browser fixture.
use candid::Principal;
use ic_wire_canister_evaluation::{Blob, CONTRACT, MAX_BLOB, MAX_WIRE, Message};
use pocket_ic::PocketIcBuilder;
use postcard_bindgen::{
    PackageInfo, generate_bindings,
    javascript::{GenerationSettings, build_package},
};
use serde_json::json;
use std::{env, fs, io, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::args_os().nth(1).expect("browser evaluation directory"));
    let generated = root.join("generated");
    fs::create_dir_all(&generated).unwrap();
    build_package(
        &generated,
        PackageInfo {
            name: "wire-bindings".into(),
            version: "0.1.1".try_into().unwrap(),
        },
        GenerationSettings::enable_all(),
        generate_bindings!(Blob, Message),
    )
    .unwrap();
    let manifest_path = generated.join("wire-bindings/package.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["private"] = json!(true);
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    fs::write(
        generated.join("contract.json"),
        serde_json::to_vec_pretty(
            &json!({"id": CONTRACT, "maxWire": MAX_WIRE, "maxBlob": MAX_BLOB}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut vectors = Vec::new();
    for (account, revision, size) in [
        (7, 42, 0),
        (u64::MAX, u64::MAX, 4096),
        (9_007_199_254_740_993, 127, 128),
    ] {
        let payload: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
        let request = Message {
            contract: CONTRACT,
            direction: 0,
            account,
            revision,
            payload: Blob::new(&payload).unwrap(),
        };
        let mut response = request.clone();
        response.direction = 1;
        response.revision = response.revision.wrapping_add(1);
        vectors.push(json!({"account":account.to_string(),"revision":revision.to_string(),"payload":payload,"request":postcard::to_allocvec(&request).unwrap(),"response":postcard::to_allocvec(&response).unwrap()}));
    }
    fs::write(
        generated.join("vectors.json"),
        serde_json::to_vec_pretty(&vectors).unwrap(),
    )
    .unwrap();
    if env::args().any(|arg| arg == "--generate-only") {
        return;
    }
    let server = PathBuf::from(env::var_os("POCKET_IC_BIN").expect("prepared PocketIC binary"));
    let wasm =
        fs::read(env::var_os("IC_WIRE_BROWSER_WASM").expect("Postcard fixture Wasm path")).unwrap();
    let mut pic = PocketIcBuilder::new()
        .with_server_binary(server)
        .with_application_subnet()
        .build();
    let id = pic.create_canister();
    pic.add_cycles(id, 10_000_000_000_000);
    pic.install_canister(id, wasm, vec![], None);
    // Confirm installation responds before publishing browser connection details.
    assert_eq!(
        pic.query_call(id, Principal::anonymous(), "measurements", vec![])
            .unwrap()
            .len(),
        24
    );
    let host = pic.make_live(None);
    fs::create_dir_all(root.join("artifacts")).unwrap();
    fs::write(
        root.join("artifacts/connection.json"),
        serde_json::to_vec_pretty(&json!({"host":host.as_str(),"canisterId":id.to_text()}))
            .unwrap(),
    )
    .unwrap();
    println!("Local browser fixture ready; close stdin to stop it.");
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    pic.stop_live();
}
