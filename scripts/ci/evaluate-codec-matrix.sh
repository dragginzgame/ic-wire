#!/usr/bin/env bash
# Full qualification matrix. Cohorts are benchmark controls, not codec modes.
set -euo pipefail
entry="${BASH_SOURCE[0]}"
[[ "$entry" == /* ]] || entry="$PWD/$entry"
root="$(cd -P -- "${entry%/*}/../.." && pwd -P)"
cd "$root"
: "${POCKET_IC_BIN:?Set POCKET_IC_BIN to an explicitly prepared server}"
[[ -x "$POCKET_IC_BIN" ]] || { echo 'POCKET_IC_BIN must be executable' >&2; exit 1; }
out="${1:-$root/target/codec-matrix}"
[[ "$out" == /* ]] || out="$root/$out"
mkdir -p "${out%/*}"
mkdir "$out"
mkdir -p "$out/source/scripts/ci"
cp Cargo.toml Cargo.lock rust-toolchain.toml "$out/source/"
cp -R crates examples "$out/source/"
cp scripts/ci/evaluate-codecs.sh scripts/ci/evaluate-codec-matrix.sh scripts/ci/codec-cohorts.psv "$out/source/scripts/ci/"
git rev-parse HEAD > "$out/source-head.txt"
git status --short > "$out/source-status.txt"
git diff --binary HEAD > "$out/source-changes.patch"
export GIT_DIR="$(git rev-parse --absolute-git-dir)" GIT_WORK_TREE="$root"
matrix_target="${CARGO_TARGET_DIR:-$root/target/codec-matrix-build}"
[[ "$matrix_target" == /* ]] || matrix_target="$root/$matrix_target"
export CARGO_TARGET_DIR="$matrix_target"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export RUNS="${RUNS:-2}" SAMPLES="${SAMPLES:-11}" BATCH_MS="${BATCH_MS:-2}"
cp scripts/ci/codec-cohorts.psv "$out/cohorts.psv"
cd "$out/source"
while IFS='|' read -r name features primary; do
    [ "$name" != hotspots ] || continue
    printf 'Running %s (%s)\n' "$name" "$primary"
    CODEC_SOURCE_SNAPSHOT="$out/source" CORPUS_FEATURE="$features" bash "$out/source/scripts/ci/evaluate-codecs.sh" "$out/$name" > "$out/$name.log" 2>&1
    printf 'Completed %s\n' "$name"
done < "$out/cohorts.psv"
printf 'Full matrix evidence: %s\n' "$out"
