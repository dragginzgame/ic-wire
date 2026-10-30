//! IC Wire cases and compiler recipe for IC Build's diagnostic gate.
use ic_build::cargo::{
    diagnostics::{check_suite, library_artifact},
    observation::schema::TargetKind,
};
use std::{
    env,
    fs::{self, File},
    path::PathBuf,
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
        "crates/ic-wire-qualification/tests/endpoint_ui"
    } else {
        "crates/ic-wire/tests/ui"
    });
    let wire = library_artifact(
        File::open(run.join("artifacts.jsonl"))?,
        "ic_wire",
        &TargetKind::Lib,
        4 * 1024 * 1024,
    )?;
    let mut cases = fs::read_dir(&source)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    cases.retain(|path| path.join("expected.txt").is_file());
    cases.sort();
    let names = cases
        .iter()
        .map(|path| path.file_name().unwrap().to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    fs::write(run.join("cases.txt"), names.join("\n") + "\n")?;
    let expected = cases
        .iter()
        .map(|path| {
            fs::read_to_string(path.join("expected.txt"))
                .map(|text| text.lines().map(str::to_owned).collect())
        })
        .collect::<Result<Vec<_>, _>>()?;
    check_suite(&expected, run, 1024 * 1024, |case| {
        let name = case.map_or("valid", |index| names[index].as_str());
        let mut command = Command::new("rustc");
        command
            .args([
                "--edition=2024",
                "--crate-type=lib",
                "--emit=metadata",
                "--crate-name",
                "ui_contract",
            ])
            .arg("--extern")
            .arg(format!("ic_wire={}", wire.display()))
            .arg("-L")
            .arg(format!(
                "dependency={}/deps",
                wire.parent().unwrap().display()
            ))
            .arg("--cfg")
            .arg(format!("ui_case=\"{name}\""))
            .arg(source.join("mod.rs"))
            .arg("-o")
            .arg(run.join(format!("{name}.rmeta")));
        command
    })?;
    println!(
        "{} expected diagnostics and passing control verified",
        cases.len()
    );
    Ok(())
}
