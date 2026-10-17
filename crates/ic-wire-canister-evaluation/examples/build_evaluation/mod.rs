//! Build and retain the exact benchmark artifacts through their measurement.
#[path = "../evidence/mod.rs"]
mod evidence;

use ic_build::{
    artifacts::{WasmBuildEvidence, WasmBuildSpec},
    cargo::selection::{CargoFeatureSelection, CargoSelection},
};
use serde_json::json;
use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let root = env::current_dir().unwrap();
    let trial = env::args().nth(1).expect("runtime, macros or browser");
    let output = PathBuf::from(env::args_os().nth(2).expect("output directory"));
    fs::create_dir_all(&output).unwrap();
    let server = PathBuf::from(env::var_os("POCKET_IC_BIN").expect("prepared PocketIC"));
    let mut provenance = evidence::capture(&server);
    let mut retained = Vec::new();
    let mut receipts = Vec::new();
    let recipes = match trial.as_str() {
        "runtime" => vec![
            ("candid-internal", "runtime-benchmark,candid-codec"),
            ("wire-internal", "runtime-benchmark"),
            (
                "candid-management",
                "runtime-benchmark,candid-codec,management",
            ),
            ("wire-management", "runtime-benchmark,management"),
        ],
        "macros" => vec![("macros", "macro-fixture")],
        "browser" => vec![("browser", "browser-fixture")],
        _ => panic!("unknown trial"),
    };
    for (name, features) in recipes {
        let spec = WasmBuildSpec::new(
            &root,
            &root.join("target/evaluation-builds").join(name),
            &["ic-wire-canister-evaluation"],
        )
        .with_cargo_selection(CargoSelection {
            profile: Some("canister-evaluation".into()),
            features: CargoFeatureSelection {
                features: features.split(',').map(str::to_owned).collect(),
                ..CargoFeatureSelection::default()
            },
            locked: true,
            ..CargoSelection::default()
        })
        .with_inherited_env(["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"]);
        let build = WasmBuildEvidence::acquire(spec).unwrap();
        // Consumer-selected export cap; IC Build owns capture and provenance.
        let mut receipt = json!(
            build
                .export_artifact(0, &output.join(format!("{name}.wasm")), 64 * 1024 * 1024,)
                .unwrap()
        );
        receipt["name"] = json!(name);
        receipt["features"] = json!(features);
        receipts.push(receipt);
        retained.push(build);
    }
    fs::write(
        output.join("build-records.json"),
        serde_json::to_vec_pretty(&receipts).unwrap(),
    )
    .unwrap();
    // Caller source custody: no source edits during this command. Before/after
    // guards detect persistent changes, not transient edits that are reverted.
    let mut host = Command::new("cargo");
    host.args([
        "run",
        "--quiet",
        "--locked",
        "-p",
        "ic-wire-canister-evaluation",
    ]);
    match trial.as_str() {
        "runtime" => {
            host.args([
                "--features",
                "runtime-benchmark",
                "--example",
                "runtime-measure",
                "--",
            ])
            .arg(&output);
        }
        "macros" => {
            host.args([
                "--features",
                "macro-fixture",
                "--example",
                "macro-fixture",
                "--",
            ])
            .arg(output.join("macros.wasm"))
            .arg(output.join("report.pending.json"));
        }
        "browser" => {
            host.args([
                "--features",
                "bindings",
                "--example",
                "browser-fixture",
                "--",
                "examples/browser",
                "--generate-only",
            ]);
        }
        _ => unreachable!(),
    }
    assert!(
        host.status().unwrap().success(),
        "fixture failed; artifacts retained"
    );
    if trial == "browser" {
        for args in [vec!["run", "check"], vec!["run", "build"], vec!["test"]] {
            assert!(
                Command::new("npm")
                    .args(args)
                    .current_dir(root.join("examples/browser"))
                    .env("IC_WIRE_BROWSER_WASM", output.join("browser.wasm"))
                    .status()
                    .unwrap()
                    .success(),
                "browser fixture failed"
            );
        }
    }
    for build in &retained {
        build
            .verify_current()
            .expect("source changed during measurement");
    }
    provenance["sources"] = evidence::sources(&trial);
    provenance["build_records"] = json!(receipts);
    let report_dir = if trial == "browser" {
        root.join("examples/browser/artifacts")
    } else {
        output
    };
    let pending = report_dir.join("report.pending.json");
    let mut report: serde_json::Value =
        serde_json::from_slice(&fs::read(&pending).unwrap()).unwrap();
    report["provenance"] = provenance;
    fs::write(&pending, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    fs::rename(pending, report_dir.join("report.json")).unwrap();
}
