from pathlib import Path
import shutil
import sys
out=Path(sys.argv[1])
here=Path(__file__).resolve().parent
s=out/'compression'
shutil.copytree(out/'baseline',s)
p=s/'Cargo.toml'
p.write_text(p.read_text().replace('# Tests and tooling only','# Tests and tooling only\nlz4_flex = { version = "=0.14.0", default-features = false, features = ["std", "safe-encode", "safe-decode", "checked-decode"] }'))
p=s/'crates/ic-wire-benchmarks/Cargo.toml'
p.write_text(p.read_text().replace('[features]','[features]\ncompression = []').replace('[dev-dependencies]','[dev-dependencies]\nlz4_flex.workspace = true')+'''
[[example]]
name = "compression-probe"
path = "drivers/compression_probe.rs"
required-features = ["wire", "ic"]

[[example]]
name = "compression-wasm"
path = "drivers/compression_wasm.rs"
crate-type = ["cdylib"]
''')
d=s/'crates/ic-wire-benchmarks/drivers'
for name in ("compression_probe.rs", "compression_wasm.rs"):
    shutil.copyfile(here/"compression"/name, d/name)
shutil.copytree(here/"compression/compression_transport", d/"compression_transport")
