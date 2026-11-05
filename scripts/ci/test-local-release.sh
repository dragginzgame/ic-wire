#!/usr/bin/env bash
set -euo pipefail
# Real local adapters/runner and Git delivery; expensive gate/setup substituted.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset RELEASE_MAKE RELEASE_RESUME_CHECK_COMMAND CARGO_RELEASE_METADATA_EDITOR
export RELEASE_GROUP="${1:?select the real fixture group}"
plan="$(bash "$root/scripts/ci/cargo-release-group.sh" plan "$RELEASE_GROUP")"
primary="$(printf '%s' "$plan" | jq -er .primary)"
changelog_path="$(printf '%s' "$plan" | jq -er .changelog)"
# Reuse the offline-admitted real editor; fixture tool setup is substituted.
FIXTURE_RELEASE_EDITOR="${FIXTURE_RELEASE_EDITOR:-$(make --no-print-directory -s -C "$root" release-editor-check)}"
[[ "$FIXTURE_RELEASE_EDITOR" == /* && -f "$FIXTURE_RELEASE_EDITOR" && -x "$FIXTURE_RELEASE_EDITOR" ]] || exit 1
export FIXTURE_RELEASE_EDITOR
export RELEASE_DELIVERY=direct RELEASE_REMOTE=origin RELEASE_BRANCH=main
export RELEASE_CACHE_PREPARE=0
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH VALIDATION_LOG_DIR
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/local-release.XXXXXX")"
complete=false
status=0
finish() {
    local status="$1" line="$2" log latest='' evidence
    if [[ "$complete" == true ]]; then
        rm -rf "$fixture"
    else
        printf 'Release fixture failed at line %s; retained: %s\n' "$line" "$fixture" >&2
        mkdir -p "$root/target/validation-failures"
        evidence="$(mktemp -d "$root/target/validation-failures/release-fixture.XXXXXX")"
        for log in "$fixture"/*.log; do
            [[ -f "$log" ]] || continue
            cp "$log" "$evidence/"
            if [[ -z "$latest" || "$log" -nt "$latest" ]]; then latest="$log"; fi
        done
        if [[ -n "$latest" ]]; then
            printf 'Last fixture log (%s):\n' "${latest##*/}" >&2
            tail -n 80 "$latest" >&2
        fi
        printf 'Fixture logs copied for CI retention: %s\n' "$evidence" >&2
        [[ "$status" != 0 ]] || status=1
    fi
    exit "$status"
}
trap 'finish "$?" "$LINENO"' EXIT
mkdir "$fixture/repo"
# Copy current release/program inputs. Historical measurements are opaque to the
# substituted gates; keep them out of this disposable adapter fixture.
(cd "$root" && git ls-files --cached --others --exclude-standard -z -- . ':(exclude)docs/evidence/**' | while IFS= read -r -d '' path; do [[ ! -f "$path" ]] || printf '%s\0' "$path"; done | tar --null -T - -cf -) | tar -xf - -C "$fixture/repo"
export PATH="$root/.tools/host/bin:$root/.tools/rust/common/bin:$PATH"
cd "$fixture/repo"
cat >> Makefile <<'MAKE'

include fixture-gates.mk
MAKE
cat > fixture-gates.mk <<'MAKE'
.PHONY: install-tools tools-check release-editor-check
release-editor-check:
	@printf '%s\n' "$$FIXTURE_RELEASE_EDITOR"
install-tools tools-check:
	@printf '%s\n' '$@' >> .git/fixture-gates
release-push-check:
	@bash scripts/ci/ic-wire-release.sh push
	@test ! -f .git/hold-push
MAKE
# Substitute only Shared's exact expensive-gate dispatch. Actual Make parsing,
# source/group admission, metadata, tags, local push and resume stay real.
FIXTURE_REAL_MAKE="$(command -v make)"
export FIXTURE_REAL_MAKE
mkdir fixture-bin
cat > fixture-bin/make <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ $# == 5 && "$1" == --no-print-directory && "$2" == -C && "$4" == -- ]]; then
    goal="$5"
    if perl -MJSON::PP -e 'local $/; open my $f,"<",$ARGV[1] or die $!; my $p=decode_json(<$f>); exit((grep {$_ eq $ARGV[0]} @{$p->{stages}})?0:1)' "$goal" "$3/ci/release-validation.json"; then
        # Bash 3.2 does not apply errexit to this [[ ... ]] failure; exit explicitly.
        if [[ "$goal" == lint && -f "$3/.git/fail-validation" ]]; then exit 1; fi
        printf '%s\n' "$goal" >> "$3/.git/fixture-gates"
        exit 0
    fi
