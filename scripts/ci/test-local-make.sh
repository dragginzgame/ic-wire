#!/usr/bin/env bash
set -euo pipefail
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
export RELEASE_DELIVERY=direct RELEASE_REMOTE=origin RELEASE_BRANCH=main

# Consumer qualification only. Git/index effects stay in this disposable export.
# Requires prepared development Rust, pinned cargo-sort, Git, Bash and GNU Make.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-wire-make.XXXXXX")"
# Make's CURDIR is physical and normalized; compare the same path identity.
fixture="$(cd "$fixture" && pwd -P)"
complete=false
trap 'status=$?; if [[ "$complete" == true ]]; then rm -rf "$fixture"; else printf "Local Make fixture retained: %s\n" "$fixture" >&2; [[ "$status" != 0 ]] || status=1; fi; exit "$status"' EXIT
mkdir "$fixture/repo" "$fixture/bin"
paths=(Makefile Cargo.toml Cargo.lock rust-toolchain.toml .gitignore .shared-tooling.snapshot docs/runtime.md .github/workflows/ci.yml)
paths+=(scripts/ci/fuzz-runtime.sh scripts/ci/check-diagnostics.sh scripts/ci/check-endpoint.sh scripts/ci/check-endpoint-ic.sh)
# Include authored crate inputs automatically, including new untracked modules.
# Git exclusions keep ignored build/cache output out of the disposable export.
git -C "$root" ls-files --cached --others --exclude-standard -z -- crates/ examples/endpoint/ > "$fixture/crate-paths"
while IFS= read -r -d '' path; do
    [[ ! -f "$root/$path" ]] || paths+=("$path")
done < "$fixture/crate-paths"
while IFS=$'\t' read -r record _ _ path; do
    [[ "$record" == file ]] || continue
    paths+=("$path")
done < "$root/.shared-tooling.snapshot"
for path in "${paths[@]}"; do
    mkdir -p "$fixture/repo/$(dirname "$path")"
    cp -pP "$root/$path" "$fixture/repo/$path"
done

# Check the real include graph and release admission, with an effect sentinel.
make --no-print-directory -C "$fixture/repo" help > "$fixture/help.log"
[[ ! -e "$fixture/repo/.tools" ]]
cat > "$fixture/repo/scripts/ci/run-release.sh" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$*" > "$RELEASE_SENTINEL"
exit 99
STUB
export RELEASE_SENTINEL="$fixture/release-ran"
for target in release-patch release-minor release-major release-resume; do
    status=0
    make --no-print-directory -C "$fixture/repo" "$target" VERSION=0.1.0 > "$fixture/$target.log" 2>&1 || status=$?
    [[ "$status" == 2 && -e "$RELEASE_SENTINEL" ]]
    if [[ "$target" == release-resume ]]; then expected="resume 0.1.0 origin main"; else expected="${target#release-} origin main"; fi
    [[ "$(cat "$RELEASE_SENTINEL")" == "$expected" ]]
    rm "$RELEASE_SENTINEL"
done

# Admit the actual local Cargo recipes without compiling or running tests.
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf '%s|%s|%s\n' "${RUSTUP_TOOLCHAIN:-development}" "${CARGO_TARGET_DIR:-}" "$*" >> "$CARGO_CALLS"
STUB
chmod +x "$fixture/bin/cargo"
export CARGO_CALLS="$fixture/cargo-calls"
for target in build check test lint check-wasm check-msrv; do
    PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" "$target"
done
cat > "$fixture/expected-calls" <<EOF
development|$fixture/repo/target|build --locked -p ic-wire
development|$fixture/repo/target|check --locked -p ic-wire --all-targets
development|$fixture/repo/target|test --locked -p ic-wire
development|$fixture/repo/target|clippy --locked -p ic-wire -p ic-wire-derive -p ic-wire-endpoint --all-targets -- -D warnings
development|$fixture/repo/target|check --locked -p ic-wire --lib --target wasm32-unknown-unknown
1.91.0|$fixture/repo/target|check --locked -p ic-wire --lib
1.91.0|$fixture/repo/target|check --locked -p ic-wire --lib --target wasm32-unknown-unknown
EOF
# The ambient development selection may be explicit (for example in CI).
sed "s/^development|/${RUSTUP_TOOLCHAIN:-development}|/" "$fixture/expected-calls" > "$fixture/expected-environment"
cmp "$fixture/expected-environment" "$CARGO_CALLS"

