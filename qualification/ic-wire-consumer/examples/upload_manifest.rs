//! Print the upload contract's API baseline using the client generator's descriptor.
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (graph, method) =
        ic_wire_consumer::upload_contract().map_err(|error| format!("upload schema: {error:?}"))?;
    let documents = graph
        .api_manifests(&[(method.export, method)])
        .map_err(|error| format!("upload manifest: {error:?}"))?;
    io::stdout()
        .lock()
        .write_all(documents[method.export].as_bytes())?;

    Ok(())
}
