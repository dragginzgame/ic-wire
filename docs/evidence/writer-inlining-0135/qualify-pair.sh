#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export CARGO_TARGET_DIR="$(cd "$root/.." && pwd)/candid-comparison-0133-build"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export POCKET_IC_BIN=/home/adam/.cache/canic/pocket-ic-server-16.0.0-pocket-ic-x86_64-linux/pocket-ic
export RUNS=2 SAMPLES=11 BATCH_MS=2
for variant in baseline output-inline; do
    python3 - "$root" "$variant" <<'PY'
from pathlib import Path
import shutil,sys
root=Path(sys.argv[1]);variant=root/'variants'/sys.argv[2]
for file in variant.rglob('*.rs'):
    shutil.copyfile(file,root/'source'/file.relative_to(variant))
PY
    cd "$root/source"
    touch crates/ic-wire-benchmarks/drivers/wasm.rs
    CODEC_SOURCE_SNAPSHOT="$root/source" CORPUS_FEATURE=cost-smoke bash scripts/ci/evaluate-codecs.sh "$root/$variant" > "$root/$variant.log" 2>&1
    python3 - "$root/$variant" <<'PY'
from pathlib import Path
import json,sys
root=Path(sys.argv[1])
for profile in ['evaluation','benchmark-speed']:
    rows=[json.loads(s) for s in (root/(profile+'-wire-build.jsonl')).read_text().splitlines()]
    for name in ['ic_wire','codec-wasm']:
        found=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name]
        assert len(found)==1 and not found[0]['fresh'],('cached candidate',name)
PY
    echo "Qualified $variant"
done
