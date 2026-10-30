#!/usr/bin/env bash
# Compare the released baseline with this adoption, using separate Cargo targets.
set -euo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
out="${1:?new absolute output directory}"
case "$out" in /*) ;; *) echo 'output must be absolute' >&2; exit 1;; esac
: "${POCKET_IC_BIN:?select the instrumented server identified in README.md}"
mkdir "$out"
# A private clone supplies the Git identity expected by the maintained runner.
# It never checks out, stages or commits anything in the consumer working tree.
git clone --quiet --no-hardlinks --no-checkout "$repo" "$out/baseline-source"
git -C "$out/baseline-source" checkout --quiet --detach e026eb4c8a7a71d4e74b459d774a83490c351762
bash "$out/baseline-source/scripts/ci/evaluate-endpoint-matrix.sh" "$out/baseline"
bash "$repo/scripts/ci/evaluate-endpoint-matrix.sh" "$out/candidate"
node "$repo/docs/evidence/platform-090-adoption/analyze.mjs" "$out/baseline" "$out/candidate" "$out"
