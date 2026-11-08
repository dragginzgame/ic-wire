#!/usr/bin/env bash
# Focused explicit study; never part of ordinary correctness/release validation.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$root"
: "${POCKET_IC_BIN:?select a prepared PocketIC server}"
if [[ -n "${IC_TESTKIT_POCKET_IC_URL+x}" ]]; then
    echo 'unset IC_TESTKIT_POCKET_IC_URL; this study owns its prepared local server' >&2
    exit 1
fi
out="${1:?new evidence directory}"
baseline="${2:?explicit baseline commit required}"
baseline="$(git rev-parse "${baseline}^{commit}")"
case "$out" in /*) ;; *) out="$root/$out" ;; esac
mkdir "$out"
mkdir "$out/source"
cp Cargo.toml Cargo.lock rust-toolchain.toml "$out/source/"
# Copy source inputs, including local additions, without ignored build/dependency trees.
git ls-files -z --cached --others --exclude-standard -- crates examples > "$out/source-files"
node - "$root" "$out/source" "$out/source-files" <<'JS'
const fs = require('node:fs');
const path = require('node:path');
const [root, destination, inventory] = process.argv.slice(2);
const files = new Set(fs.readFileSync(inventory, 'utf8').split('\0').filter(Boolean));
for (const file of files) {
    const source = path.join(root, file);
    if (!fs.existsSync(source)) continue; // Preserve tracked local deletions.
    if (!fs.lstatSync(source).isFile()) throw new Error(`expected regular source: ${file}`);
    const target = path.join(destination, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(source, target);
}
JS
cp "$root/scripts/ci/evaluate-caller-storage.sh" "$out/runner.sh"
cp -R "$out/source" "$out/baseline"
# Replace the codec trees completely; an overlay would retain removed candidate files.
rm -r "$out/baseline/crates/ic-wire" "$out/baseline/crates/ic-wire-derive"
git archive "$baseline" crates/ic-wire crates/ic-wire-derive | tar -x -C "$out/baseline"
printf '%s\n' "$baseline" > "$out/baseline-commit.txt"
rustc -vV > "$out/compiler.txt"
git rev-parse HEAD > "$out/base-commit.txt"
git diff > "$out/dirty.patch"
shasum -a 256 Cargo.lock "$POCKET_IC_BIN" crates/ic-wire-benchmarks/benches/caller_storage/mod.rs > "$out/hashes.txt"
node - "$out" > "$out/source-hashes.json" <<'JS'
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const root = process.argv[2], hashes = {};
function visit(directory) {
    for (const entry of fs.readdirSync(path.join(root, directory), { withFileTypes: true }).sort((a,b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)) {
        const file = path.join(directory, entry.name);
        if (entry.isDirectory()) visit(file);
        else hashes[file] = crypto.createHash('sha256').update(fs.readFileSync(path.join(root, file))).digest('hex');
    }
}
visit('source'); visit('baseline');
console.log(JSON.stringify(hashes, null, 2));
JS
export CARGO_TARGET_DIR="$out/candidate-target"
cd "$out/source"
printf 'profile,lane,total_wasm_bytes,code_section_bytes\n' > "$out/wasm.csv"
for profile in evaluation benchmark-speed; do
    for lane in before-vec after-vec after-slice after-scoped; do
        features=()
        if [[ "$lane" == after-slice ]]; then features=(--features caller-slice); fi
        if [[ "$lane" == after-scoped ]]; then features=(--features caller-scoped); fi
        if [[ "$lane" == before-vec ]]; then
            CARGO_TARGET_DIR="$out/baseline-target" cargo build --locked --offline \
                --manifest-path "$out/baseline/Cargo.toml" --profile "$profile" \
                -p ic-wire-benchmarks --example caller-storage --target wasm32-unknown-unknown \
                > "$out/$profile-$lane-build.log" 2>&1
            artifact="$out/baseline-target/wasm32-unknown-unknown/$profile/examples/caller_storage.wasm"
        else
            cargo build --locked --offline --profile "$profile" -p ic-wire-benchmarks \
            --example caller-storage --target wasm32-unknown-unknown "${features[@]}" \
            > "$out/$profile-$lane-build.log" 2>&1
            artifact="$CARGO_TARGET_DIR/wasm32-unknown-unknown/$profile/examples/caller_storage.wasm"
        fi
        cp "$artifact" "$out/$profile-$lane.wasm"
        bytes="$(wc -c < "$artifact" | tr -d ' ')"
        code="$(cargo run --locked --offline -p ic-wire-benchmarks --example codec-report -- --code-size "$artifact" 2>> "$out/$profile-$lane-build.log")"
        printf '%s,%s,%s,%s\n' "$profile" "$lane" "$bytes" "$code" >> "$out/wasm.csv"
    done
    mkdir "$out/$profile-native"
    for mode in time allocations heap; do
        features=()
        if [[ "$mode" != time ]]; then features=(--features "$mode"); fi
        IC_WIRE_STORAGE_EVIDENCE="$out/$profile-native" cargo test --locked --offline \
            --profile "$profile" -p ic-wire-benchmarks --example caller-storage "${features[@]}" \
            native_measurement -- --ignored --test-threads=1 \
            > "$out/$profile-native-$mode.log" 2>&1
    done
done
IC_WIRE_STORAGE_EVIDENCE="$out" cargo test --locked --offline -p ic-wire-benchmarks \
    --example caller-storage --features ic ic_measurement -- --ignored --test-threads=1 \
    > "$out/ic.log" 2>&1
printf 'Evidence: %s\n' "$out"
