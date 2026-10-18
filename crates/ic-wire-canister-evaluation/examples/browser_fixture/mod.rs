//! Generate runtime clients and serve a controller-bound local browser fixture.
use ic_platform::Principal;
use ic_testkit::pic::{CanisterInstallExt, InstallSpec, PocketIcBuilder, PocketIcStartupConfig};
use ic_wire::{List, Method, Text, encode_request, encode_response};
use ic_wire_canister_evaluation::commands::{
    Apply, Command, Failure, Inspect, Inspection, NATIVE_ID, NATIVE_LIMITS, NativeTree, Receipt,
    Snapshot,
};
use serde_json::json;
use std::{env, fs, io, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::args_os().nth(1).expect("browser evaluation directory"));
    let generated = root.join("generated");
    fs::create_dir_all(&generated).unwrap();
    fs::write(
        generated.join("wire-runtime.ts"),
        ic_wire::typescript::RUNTIME,
    )
    .unwrap();
    fs::write(
        generated.join("apply.ts"),
        ic_wire::typescript::generate::<Apply>().unwrap(),
    )
    .unwrap();
    fs::write(
        generated.join("inspect.ts"),
        ic_wire::typescript::generate::<Inspect>().unwrap(),
    )
    .unwrap();
    if env::args().any(|arg| arg == "--generate-only") {
        return;
    }
    let caller =
        Principal::from_text(env::var("IC_WIRE_BROWSER_CALLER").expect("test identity principal"))
            .unwrap();
    let server = PathBuf::from(env::var_os("POCKET_IC_BIN").expect("prepared PocketIC binary"));
    let wasm =
        fs::read(env::var_os("IC_WIRE_BROWSER_WASM").expect("runtime fixture Wasm path")).unwrap();
    let mut owned = PocketIcStartupConfig::Spawn {
        server_binary: server.clone(),
        timeout: std::time::Duration::from_secs(60),
        server_hard_ttl: None,
        server_idle_ttl: None,
        server_output_files: None,
    }
    .build_instance(PocketIcBuilder::new().with_application_subnet())
    .unwrap();
    let pic = &mut owned.instance;
    let id = pic
        .try_create_and_install(InstallSpec {
            wasm,
            init_bytes: vec![],
            cycles: 10_000_000_000_000,
            install_sender: None,
            label: None,
        })
        .unwrap();
    pic.set_controllers(id, None, vec![Principal::anonymous(), caller])
        .unwrap();
    let inspection = Inspection {
        labels: List::new(vec![
            Text::new("é🦀".into()).unwrap(),
            Text::new("\u{feff}text".into()).unwrap(),
        ])
        .unwrap(),
        correlation: Some(u64::MAX),
    };
    let query = encode_request::<Inspect>(&inspection).unwrap();
    let query_reply = encode_response::<Inspect>(&Snapshot {
        balance: 0,
        inspection,
    })
    .unwrap();
    assert_eq!(
        pic.query_call(id, caller, Inspect::NAME, query.as_bytes().to_vec())
            .unwrap(),
        query_reply.as_bytes()
    );
    let command = Command { id: 1, amount: 7 };
    let request = encode_request::<Apply>(&command).unwrap();
    let success = encode_response::<Apply>(&Ok(Receipt { id: 1, balance: 7 })).unwrap();
    let failure =
        encode_response::<Apply>(&Err(Failure::Overflow(Receipt { id: 2, balance: 7 }))).unwrap();
    fs::write(generated.join("vectors.json"), serde_json::to_vec_pretty(&json!({"query":query.as_bytes(),"queryReply":query_reply.as_bytes(),"request":request.as_bytes(),"success":success.as_bytes(),"failure":failure.as_bytes()})).unwrap()).unwrap();
    let tree = NativeTree::Branch(vec![NativeTree::Leaf("é".into(), u128::MAX)]);
    let native_request = ic_wire::native::encode_request(NATIVE_ID, &tree, NATIVE_LIMITS).unwrap();
    let native_reply = ic_wire::native::encode_response(NATIVE_ID, &tree, NATIVE_LIMITS).unwrap();
    assert_eq!(
        pic.query_call(id, caller, "native_echo", native_request.clone())
            .unwrap(),
        native_reply
    );
    fs::write(
        generated.join("native-vectors.json"),
        serde_json::to_vec_pretty(
            &json!({"id":NATIVE_ID.to_string(),"request":native_request,"response":native_reply}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut deep = tree;
    for _ in 0..64 {
        deep = NativeTree::Branch(vec![deep]);
    }
    let mut forged = native_request[..11].to_vec();
    forged.extend(postcard::to_allocvec(&deep).unwrap());
    assert!(pic.query_call(id, caller, "native_echo", forged).is_err());
    let host = pic.make_live(None);
    fs::create_dir_all(root.join("artifacts")).unwrap();
    fs::write(
        root.join("artifacts/connection.json"),
        serde_json::to_vec_pretty(
            &json!({"host":host.as_str(),"canisterId":id.to_text(),"caller":caller.to_text()}),
        )
        .unwrap(),
    )
    .unwrap();
    println!("Local runtime browser fixture ready; close stdin to stop it.");
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    pic.stop_live();
}
