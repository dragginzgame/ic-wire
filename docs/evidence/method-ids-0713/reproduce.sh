#!/usr/bin/env bash
set -euo pipefail
evidence=$(cd -- "$(dirname -- "$0")" && pwd)
repo=$(git -C "$evidence" rev-parse --show-toplevel)
work=$(mktemp -d)
printf 'Evidence output: %s\n' "$work"
mkdir "$work/before" "$work/after"
git -C "$repo" archive 974b6ff3e4a89c37a6c93081d7a26bee2e709f7b | tar -x -C "$work/before"
cp -a "$work/before/." "$work/after/"
git -C "$work/after" apply "$evidence/candidate.patch"
for variant in baseline candidate; do
    source=before
    if [ "$variant" = candidate ]; then source=after; fi
    probe="$work/$variant"
    mkdir -p "$probe/src"
    cp "$work/before/scripts/benchmarks/generator/mod.rs" "$probe/src/main.rs"
    cp "$work/before/Cargo.lock" "$work/before/rust-toolchain.toml" "$probe/"
    cat > "$probe/Cargo.toml" <<EOF
[workspace]
[package]
name="generator-probe"
version="0.0.0"
edition="2024"
publish=false
[dependencies]
ic-wire={path="$work/$source/crates/ic-wire",features=["typescript"]}
serde_json="1"
[profile.release]
opt-level=3
lto=true
codegen-units=1
EOF
    (cd "$probe" && cargo build --offline --release)
done
cmp "$work/baseline/Cargo.lock" "$work/candidate/Cargo.lock"
for run in 1 2 3; do
    variants=(baseline candidate)
    if [ "$run" = 2 ]; then variants=(candidate baseline); fi
    for variant in "${variants[@]}"; do
        mkdir "$work/$variant-$run"
        "$work/$variant/target/release/generator-probe" "$work/$variant-$run" > "$work/$variant-$run.csv"
    done
    diff -qr "$work/baseline-$run" "$work/candidate-$run"
done
