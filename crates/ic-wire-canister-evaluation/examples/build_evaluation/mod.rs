//! Build and retain the exact benchmark artifacts through their measurement.
use ic_build::artifacts::{WasmBuildSpec, build_wasm_canisters_cached, resolve_cargo_build_inputs};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    let root = env::current_dir().unwrap();
    let trial = env::args().nth(1).expect("runtime, macros or browser");
    let output = PathBuf::from(env::args_os().nth(2).expect("output directory"));
    fs::create_dir_all(&output).unwrap();
    let compiler = Command::new("rustc").arg("-vV").output().unwrap();
    assert!(compiler.status.success());
    fs::write(output.join("build-rustc.txt"), &compiler.stdout).unwrap();
    fs::write(
        output.join("build-flags.txt"),
        format!(
            "RUSTFLAGS={}\nCARGO_ENCODED_RUSTFLAGS={}\n",
            env::var("RUSTFLAGS").unwrap_or_default(),
            env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default()
        ),
    )
    .unwrap();
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
        .with_cargo_profile_args([
            "--locked",
            "--profile",
            "canister-evaluation",
            "--features",
            features,
        ])
        .with_inherited_env(["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"]);
        let inputs = resolve_cargo_build_inputs(&spec).unwrap();
        let build = build_wasm_canisters_cached(&spec).unwrap();
        assert!(
            inputs.is_current(&spec).unwrap(),
            "source changed during build"
        );
        assert_eq!(build.record().input_digest(), inputs.input_digest());
        let bytes = fs::read(&build.record().artifacts()[0]).unwrap();
        let sha = format!("{:x}", Sha256::digest(&bytes));
        fs::write(output.join(format!("{name}.wasm")), bytes).unwrap();
        receipts.push(
            json!({"name": name, "features": features, "wasm_sha256": sha,
                "build_fingerprint": build.record().fingerprint().to_hex(),
                "input_digest": inputs.input_digest().to_hex(),
                "validation_digest": inputs.validation_digest().to_hex()}),
        );
        retained.push((spec, inputs, build));
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
    for (spec, inputs, _build) in &retained {
        assert!(
            inputs.is_current(spec).unwrap(),
            "source changed during measurement"
        );
    }
    if trial != "browser" {
        fs::rename(
            output.join("report.pending.json"),
            output.join("report.json"),
        )
        .unwrap();
    }
}
