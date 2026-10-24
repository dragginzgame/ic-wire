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
cp -R crates "$out/source/"
cp scripts/ci/evaluate-codecs.sh scripts/ci/evaluate-codec-matrix.sh "$out/source/scripts/ci/"
git rev-parse HEAD > "$out/source-head.txt"
git status --short > "$out/source-status.txt"
git diff --binary HEAD > "$out/source-changes.patch"
export GIT_DIR="$(git rev-parse --absolute-git-dir)" GIT_WORK_TREE="$root"
matrix_target="${CARGO_TARGET_DIR:-$root/target/codec-matrix-build}"
[[ "$matrix_target" == /* ]] || matrix_target="$root/$matrix_target"
export CARGO_TARGET_DIR="$matrix_target"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
export RUNS="${RUNS:-2}" SAMPLES="${SAMPLES:-11}" BATCH_MS="${BATCH_MS:-2}"
cat > "$out/cohorts.psv" <<'COHORTS'
standard||wire-native
toko|toko|wire-native
derive-1|derive-scale-1|wire-bounded
derive-10|derive-scale-10|wire-bounded
derive-100|derive-scale|wire-bounded
packed-i16|packed-i16|wire-packed-i16
packed-control|packed-i16-baseline|wire-native
packed-shared-1|packed-scale-shared,derive-scale-1|wire-bounded
packed-shared-10|packed-scale-shared,derive-scale-10|wire-bounded
packed-shared-100|packed-scale-shared|wire-bounded
packed-varied-1|packed-scale,derive-scale-1|wire-bounded
packed-varied-10|packed-scale,derive-scale-10|wire-bounded
packed-varied-100|packed-scale|wire-bounded
bulk|bulk|wire-bounded
bulk-control|bulk,bulk-baseline|wire-bounded
bulk-1|bulk-scale,derive-scale-1|wire-bounded
bulk-10|bulk-scale,derive-scale-10|wire-bounded
bulk-100|bulk-scale|wire-bounded
bulk-control-1|bulk-scale,bulk-baseline,derive-scale-1|wire-bounded
bulk-control-10|bulk-scale,bulk-baseline,derive-scale-10|wire-bounded
bulk-control-100|bulk-scale,bulk-baseline|wire-bounded
COHORTS
cd "$out/source"
while IFS='|' read -r name features primary; do
    printf 'Running %s (%s)\n' "$name" "$primary"
    CORPUS_FEATURE="$features" bash "$out/source/scripts/ci/evaluate-codecs.sh" "$out/$name" > "$out/$name.log" 2>&1
    printf 'Completed %s\n' "$name"
done < "$out/cohorts.psv"
printf 'Full matrix evidence: %s\n' "$out"
