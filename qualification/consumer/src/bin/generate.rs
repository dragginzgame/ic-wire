//! Write standalone application clients and their matching runtime.
#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let output = args
        .next()
        .ok_or("provide an output directory and optional --deny-warnings")?;
    let deny = match args.next() {
        None => false,
        Some(flag) if flag == "--deny-warnings" => true,
        _ => return Err("unknown argument; expected --deny-warnings".into()),
    };
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let output = std::path::Path::new(&output);
    let mut clean = true;
    let (payload, methods) = ic_wire_consumer::clients("./runtime.ts", |report| {
        eprint!("{report}");
        clean &= report.is_clean();
    })
    .map_err(|error| format!("schema generation failed: {error:?}"))?;
    if deny && !clean {
        return Err("unreviewed bulk diagnostics".into());
    }
    std::fs::create_dir_all(output)?;
    std::fs::write(output.join("payload.ts"), payload)?;
    std::fs::write(output.join("methods.ts"), methods)?;
    std::fs::write(output.join("runtime.ts"), candidate::typescript::RUNTIME)?;
    Ok(())
}
#[cfg(target_family = "wasm")]
fn main() {
    panic!("client generation is host-only");
}
