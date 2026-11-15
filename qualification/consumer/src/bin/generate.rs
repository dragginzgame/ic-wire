//! Write standalone application clients and their matching runtime.
#[cfg(not(target_family = "wasm"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .ok_or("provide an output directory")?;
    let output = std::path::Path::new(&output);
    std::fs::create_dir_all(output)?;
    let (payload, methods) = ic_wire_consumer::clients("./runtime.ts")
        .map_err(|error| format!("schema generation failed: {error:?}"))?;
    std::fs::write(output.join("payload.ts"), payload)?;
    std::fs::write(output.join("methods.ts"), methods)?;
    std::fs::write(output.join("runtime.ts"), candidate::typescript::RUNTIME)?;
    Ok(())
}
#[cfg(target_family = "wasm")]
fn main() {
    panic!("client generation is host-only");
}