# Real workspace metadata and Git identity; Cargo upload remains a sentinel.
export FIXTURE_CARGO_METADATA="$fixture/publication-metadata.json"
(cd "$fixture/repo" && cargo metadata --no-deps --locked --offline --format-version 1) > "$FIXTURE_CARGO_METADATA"
git -C "$fixture/repo" init -q
git -C "$fixture/repo" add -- .
git -C "$fixture/repo" -c user.name=Fixture -c user.email=fixture@example.invalid -c core.hooksPath=/dev/null commit -qm 'Disposable publication baseline'
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf '%s|%s|%s\n' "${RUSTUP_TOOLCHAIN:-development}" "${CARGO_TARGET_DIR:-}" "$*" >> "$CARGO_CALLS"
case "$1" in
    metadata) cat "$FIXTURE_CARGO_METADATA" ;;
    clippy) exit "${FIXTURE_LINT_STATUS:-0}" ;;
    publish) printf 'Substituted Cargo publication\n'; exit "${FIXTURE_PUBLISH_STATUS:-0}" ;;
esac
STUB

# All three commands retain the lint gate, including parallel Make. Metadata
# admission may precede lint; neither packaging nor upload can follow its failure.
for target in publish publish-check check-package; do
    : > "$CARGO_CALLS"
    if FIXTURE_LINT_STATUS=41 PATH="$fixture/bin:$PATH" make -j4 --no-print-directory -C "$fixture/repo" "$target" > "$fixture/$target-lint.log" 2>&1; then exit 1; fi
    grep -F '|clippy --locked -p ic-wire -p ic-wire-derive -p ic-wire-endpoint --all-targets -- -D warnings' "$CARGO_CALLS" >/dev/null
    if grep -Eq '\|(publish|package) ' "$CARGO_CALLS"; then exit 1; fi
done

# Both defaults and explicitly reconciled subsets resolve exact workspace
# versions, retain the registry and dispatch once; publish-check is always dry.
for target in publish publish-check; do
    for selection in default 'ic-wire-derive ic-wire' ic-wire ic-wire-derive; do
        : > "$CARGO_CALLS"
        args=()
        packages="$selection"
        if [[ "$selection" == default ]]; then packages='ic-wire-derive ic-wire ic-wire-endpoint'; else args=("PACKAGE=$selection"); fi
        PATH="$fixture/bin:$PATH" make -j4 --no-print-directory -C "$fixture/repo" "$target" ${args[@]+"${args[@]}"} > "$fixture/publication-selection.log" 2>&1
        [[ "$(wc -l < "$CARGO_CALLS")" == 3 ]]
        expected='publish --locked --registry crates-io'
        for package in $packages; do version="$(jq -r --arg name "$package" '.packages[] | select(.name == $name) | .version' "$FIXTURE_CARGO_METADATA")"; expected+=" --package $package@$version"; done
        [[ "$target" != publish-check ]] || expected+=' --dry-run'
        [[ "$(tail -n 1 "$CARGO_CALLS")" == "${RUSTUP_TOOLCHAIN:-development}|$fixture/repo/target|$expected" ]]
    done
done
for selection in '' ic-wire-qualification 'ic-wire ic-wire'; do
    : > "$CARGO_CALLS"
    if PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" publish "PACKAGE=$selection" > "$fixture/publish-invalid.log" 2>&1; then exit 1; fi
    [[ "$(wc -l < "$CARGO_CALLS")" == 1 ]]
    grep -F '|metadata --no-deps --locked --offline --format-version 1' "$CARGO_CALLS" >/dev/null
