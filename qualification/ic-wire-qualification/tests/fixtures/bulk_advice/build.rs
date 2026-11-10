use std::{collections::BTreeMap, env, fs, path::PathBuf};

fn main() {
    println!("cargo::rerun-if-env-changed=WIRE_ADVICE_CASE");
    let case = env::var("WIRE_ADVICE_CASE").unwrap();
    let types = BTreeMap::from([(
        "Payload".into(),
        serde_json::json!({
            "k":"record","fields":[
                {"name":"data","value":{"k":"list","item":{"k":"u","bits":8}}},
                {"name":"words","value":{"k":"list","item":{"k":"u","bits":16}}},
                {"name":"ids","value":{"k":"list","item":{"k":"array","max":16,"item":{"k":"u","bits":8}}}},
                {"name":"flags","value":{"k":"list","item":{"k":"bool"}}},
                {"name":"signed","value":{"k":"list","item":{"k":"i","bits":32}}},
                {"name":"hashes","value":{"k":"list","item":{"k":"array","max":32,"item":{"k":"u","bits":8}}}},
                {"name":"samples32","value":{"k":"list","item":{"k":"float","bits":32}}},
                {"name":"samples64","value":{"k":"list","item":{"k":"float","bits":64}}}
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
        for suffix in ["signed/list/i32", "hashes/list/hash32", "samples32/list/f32", "samples64/list/f64"] {
            acknowledgements.insert(format!("/types/Payload/fields/{suffix}"), "Small bounded control; retain ordinary representation".into());
        }
        acknowledgements.insert("/types/Payload/fields/ids/list/hash16".into(), "Small bounded ID list".into());
        acknowledgements.insert("/types/Payload/fields/flags/list/bool".into(), "Bounded admin flags".into());
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
