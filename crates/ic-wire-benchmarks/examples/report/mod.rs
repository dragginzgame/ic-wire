//! Summarize raw samples without dropping cases or aggregating unlike workloads.
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
fn command() -> clap::Command {
    clap::Command::new("codec-report")
        .about("Summarize codec evidence, inspect a Wasm code section, or find a Cargo artifact")
        .arg(clap::Arg::new("directory").value_parser(clap::value_parser!(PathBuf)))
        .arg(
            clap::Arg::new("primary")
                .value_parser(["wire-native", "wire-bounded", "wire-packed-i16"])
                .default_value_if(
                    "directory",
                    clap::builder::ArgPredicate::IsPresent,
                    "wire-native",
                )
                .requires("directory"),
        )
        .arg(
            clap::Arg::new("code-size")
                .long("code-size")
                .value_name("WASM")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            clap::Arg::new("artifact")
                .long("artifact")
                .value_name("CARGO_JSON")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .group(
            clap::ArgGroup::new("mode")
                .args(["directory", "code-size", "artifact"])
                .required(true),
        )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = command().get_matches();
    if let Some(path) = arguments.get_one::<PathBuf>("code-size") {
        println!("{}", code_size(&fs::read(path)?)?);
    } else if let Some(path) = arguments.get_one::<PathBuf>("artifact") {
        println!("{}", artifact(&fs::read_to_string(path)?)?.display());
    } else {
        let root = arguments.get_one::<PathBuf>("directory").unwrap();
        let primary = arguments.get_one::<String>("primary").unwrap();
        let report = comparison(root, primary)?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("comparison.csv"))?
            .write_all(&report)?;
    }
    Ok(())
}

// Read section framing only; this reports the code-section payload, not module
// validity or a sum of compressed bytes. Cargo supplies the validated module.
fn code_size(wasm: &[u8]) -> Result<usize, &'static str> {
    if !wasm.starts_with(b"\0asm\x01\0\0\0") {
        return Err("expected a core Wasm version 1 module");
    }
    let mut rest = &wasm[8..];
    let mut code = None;
    while let Some((&id, tail)) = rest.split_first() {
        rest = tail;
        let mut length = 0_u32;
        for shift in (0..35).step_by(7) {
            let (&byte, tail) = rest.split_first().ok_or("truncated section length")?;
            rest = tail;
            if shift == 28 && byte > 15 {
                return Err("section length overflow");
            }
            length |= u32::from(byte & 127) << shift;
            if byte & 128 == 0 {
                break;
            }
        }
        let length = usize::try_from(length).map_err(|_| "section length overflow")?;
        rest = rest.get(length..).ok_or("truncated section payload")?;
        if id == 10 && code.replace(length).is_some() {
            return Err("duplicate code section");
        }
    }
    Ok(code.unwrap_or(0))
}

fn artifact(messages: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();
    for line in messages.lines() {
        let row: Value = serde_json::from_str(line)?;
        if row["reason"] == "compiler-artifact" && row["target"]["name"] == "codec-wasm" {
            for name in row["filenames"]
                .as_array()
                .ok_or("missing artifact filenames")?
            {
                let path = PathBuf::from(name.as_str().ok_or("invalid artifact filename")?);
                if path
                    .extension()
                    .is_some_and(|extension| extension == "wasm")
                {
                    paths.push(path);
                }
            }
        }
    }
    if paths.len() != 1 {
        return Err("expected exactly one codec-wasm Wasm artifact".into());
    }
    Ok(paths.remove(0))
}

fn comparison(root: &Path, primary: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut out = Vec::new();
    writeln!(
        out,
        "run,case,wire_codec,operation,wire_bytes,candid_bytes,wire_size_change_pct,wire_ns,candid_ns,wire_time_change_pct,candid_over_wire_speed_ratio"
    )?;
    let mut dirs = fs::read_dir(root)?
        .map(|e| e.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|p| {
            p.is_dir()
                && p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains("-native-")
        })
        .collect::<Vec<_>>();
    if dirs.is_empty() {
        return Err("no native benchmark runs found".into());
    }
    dirs.sort();
    for dir in dirs {
        let mut rows = BTreeMap::new();
        for (index, line) in fs::read_to_string(dir.join("samples.jsonl"))?
            .lines()
            .enumerate()
        {
            let row: Value = serde_json::from_str(line)?;
            // Reject incomplete evidence before pairing: two absent identities
            // must not compare equal, and zero denominators cannot yield ratios.
            let strings = ["case", "operation", "codec", "input_sha256"];
            if strings
                .iter()
                .any(|key| row[key].as_str().is_none_or(str::is_empty))
                || row["encoded_bytes"].as_u64().is_none_or(|bytes| bytes == 0)
                || row["measurement"]["median_ns"]
                    .as_f64()
                    .is_none_or(|time| !time.is_finite() || time <= 0.0)
            {
                return Err(format!(
                    "invalid measurement in {} at line {}",
                    dir.display(),
                    index + 1
                )
                .into());
            }
            let previous = rows.insert(
                (
                    row["case"].as_str().unwrap().to_owned(),
                    row["operation"].as_str().unwrap().to_owned(),
                    row["codec"].as_str().unwrap().to_owned(),
                ),
                row,
            );
            if previous.is_some() {
                return Err(format!("duplicate measurement in {}", dir.display()).into());
            }
        }
        let mut matched = 0;
        for ((case, operation, codec), wire) in &rows {
            if codec == "candid"
                && !rows.contains_key(&(case.clone(), operation.clone(), primary.into()))
            {
                return Err(format!("missing Wire measurement: {case}/{operation}").into());
            }
            if !codec.starts_with("wire-") {
                continue;
            }
            // Reuse is explicitly Wire-only; every other operation needs a pair.
            if operation == "encode-reuse" {
                if !rows.contains_key(&(case.clone(), "encode-fresh".into(), codec.clone())) {
                    return Err(format!("reuse without fresh encoding: {case}/{codec}").into());
                }
                continue;
            }
            let candid = rows
                .get(&(case.clone(), operation.clone(), "candid".into()))
                .ok_or_else(|| format!("missing Candid measurement: {case}/{operation}"))?;
            matched += 1;
            if wire["input_sha256"] != candid["input_sha256"] {
                return Err(format!("input identity mismatch: {case}/{operation}/{codec}").into());
            }
            let w = wire["measurement"]["median_ns"].as_f64().unwrap();
            let c = candid["measurement"]["median_ns"].as_f64().unwrap();
            let wb = wire["encoded_bytes"].as_f64().unwrap();
            let cb = candid["encoded_bytes"].as_f64().unwrap();
            writeln!(
                out,
                "{},{case},{codec},{operation},{wb},{cb},{:.3},{w},{c},{:.3},{:.3}",
                dir.file_name().unwrap().to_string_lossy(),
                100.0 * (wb / cb - 1.0),
                100.0 * (w / c - 1.0),
                c / w
            )?;
        }
        if matched == 0 {
            return Err(format!("no matched measurements in {}", dir.display()).into());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