done
: > "$CARGO_CALLS"
if PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" publish PUBLISH_PACKAGES=ic-wire > "$fixture/retired-selector.log" 2>&1; then exit 1; fi
[[ ! -s "$CARGO_CALLS" ]]
grep -F 'PUBLISH_PACKAGES was replaced by PACKAGE' "$fixture/retired-selector.log" >/dev/null

# A failed attempt stays failed with one dispatch and retained status; no retry.
: > "$CARGO_CALLS"
if FIXTURE_PUBLISH_STATUS=37 PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" publish PACKAGE=ic-wire > "$fixture/publish-failed.log" 2>&1; then exit 1; fi
[[ "$(grep -c '|publish ' "$CARGO_CALLS")" == 1 ]]
grep -rlx 37 "$fixture/repo/target/publication" | grep '/cargo-status$' >/dev/null
# Dirty source is refused before the simulated Cargo upload as well.
printf '\nDirty publication fixture.\n' >> "$fixture/repo/docs/runtime.md"
: > "$CARGO_CALLS"
if PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" publish > "$fixture/publish-dirty.log" 2>&1; then exit 1; fi
if grep -q '|publish ' "$CARGO_CALLS"; then exit 1; fi
git -C "$fixture/repo" restore docs/runtime.md

# Verify the actual CI recipes partition the complete roster. Substitute only
# dispatch: no broad gate or release effect runs in this disposable checkout.
cat > "$fixture/repo/scripts/ci/run-validation-targets.sh" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$@" > "$CI_SELECTION"
exit "${CI_FIXTURE_STATUS:-0}"
STUB
for target in ci ci-core ci-typescript ci-benchmarks; do
    export CI_SELECTION="$fixture/$target-selection"
    make -j4 --no-print-directory -C "$fixture/repo" "$target" IC_WIRE_CI_TARGETS=ignored > "$fixture/$target.log" 2>&1
    [[ "$(head -n 1 "$CI_SELECTION")" == --fail-fast ]]
    [[ "$(sed -n '2p' "$CI_SELECTION")" == tools-check ]]
    tail -n +2 "$CI_SELECTION" > "$fixture/$target-roster"
    if CI_FIXTURE_STATUS=37 make -j4 --no-print-directory -C "$fixture/repo" "$target" > "$fixture/$target-failed.log" 2>&1; then exit 1; fi
done
grep -Ev '^(check-benchmarks|check-typescript)$' "$fixture/ci-roster" > "$fixture/expected-core"
cmp "$fixture/expected-core" "$fixture/ci-core-roster"
printf '%s\n' tools-check check-typescript > "$fixture/expected-typescript"
printf '%s\n' tools-check check-benchmarks > "$fixture/expected-benchmarks"
cmp "$fixture/expected-typescript" "$fixture/ci-typescript-roster"
cmp "$fixture/expected-benchmarks" "$fixture/ci-benchmarks-roster"

# Each hosted suite uses the owning Make recipe, independent native runners and
# unique failure artifacts. Keep the old scaffold check names as final gates.
yq -o=json '.' "$fixture/repo/.github/workflows/ci.yml" > "$fixture/workflow.json"
jq -e '
    .jobs.validate as $job | .jobs.scaffold as $gate |
    ($job.strategy.matrix.suite == ["core", "typescript", "benchmarks"]) and
    ($job.strategy.matrix.os == ["ubuntu-24.04", "macos-15", "macos-15-intel"]) and
    ($job["runs-on"] == "${{ matrix.os }}") and
    ($job.needs == null) and ($job.strategy["fail-fast"] == false) and
    ([$job.steps[] | select(.run == "make \"ci-$CI_GROUP\"")] | length == 1) and
    ([$job.steps[] | select(.run == "make \"ci-$CI_GROUP\"")][0].env.CI_GROUP == "${{ matrix.suite }}") and
    ([$job.steps[] | select(.uses == "./.github/actions/retain-failure-evidence")][0].with.name |
        contains("matrix.suite") and contains("matrix.os")) and
    ($gate.needs == "validate") and ($gate.if == "${{ always() }}") and
    ($gate.name == null) and ($gate.strategy.matrix.os == $job.strategy.matrix.os) and
    ($gate.steps[0].env.VALIDATION_RESULT == "${{ needs.validate.result }}") and
    ($gate.steps[0].run == "test \"$VALIDATION_RESULT\" = success")
