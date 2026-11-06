#!/usr/bin/env bash
set -euo pipefail
# Real local adapters/runner and Git delivery; expensive gate/setup substituted.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
frozen=false
if [[ $# == 2 && "$1" == core && "$2" == --frozen ]]; then
    frozen=true
elif [[ $# != 1 ]]; then
    echo 'usage: test-local-release.sh core|endpoint [--frozen (core batch only)]' >&2
    exit 2
fi
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset CARGO_TARGET_DIR RELEASE_FROZEN_GROUPS RELEASE_FROZEN_PUBLISH_TARGET FROZEN_RELEASE_ACTIVE
unset FIXTURE_NORMAL_SOURCE FIXTURE_PUBLISH_CALLS FIXTURE_HOLD_PUBLICATION
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
	@printf '%s\n' '$@' >> "$$(git rev-parse --git-path fixture-gates)"
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
        gitdir="$(git -C "$3" rev-parse --absolute-git-dir)"
        if [[ "$goal" == lint && -f "$gitdir/fail-validation" ]]; then exit 1; fi
        printf '%s\n' "$goal" >> "$gitdir/fixture-gates"
        if [[ -n "${FIXTURE_NORMAL_SOURCE:-}" ]]; then
            if grep -q frozen-deferred "$3/Cargo.lock"; then exit 1; fi
            if [[ "$goal" == lint && "$RELEASE_GROUP" == core ]]; then
                printf '\n# frozen-deferred\n' >> "$FIXTURE_NORMAL_SOURCE/Cargo.lock"
            fi
        fi
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
if [[ "$frozen" == true ]]; then
    endpoint_previous="$(make --no-print-directory -s release-version RELEASE_GROUP=endpoint)"
    endpoint_candidate="$(bash scripts/ci/next-release-version.sh "$endpoint_previous" patch)"
    printf '# Changelog\n\n## [%s]\n\n- Disposable release fixture.\n\n## [%s] - 2000-01-01\n\n- Fixture baseline.\n' "$endpoint_candidate" "$endpoint_previous" > releases/endpoint/CHANGELOG.md
    # Keep real publication admission, selection and receipts; substitute only
    # the registry-effect command. This is not archive/build qualification.
    export FIXTURE_REAL_CARGO="$(command -v cargo)"
    cat > fixture-bin/cargo <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == publish ]]; then
    if grep -q frozen-deferred Cargo.lock; then exit 1; fi
    printf '%s|%s|%s\n' "$RELEASE_GROUP" "$PWD" "$*" >> "$FIXTURE_PUBLISH_CALLS"
    if [[ "$RELEASE_GROUP" == endpoint && -f "$FIXTURE_HOLD_PUBLICATION" ]]; then exit 23; fi
    exit 0
fi
exec "$FIXTURE_REAL_CARGO" "$@"
STUB
    chmod +x fixture-bin/cargo
    printf '\nlint:\n\t@:\n' >> fixture-gates.mk
fi
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
if [[ "$frozen" == true ]]; then
    export FIXTURE_NORMAL_SOURCE="$PWD" FIXTURE_PUBLISH_CALLS="$fixture/publication-calls"
    export FIXTURE_HOLD_PUBLICATION="$fixture/hold-publication"
    touch "$FIXTURE_HOLD_PUBLICATION"
    if make release-patch RELEASE_FROZEN_GROUPS=core,endpoint RELEASE_FROZEN_PUBLISH_TARGET=publish > "$fixture/frozen.log" 2>&1; then exit 1; fi
    intents=(.git/release-state/frozen/batch-*/intent.json)
    [[ ${#intents[@]} == 1 && -f "${intents[0]}" ]]
    intent="${intents[0]}"
    batch="$(basename "$(dirname "$intent")")"
    work="$PWD/target/release-frozen/$batch"
    gitdir="$(git -C "$work" rev-parse --absolute-git-dir)"
    jq -e '.phase == "publishing" and .index == 1' "$intent" >/dev/null
    [[ "$(git rev-parse HEAD)" == "$source" && "$(make -s release-version)" == "$previous" ]]
    [[ "$(git status --porcelain)" == ' M Cargo.lock' ]]
    grep -q frozen-deferred Cargo.lock
    if grep -q frozen-deferred "$work/Cargo.lock"; then exit 1; fi
    commit="$(git -C "$work" rev-parse HEAD)"
    [[ "$(git --git-dir="$fixture/remote.git" rev-parse refs/heads/main)" == "$commit" ]]
    for group in core endpoint; do
        version="$candidate"
        [[ "$group" == core ]] || version="$endpoint_candidate"
        [[ -f "$gitdir/release-state/$group-$version.plan" ]]
        [[ "$(git -C "$work" rev-parse "$group-v$version^{}")" == "$(git --git-dir="$fixture/remote.git" rev-parse "$group-v$version^{}")" ]]
    done
    cp "$FIXTURE_PUBLISH_CALLS" "$fixture/before-resume"
    if perl scripts/ci/run-frozen-release.pl --root "$PWD" --resume "$batch" > "$fixture/uncertain.log" 2>&1; then exit 1; fi
    grep -q 'publication outcome uncertain' "$fixture/uncertain.log"
    cmp "$FIXTURE_PUBLISH_CALLS" "$fixture/before-resume"
    # The fixture transport did no uploads, so not-published is known here.
    rm "$FIXTURE_HOLD_PUBLICATION"
    perl scripts/ci/run-frozen-release.pl --root "$PWD" --resume "$batch" --publication-result not-published > "$fixture/recovered.log" 2>&1
    jq -e '.phase == "complete" and .index == 2' "$intent" >/dev/null
    [[ "$(git -C "$work" rev-parse HEAD)" == "$commit" ]]
    printf 'core|%s|publish --locked --registry crates-io --package ic-wire-derive@%s --package ic-wire@%s\n' "$work" "$candidate" "$candidate" > "$fixture/expected-publication"
    for attempt in failed recovered; do
        printf 'endpoint|%s|publish --locked --registry crates-io --package ic-wire-endpoint-macros@%s --package ic-wire-endpoint@%s\n' "$work" "$endpoint_candidate" "$endpoint_candidate" >> "$fixture/expected-publication"
    done
    cmp "$FIXTURE_PUBLISH_CALLS" "$fixture/expected-publication"
    perl scripts/ci/run-frozen-release.pl --root "$PWD" --resume "$batch" > "$fixture/complete-resume.log" 2>&1
    cmp "$FIXTURE_PUBLISH_CALLS" "$fixture/expected-publication"
    [[ "$(git rev-parse HEAD)" == "$source" && "$(git status --porcelain)" == ' M Cargo.lock' ]]
    [[ -z "$(git -C "$work" status --porcelain)" ]]
    complete=true
    echo 'Frozen Wire hooks passed: both groups, deferred lock isolation, durable local Git delivery, uncertain publication refusal and idempotent recovery. Gates and upload substituted; no registry publication.'
    exit 0
fi
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
