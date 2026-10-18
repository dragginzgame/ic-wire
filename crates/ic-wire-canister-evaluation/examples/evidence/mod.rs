//! Consumer-owned report identities; IC Host owns bounded streaming file hashes.
use ic_host_fs::read::hash_file;
use serde_json::{Value, json};
use std::{env, ffi::OsStr, path::Path, process::Command};

pub fn capture(server: &Path) -> Value {
    let version = |program: &OsStr, args: &[&str]| {
        let output = Command::new(program).args(args).output().unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    };
    json!({
        "rustc": version(&env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()), &["-vV"]),
        "rustflags": env::var("RUSTFLAGS").unwrap_or_default(),
        "encoded_rustflags": env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default(),
        "pocket_ic_version": version(server.as_os_str(), &["--version"]),
        "pocket_ic_sha256": hash_file(server, 512 * 1024 * 1024).unwrap().sha256.to_string(),
    })
}

pub fn sources(trial: &str) -> Value {
    let paths: &[&str] = match trial {
        "macros" => &[
            "Cargo.toml",
            "Cargo.lock",
            "crates/ic-wire-derive/Cargo.toml",
            "crates/ic-wire-derive/src/lib.rs",
            "crates/ic-wire-derive/src/enum_expansion/mod.rs",
            "crates/ic-wire-adapter-fixture/Cargo.toml",
            "crates/ic-wire-adapter-fixture/src/lib.rs",
            "crates/ic-wire-adapter-fixture/src/adapter_fixture/mod.rs",
            "crates/ic-wire-canister-evaluation/Cargo.toml",
            "crates/ic-wire-canister-evaluation/src/lib.rs",
            "crates/ic-wire-canister-evaluation/src/macro_fixture/mod.rs",
            "crates/ic-wire-canister-evaluation/src/macro_fixture/tests/mod.rs",
            "crates/ic-wire-canister-evaluation/examples/macro_fixture/mod.rs",
            "crates/ic-wire/src/contract/mod.rs",
            "crates/ic-wire/src/shape/mod.rs",
            "scripts/ci/evaluate-macros.sh",
        ],
        "browser" => &[
            "examples/browser/node_modules/@dragginzgame/ic-testkit-browser/testing/browser/session.mjs",
            "examples/browser/test.mjs",
            "examples/browser/package.json",
            "examples/browser/package-lock.json",
            "examples/browser/tsconfig.json",
            "examples/browser/dist/main.js",
            "examples/browser/generated/apply.ts",
            "examples/browser/generated/inspect.ts",
            "examples/browser/generated/wire-runtime.ts",
            "examples/browser/generated/vectors.json",
            "examples/browser/src/client.ts",
            "examples/browser/src/main.ts",
            "examples/browser/src/typechecks.ts",
            "Cargo.toml",
            "Cargo.lock",
            "crates/ic-wire/Cargo.toml",
            "crates/ic-wire/src/lib.rs",
            "crates/ic-wire/src/bounded/mod.rs",
            "crates/ic-wire/src/native/mod.rs",
            "crates/ic-wire/src/native/encode/mod.rs",
            "crates/ic-wire/src/native/decode/mod.rs",
            "crates/ic-wire/src/shape/mod.rs",
            "crates/ic-wire-canister-evaluation/src/lib.rs",
            "crates/ic-wire-canister-evaluation/src/browser/mod.rs",
            "crates/ic-wire-canister-evaluation/src/commands/mod.rs",
            "crates/ic-wire/src/typescript/mod.rs",
            "crates/ic-wire/src/typescript/metadata/mod.rs",
            "crates/ic-wire/src/typescript/runtime.ts",
            "crates/ic-wire/src/contract/mod.rs",
            "crates/ic-wire/src/schema/mod.rs",
            "crates/ic-wire-derive/src/lib.rs",
            "crates/ic-wire-derive/src/enum_expansion/mod.rs",
            "crates/ic-wire-canister-evaluation/Cargo.toml",
            "scripts/ci/evaluate-browser.sh",
            "crates/ic-wire-canister-evaluation/examples/browser_fixture/mod.rs",
            "examples/browser/generated/native-vectors.json",
        ],
        _ => panic!("unknown evidence trial"),
    };
    paths
        .iter()
        .copied()
        .chain([
            "crates/ic-wire-canister-evaluation/examples/build_evaluation/mod.rs",
            "crates/ic-wire-canister-evaluation/examples/evidence/mod.rs",
        ])
        .map(|path| {
            (
                path.to_owned(),
                json!(
                    hash_file(Path::new(path), 16 * 1024 * 1024)
                        .unwrap()
                        .sha256
                        .to_string()
                ),
            )
        })
        .collect::<serde_json::Map<_, _>>()
        .into()
}