' "$fixture/workflow.json" > /dev/null
for result in success failure cancelled skipped; do
    status=0
    VALIDATION_RESULT="$result" bash -c 'test "$VALIDATION_RESULT" = success' || status=$?
    if [[ "$result" == success ]]; then [[ "$status" == 0 ]]; else [[ "$status" != 0 ]]; fi
done
cp -p "$root/scripts/ci/run-validation-targets.sh" "$fixture/repo/scripts/ci/run-validation-targets.sh"

# Restore the reviewed runner before exercising the real formatting hook.
cp -p "$root/scripts/ci/run-release.sh" "$fixture/repo/scripts/ci/run-release.sh"
git -C "$fixture/repo" init -q
git -C "$fixture/repo" add -- "${paths[@]}"
make --no-print-directory -C "$fixture/repo" install-hooks > "$fixture/install-hooks.log"
[[ "$(git -C "$fixture/repo" config --local core.hooksPath)" == .githooks ]]
(
    cd "$fixture/repo"
    # The consumer's grouped policy must preserve roles and reject disorder
    # inside a group, using the same commands as its formatting hook.
    cp Cargo.toml "$fixture/grouped-before"
    make fmt > "$fixture/grouped-write.log" 2>&1
    make fmt >> "$fixture/grouped-write.log" 2>&1
    cmp Cargo.toml "$fixture/grouped-before"
    perl -0pi -e 's/^(proc-macro2[^\n]*\n)(quote[^\n]*\n)/$2$1/m' Cargo.toml
    if make fmt-check > "$fixture/grouped-unsorted.log" 2>&1; then exit 1; fi
    make fmt > "$fixture/grouped-repair.log" 2>&1
    cmp Cargo.toml "$fixture/grouped-before"
    bash .githooks/pre-commit > "$fixture/initial-hook.log" 2>&1
    git -c user.name=Fixture -c user.email=fixture@example.invalid -c core.hooksPath=/dev/null commit -qm 'Disposable formatting baseline'
    printf '\nconst _:u8=1;\n' >> crates/ic-wire/src/lib.rs
    git add -- crates/ic-wire/src/lib.rs
    printf '\nUnrelated fixture edit.\n' >> DRAGGINZGAME.md
    cp DRAGGINZGAME.md "$fixture/unrelated-before"
    bash .githooks/pre-commit > "$fixture/selected-hook.log" 2>&1
    grep -F 'const _: u8 = 1;' crates/ic-wire/src/lib.rs >/dev/null
    cmp "$fixture/unrelated-before" DRAGGINZGAME.md
    [[ "$(git diff --cached --name-only)" == crates/ic-wire/src/lib.rs ]]
    printf '\n// Unstaged fixture edit.\n' >> crates/ic-wire/src/lib.rs
    git diff --binary > "$fixture/unstaged-before"
    git diff --cached --binary > "$fixture/staged-before"
    status=0
    bash .githooks/pre-commit > "$fixture/partial-hook.log" 2>&1 || status=$?
    [[ "$status" != 0 ]]
    grep -F 'partially staged file' "$fixture/partial-hook.log" >/dev/null
    git diff --binary > "$fixture/unstaged-after"
    git diff --cached --binary > "$fixture/staged-after"
    cmp "$fixture/unstaged-before" "$fixture/unstaged-after"
    cmp "$fixture/staged-before" "$fixture/staged-after"
)
complete=true
printf '%s\n' 'Local Make routing, release routing and native formatting hook passed (disposable Git fixture).'
