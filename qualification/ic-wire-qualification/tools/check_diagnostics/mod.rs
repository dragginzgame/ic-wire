//! IC Wire cases and compiler recipe for IC Build's diagnostic gate.
use ic_build::cargo::{
    diagnostics::{check_suite, library_artifact},
    observation::schema::{Message, TargetKind},
};
use std::{
    collections::BTreeSet,
    env,
    fs::{self, File},
    io::{self, BufReader, Read},
    path::{Path, PathBuf},
    process::Command,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    let arguments = clap::Command::new("check-diagnostics")
        .arg(
            clap::Arg::new("directory")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            clap::Arg::new("endpoint")
                .long("endpoint")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();
    let run = arguments.get_one::<PathBuf>("directory").unwrap();
    let source = root.join(if arguments.get_flag("endpoint") {
        "qualification/ic-wire-qualification/tests/endpoint_ui"
    } else {
        "crates/ic-wire/tests/ui"
    });
    let wire = library_artifact(
        File::open(run.join("artifacts.jsonl"))?,
        "ic_wire",
        &TargetKind::Lib,
        4 * 1024 * 1024,
    )?;
    let endpoint = if arguments.get_flag("endpoint") {
        Some(library_artifact(
            File::open(run.join("artifacts.jsonl"))?,
            "ic_wire_endpoint",
            &TargetKind::Lib,
            4 * 1024 * 1024,
        )?)
    } else {
        None
    };
    let dependencies = dependency_directories(File::open(run.join("artifacts.jsonl"))?)?;
    let (names, expected): (Vec<_>, Vec<_>) = diagnostic_cases(&source)?.into_iter().unzip();
    fs::write(run.join("cases.txt"), names.join("\n") + "\n")?;
    check_suite(&expected, run, 1024 * 1024, |case| {
        let name = case.map_or("valid", |index| names[index].as_str());
        let mut command = Command::new("rustc");
        command
            // Direct rustc bypasses Cargo's target environment. Match the
            // --crate-name below so Platform can scope native export symbols.
            .env("CARGO_CRATE_NAME", "ui_contract")
            .args([
                "--edition=2024",
                "--crate-type=lib",
                "--emit=metadata",
                "--crate-name",
                "ui_contract",
            ])
            .arg("--extern")
            .arg(format!("ic_wire={}", wire.display()))
            .arg("--cfg")
            .arg(format!("ui_case=\"{name}\""))
            .arg(source.join("mod.rs"))
            .arg("-o")
            .arg(run.join(format!("{name}.rmeta")));
        // Rust 1.100 stores dependencies in per-package output directories.
        // Cargo's current artifact stream owns paths; never scan stale deps/.
        for directory in &dependencies {
            command
                .arg("-L")
                .arg(format!("dependency={}", directory.display()));
        }
        if let Some(endpoint) = &endpoint {
            command
                .arg("--extern")
                .arg(format!("ic_wire_endpoint={}", endpoint.display()));
        }
        command
    })?;
    println!(
        "{} expected diagnostics and passing control verified",
        names.len()
    );
    Ok(())
}

// Called only after library_artifact admitted the complete, bounded Cargo stream.
fn dependency_directories(input: impl Read) -> io::Result<BTreeSet<PathBuf>> {
    let mut directories = BTreeSet::new();
    for message in Message::parse_stream(BufReader::new(input)) {
        if let Message::CompilerArtifact(artifact) = message? {
            for file in artifact.filenames {
                if matches!(
                    file.extension(),
                    Some("rlib" | "rmeta" | "so" | "dylib" | "dll")
                ) {
                    let parent = file
                        .parent()
                        .ok_or_else(|| io::Error::other("artifact has no parent"))?;
                    directories.insert(parent.to_path_buf().into_std_path_buf());
                }
            }
        }
    }
    Ok(directories)
}

// Only module wiring and the explicit pass/support inputs lack expected.txt.
// Missing or unreadable required case files must not silently shrink the suite.
fn diagnostic_cases(source: &Path) -> io::Result<Vec<(String, Vec<String>)>> {
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid diagnostic case inventory",
        )
    };
    let mut cases = Vec::new();
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let name = entry.file_name().into_string().map_err(|_| invalid())?;
        if name == "mod.rs" && kind.is_file() {
            continue;
        }
        if !kind.is_dir() {
            return Err(invalid());
        }
        if matches!(name.as_str(), "valid" | "support") {
            continue;
        }
        let text = fs::read_to_string(entry.path().join("expected.txt"))?;
        cases.push((name, text.lines().map(str::to_owned).collect()));
    }
    if cases.is_empty() {
        return Err(invalid());
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(cases)
}

#[cfg(test)]
mod tests;
