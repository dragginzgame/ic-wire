#!/usr/bin/env bash
set -euo pipefail
# Real local adapters/runner and Git delivery; expensive gate/setup substituted.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset RELEASE_MAKE RELEASE_RESUME_CHECK_COMMAND
export RELEASE_DELIVERY=direct RELEASE_REMOTE=origin RELEASE_BRANCH=main
export RELEASE_CACHE_PREPARE=0
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH VALIDATION_LOG_DIR
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-wire-release.XXXXXX")"
complete=false
status=0
trap 'status=$?; if [[ "$complete" == true ]]; then rm -rf "$fixture"; else echo "Release fixture retained: $fixture" >&2; [[ "$status" != 0 ]] || status=1; fi; exit "$status"' EXIT
mkdir "$fixture/repo"
# Export current authored inputs, including this untracked adapter during adoption.
(cd "$root" && git ls-files --cached --others --exclude-standard -z | while IFS= read -r -d '' path; do [[ ! -f "$path" ]] || printf '%s\0' "$path"; done | tar --null -T - -cf -) | tar -xf - -C "$fixture/repo"
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH"
cd "$fixture/repo"
cat >> Makefile <<'MAKE'

include fixture-gates.mk
MAKE
cat > fixture-gates.mk <<'MAKE'
.PHONY: install-tools tools-check ci
install-tools tools-check:
	@printf '%s\n' '$@' >> .git/fixture-gates
ci:
	@test ! -f .git/fail-validation
	@printf '%s\n' ci >> .git/fixture-gates
release-push-check:
	@bash scripts/ci/ic-wire-release.sh push
	@test ! -f .git/hold-push
MAKE
# This fixture exercises patch delivery, independently of the consumer's pending
# patch/minor/major notes. Only the disposable copy receives synthetic notes;
# the real release preflight still checks the real changelog against its candidate.
previous="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
candidate="$(bash scripts/ci/next-release-version.sh "$previous" patch)"
printf '# Changelog\n\n## [%s]\n\n- Disposable release fixture.\n\n## [%s] - 2000-01-01\n\n- Fixture baseline.\n' "$candidate" "$previous" > CHANGELOG.md
git init -q -b main
git config user.name Fixture
git config user.email fixture@example.invalid
git config core.hooksPath /dev/null
git add -- .
git commit -qm 'Disposable release baseline'
git init -q --bare "$fixture/remote.git"
git remote add origin "$fixture/remote.git"
git push -q -u origin main
source="$(git rev-parse HEAD)"
# Dirty source must refuse before gate or version effects.
printf '\nfixture edit\n' >> README.md
if make release-patch > "$fixture/dirty.log" 2>&1; then exit 1; fi
[[ ! -e .git/fixture-gates && "$(make -s release-version)" == "$previous" ]]
grep -F 'README.md' "$fixture/dirty.log" >/dev/null
git restore README.md
# Candidate conflict must refuse before tools or validation.
cp CHANGELOG.md "$fixture/changelog"
printf '# Changelog\n\n## [99.0.0]\n' > CHANGELOG.md
git add CHANGELOG.md
git commit -qm 'Conflicting candidate'
if make release-patch > "$fixture/candidate.log" 2>&1; then exit 1; fi
[[ ! -e .git/fixture-gates ]]
cp "$fixture/changelog" CHANGELOG.md
git add CHANGELOG.md
git commit -qm 'Restore candidate'
source="$(git rev-parse HEAD)"
# Failed validation leaves metadata untouched and retains actual logger output.
touch .git/fail-validation
if make release-patch > "$fixture/failed.log" 2>&1; then exit 1; fi
[[ "$(make -s release-version)" == "$previous" && "$(git rev-parse HEAD)" == "$source" ]]
[[ -d ".git/release-state/ic-wire-$candidate/validation" ]]
[[ ! -e ".git/release-state/ic-wire-$candidate/validated" ]]
rm .git/fail-validation
touch .git/hold-push
if make release-patch > "$fixture/release.log" 2>&1; then exit 1; fi
# Interruption after tagging resumes only with the same validation evidence.
cp ".git/release-state/ic-wire-$candidate/validated" "$fixture/validated"
printf 'wrong source\n' > ".git/release-state/ic-wire-$candidate/validated"
if make release-resume "VERSION=$candidate" > "$fixture/receipt.log" 2>&1; then exit 1; fi
grep -F 'validation identity changed' "$fixture/receipt.log" >/dev/null
cp "$fixture/validated" ".git/release-state/ic-wire-$candidate/validated"
rm .git/hold-push
make release-resume "VERSION=$candidate" > "$fixture/recovered.log" 2>&1
[[ "$(make -s release-version)" == "$candidate" ]]
commit="$(git rev-parse HEAD)"
[[ "$(git log -1 --format=%s)" == "Release $candidate" ]]
[[ "$(git --git-dir="$fixture/remote.git" rev-parse refs/heads/main)" == "$commit" ]]
[[ "$(git --git-dir="$fixture/remote.git" rev-parse "refs/tags/v$candidate^{}")" == "$commit" ]]
[[ "$(git diff-tree --no-commit-id --name-only -r HEAD)" == $'CHANGELOG.md\nCargo.lock\nCargo.toml' ]]
[[ -z "$(git status --porcelain)" ]]
for package in ic-wire ic-wire-derive; do
    yq -p toml -o json '.' "crates/$package/Cargo.toml" | jq -e '.package.publish == ["crates-io"]' >/dev/null
done
yq -p toml -o json '.' crates/ic-wire-canister-evaluation/Cargo.toml | jq -e '.package.publish == false' >/dev/null
make release-resume "VERSION=$candidate" > "$fixture/resume.log" 2>&1
[[ "$(git rev-parse HEAD)" == "$commit" ]]
complete=true
printf '%s\n' 'Local release passed: dirty/conflicting source refusal, retained failed validation, exact metadata/tag/push, resume and receipt rejection; no publication.'
