#!/usr/bin/env bash
# Real group metadata editor only; no Git/index/tag/publication effects.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
root="$(cd "$here/../../.." && pwd -P)"
out="${1:?provide a fresh absolute output directory}"
case "$out" in /*) ;; *) exit 2;; esac
mkdir "$out"
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH"
export CARGO_RELEASE_METADATA_EDITOR="$(make --no-print-directory -s -C "$root" release-editor-check)"
export CARGO_RELEASE_METADATA_MAX_BYTES=8388608
unset RELEASE_COMMIT
mkdir "$out/base"
git -C "$root" archive "$(cat "$here/baseline-revision.txt")" | tar -xf - -C "$out/base"
patch -d "$out/base" -p1 < "$here/inputs.patch"
for group in core endpoint; do
    cp -R "$out/base" "$out/$group"
    cd "$out/$group"
    plan="$(bash scripts/ci/cargo-release-group.sh plan "$group")"
    export RELEASE_PREVIOUS="$(printf '%s' "$plan" | jq -er .previous)"
    export RELEASE_VERSION="$(bash scripts/ci/next-release-version.sh "$RELEASE_PREVIOUS" patch)"
    export RELEASE_DATE=2026-10-31
    ledger="$(printf '%s' "$plan" | jq -er .changelog)"
    printf '# Fixture\n\n## [%s]\n\n- Isolated editor qualification.\n\n## [%s] - 2000-01-01\n\n- Baseline.\n' "$RELEASE_VERSION" "$RELEASE_PREVIOUS" > "$ledger"
    bash scripts/ci/cargo-release-group.sh prepare "$group"
    bash scripts/ci/cargo-release-group.sh check "$group" '' --manifest-order-grouped
    python3 - "$out/base" "$PWD" "$group" "$RELEASE_VERSION" <<'PY'
import json,sys,tomllib
from pathlib import Path
before,after=map(Path,sys.argv[1:3]);group,next_version=sys.argv[3:];policy=json.loads((after/'ci/cargo-release-groups.json').read_text())
old=tomllib.loads((before/'Cargo.toml').read_text());new=tomllib.loads((after/'Cargo.toml').read_text())
for g in policy['groups']:
 for alias in g['dependencies']:
  expected=next_version if g['name']==group else old['workspace']['dependencies'][alias]['version']
  assert new['workspace']['dependencies'][alias]['version']==expected,(alias,expected)
 for name in g['members']:
  dep=next(d for alias,d in new['workspace']['dependencies'].items() if d.get('package',alias)==name)
  p=Path(dep['path'])/'Cargo.toml';a=tomllib.loads((after/p).read_text());b=tomllib.loads((before/p).read_text())
  assert a['package']['version']==(next_version if g['name']==group else b['package']['version'])
print(group,'selected member/alias versions updated; other group identities preserved')
PY
done
