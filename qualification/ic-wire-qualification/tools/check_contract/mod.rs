//! Conservative host contract diff, with validation owned by Wire's generator.
use ic_wire::typescript;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    #[serde(default)]
    types: BTreeMap<String, Value>,
    #[serde(default)]
    methods: Vec<Value>,
    #[serde(default)]
    payloads: Vec<Value>,
    #[serde(default)]
    vectors: BTreeMap<String, Vec<u8>>,
}
impl Snapshot {
    fn validate(&self) -> Result<(), typescript::GenerationError> {
        typescript::generate_registry(
            &self.types,
            &self.methods,
            "./wire-runtime.js",
            &BTreeMap::new(),
            |_| {},
        )?;
        typescript::generate_payload_registry(
            &self.types,
            &self.payloads,
            "./wire-runtime.js",
            &BTreeMap::new(),
            |_| {},
        )?;
        Ok(())
    }
}

fn changed<K: Ord + std::fmt::Display, V: PartialEq>(
    kind: &str,
    before: &BTreeMap<K, V>,
    after: &BTreeMap<K, V>,
    output: &mut Vec<String>,
) {
    for key in before.keys().chain(after.keys()).collect::<BTreeSet<_>>() {
        if before.get(key) != after.get(key) {
            output.push(format!("{kind}/{key}"));
        }
    }
}
fn changes(before: &Snapshot, after: &Snapshot) -> Vec<String> {
    let mut output = Vec::new();
    changed("types", &before.types, &after.types, &mut output);
    for (kind, a, b) in [
        ("methods", &before.methods, &after.methods),
        ("payloads", &before.payloads, &after.payloads),
    ] {
        // Both inputs have passed generation, including duplicate-name refusal.
        fn named(values: &[Value]) -> BTreeMap<&str, &Value> {
            values
                .iter()
                .map(|v| (v["name"].as_str().unwrap(), v))
                .collect()
        }
        changed(kind, &named(a), &named(b), &mut output);
    }
    changed("vectors", &before.vectors, &after.vectors, &mut output);
    output
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = clap::Command::new("check-contract")
        .about("Refuse changed contract declarations or frozen bytes; never rewrite the baseline")
        .arg(
            clap::Arg::new("before")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            clap::Arg::new("after")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .get_matches();
    let read = |path| -> Result<Snapshot, Box<dyn std::error::Error>> {
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    };
    let before = read(args.get_one::<PathBuf>("before").unwrap())?;
    let after = read(args.get_one::<PathBuf>("after").unwrap())?;
    for input in [&before, &after] {
        input
            .validate()
            .map_err(|error| format!("invalid contract snapshot: {error:?}"))?;
    }
    let changes = changes(&before, &after);
    if changes.is_empty() {
        println!("Contract declarations and frozen vectors unchanged");
        return Ok(());
    }
    for path in changes.iter().take(8) {
        eprintln!("changed: {path}");
    }
    Err(format!("{} changed declarations/vectors; review bytes, identities, limits and client adoption before updating the baseline. This is a conservative diff, not a compatibility proof or schema fingerprint.", changes.len()).into())
}

#[cfg(test)]
mod tests;
