use std::{collections::BTreeMap, env, fs, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-env-changed=WIRE_ADVICE_CASE");
    let case = env::var("WIRE_ADVICE_CASE").unwrap();
    let types = BTreeMap::from([(
        "Payload".into(),
        serde_json::json!({
            "k":"record","fields":[
                {"name":"data","value":{"k":"list","item":{"k":"u","bits":8}}},
                {"name":"words","value":{"k":"list","item":{"k":"u","bits":16}}}
            ]
        }),
    )]);
    let mut acknowledgements = BTreeMap::new();
    if case != "default" {
        acknowledgements.insert(
            "/types/Payload/fields/data/list/u8".into(),
            "At most four bytes".into(),
        );
    }
    if case == "all" || case == "stale" {
        acknowledgements.insert(
            "/types/Payload/fields/words/list/u16".into(),
            "Small varints".into(),
        );
    }
    if case == "stale" {
        acknowledgements.insert("/removed/field".into(), "Removed declaration".into());
    }
    let source = ic_wire::typescript::generate_registry(
        &types,
        &[],
        "./runtime.js",
        &acknowledgements,
        |report| {
            for warning in report.warnings {
                println!("cargo::warning={warning}");
            }
            for path in report.stale_acknowledgements {
                println!("cargo::warning={path}: stale acknowledgement");
            }
        },
    )
    .unwrap();
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings.ts"),
        source,
    )
    .unwrap();
}
