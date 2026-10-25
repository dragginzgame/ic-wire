#!/usr/bin/env bash
set -euo pipefail
evidence=$(cd -- "$(dirname -- "$0")" && pwd)
repo=$(git -C "$evidence" rev-parse --show-toplevel)
work=$(mktemp -d)
printf 'Evidence output: %s\n' "$work"
mkdir "$work/before" "$work/after"
git -C "$repo" archive c2adbfb47da6c180bc470c0d5204a35d7b61d574 | tar -x -C "$work/before"
cp -a "$work/before/." "$work/after/"
git -C "$work/after" apply "$evidence/candidate.patch"
for variant in baseline candidate; do
    source=before
    if [ "$variant" = candidate ]; then source=after; fi
    probe="$work/$variant"
    mkdir -p "$probe/crates/probe/src"
    cp "$evidence/probe.rs" "$probe/crates/probe/src/main.rs"
    cp "$evidence/Cargo.lock" "$probe/Cargo.lock"
    cp "$work/before/rust-toolchain.toml" "$probe/rust-toolchain.toml"
    cat > "$probe/Cargo.toml" <<EOF
[workspace]
members=["crates/probe"]
resolver="2"
[workspace.package]
version="0.0.0"
edition="2021"
[workspace.dependencies]
ic-wire={path="$work/$source/crates/ic-wire",features=["typescript"]}
serde_json="1"
[profile.release]
opt-level=3
lto=true
codegen-units=1
EOF
    cat > "$probe/crates/probe/Cargo.toml" <<'EOF'
[package]
name="advice-probe"
version.workspace=true
edition.workspace=true
publish=false
[features]
candidate=[]
[dependencies]
ic-wire.workspace=true
serde_json.workspace=true
EOF
    flags=()
    if [ "$variant" = candidate ]; then flags=(--features candidate); fi
    (cd "$probe" && cargo build --offline --locked --release "${flags[@]}")
done
# Compile both variants before timing; alternate the execution order.
for run in 1 2 3; do
    variants=(baseline candidate)
    if [ "$run" = 2 ]; then variants=(candidate baseline); fi
    for variant in "${variants[@]}"; do
        mkdir "$work/$variant-$run"
        "$work/$variant/target/release/advice-probe" "$work/$variant-$run" > "$work/$variant-$run.csv"
    done
    diff -qr "$work/baseline-$run" "$work/candidate-$run"
done