fi
exec "$FIXTURE_REAL_MAKE" "$@"
STUB
chmod +x fixture-bin/make
export PATH="$PWD/fixture-bin:$PATH"
# This fixture exercises patch delivery, independently of the consumer's pending
# patch/minor/major notes. Only the disposable copy receives synthetic notes;
# the real release preflight still checks the real changelog against its candidate.
previous="$(make --no-print-directory -s release-version)"
candidate="$(bash scripts/ci/next-release-version.sh "$previous" patch)"
printf '# Changelog\n\n## [%s]\n\n- Disposable release fixture.\n\n## [%s] - 2000-01-01\n\n- Fixture baseline.\n' "$candidate" "$previous" > "$changelog_path"
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
# Normal releases must ignore malformed historical plans and retain their evidence.
mkdir -p .git/release-state/broken.plan
printf 'retained historical plan\n' > .git/release-state/0.0.1.plan
# Dirty source must refuse before gate or version effects.
printf '\nfixture edit\n' >> README.md
if make release-patch > "$fixture/dirty.log" 2>&1; then exit 1; fi
[[ ! -e .git/fixture-gates && "$(make -s release-version)" == "$previous" ]]
grep -F 'commit or restore worktree changes' "$fixture/dirty.log" >/dev/null
git restore README.md
# Candidate conflict must refuse before tools or validation.
cp "$changelog_path" "$fixture/changelog"
printf '# Changelog\n\n## [99.0.0]\n' > "$changelog_path"
git add "$changelog_path"
git commit -qm 'Conflicting candidate'
if make release-patch > "$fixture/candidate.log" 2>&1; then exit 1; fi
[[ ! -e .git/fixture-gates ]]
cp "$fixture/changelog" "$changelog_path"
git add "$changelog_path"
git commit -qm 'Restore candidate'
source="$(git rev-parse HEAD)"
# Failed validation leaves metadata untouched and retains actual logger output.
touch .git/fail-validation
if make release-patch > "$fixture/failed.log" 2>&1; then exit 1; fi
[[ "$(make -s release-version)" == "$previous" && "$(git rev-parse HEAD)" == "$source" ]]
[[ -d target/release-validation ]]
[[ ! -e ".git/release-state/$primary-$candidate/validated" ]]
rm .git/fail-validation
touch .git/hold-push
if make release-patch > "$fixture/release.log" 2>&1; then exit 1; fi
# Interruption after tagging resumes only with the same validation evidence.
cp ".git/release-state/$primary-$candidate/validated" "$fixture/validated"
printf 'wrong source\n' > ".git/release-state/$primary-$candidate/validated"
if make release-resume "VERSION=$candidate" > "$fixture/receipt.log" 2>&1; then exit 1; fi
grep -F 'validation receipt missing or changed' "$fixture/receipt.log" >/dev/null
cp "$fixture/validated" ".git/release-state/$primary-$candidate/validated"
rm .git/hold-push
make release-resume "VERSION=$candidate" > "$fixture/recovered.log" 2>&1
[[ "$(make -s release-version)" == "$candidate" ]]
commit="$(git rev-parse HEAD)"
[[ "$(git log -1 --format=%s)" == "Release $RELEASE_GROUP $candidate" ]]
[[ "$(git --git-dir="$fixture/remote.git" rev-parse refs/heads/main)" == "$commit" ]]
[[ "$(git --git-dir="$fixture/remote.git" rev-parse "refs/tags/$RELEASE_GROUP-v$candidate^{}")" == "$commit" ]]
bash scripts/ci/cargo-release-group.sh files "$RELEASE_GROUP" | tr '\0' '\n' | sort > "$fixture/expected-files"
git diff-tree --no-commit-id --name-only -r HEAD | sort > "$fixture/changed-files"
cmp "$fixture/expected-files" "$fixture/changed-files"
[[ -z "$(git status --porcelain)" ]]
for package in ic-wire ic-wire-derive; do
    yq -p toml -o json '.' "crates/$package/Cargo.toml" | jq -e '.package.publish == ["crates-io"]' >/dev/null
done
yq -p toml -o json '.' crates/ic-wire-qualification/Cargo.toml | jq -e '.package.publish == false' >/dev/null
make release-resume "VERSION=$candidate" > "$fixture/resume.log" 2>&1
[[ "$(git rev-parse HEAD)" == "$commit" ]]
# Current-source ordering refuses disorder, while committed-source admission
# must read the selected release rather than the surrounding dirty worktree.
cp Cargo.toml "$fixture/grouped-manifest"
perl -0pi -e 's/^(proc-macro2[^\n]*\n)(quote[^\n]*\n)/$2$1/m' Cargo.toml
export RELEASE_VERSION="$candidate"
RELEASE_DATE="$(sed -n 's/^## \[[^]]*\] - \([0-9-]*\)$/\1/p' "$changelog_path" | head -n 1)"
export RELEASE_DATE
if RELEASE_COMMIT='' bash scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped > "$fixture/grouped-current.log" 2>&1; then exit 1; fi
RELEASE_COMMIT="$commit" bash scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped > "$fixture/grouped-committed.log" 2>&1
cmp Cargo.toml <(perl -0pe 's/^(proc-macro2[^\n]*\n)(quote[^\n]*\n)/$2$1/m' "$fixture/grouped-manifest")
cp "$fixture/grouped-manifest" Cargo.toml
[[ "$(cat .git/release-state/0.0.1.plan)" == 'retained historical plan' ]]
[[ -d .git/release-state/broken.plan ]]
# Make check-release invokes this fixture for each real configured group. The
# earlier synthetic endpoint policy is superseded: it omitted the now-real
# independently versioned endpoint member and duplicated these delivery checks.
# Exact changed-file custody plus the clean worktree checks above also prove
# unselected manifests/ledgers remain unchanged; no separate hash roster is needed.
complete=true
printf '%s\n' 'Local release passed: dirty/conflicting source refusal, retained failed validation, exact metadata/tag/push, resume and receipt rejection; no publication.'
