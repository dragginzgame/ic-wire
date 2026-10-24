//! Host-only schema advice, optionally enforced in consumer CI.
use clap::{Arg, ArgAction, Command};
use ic_wire::typescript::lint_bulk_fields;
use std::{collections::BTreeMap, fs, process::ExitCode};

fn run() -> Result<ExitCode, String> {
    let args = Command::new("bulk-lint")
        .about("Suggest bulk fields in a JSON object of named Wire schemas")
        .arg(Arg::new("schemas").required(true))
        .arg(
            Arg::new("acknowledgements")
                .long("acknowledgements")
                .value_name("JSON"),
        )
        .arg(
            Arg::new("deny-warnings")
                .long("deny-warnings")
                .action(ArgAction::SetTrue),
        )
        .get_matches();
    let types = serde_json::from_slice(
        &fs::read(args.get_one::<String>("schemas").unwrap()).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let acknowledged: BTreeMap<String, String> = match args.get_one::<String>("acknowledgements") {
        Some(path) => serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?,
        None => BTreeMap::new(),
    };
    let report = lint_bulk_fields(&types, &acknowledged)
        .map_err(|e| format!("invalid bulk-lint input: {e:?}"))?;
    for warning in &report.warnings {
        eprintln!(
            "warning: {}: consider {}; preserve constraints and measure small-value size tradeoffs",
            warning.path, warning.representation
        );
    }
    for path in &report.stale_acknowledgements {
        eprintln!("warning: {path}: stale acknowledgement");
    }
    Ok(if args.get_flag("deny-warnings") && !report.is_clean() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
