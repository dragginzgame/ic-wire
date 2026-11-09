#!/usr/bin/env bash
set -euo pipefail
case_selection=all
if [[ $# != 0 ]]; then
    [[ $# == 2 && "$1" == --case ]] || { echo 'usage: test-local-make.sh [--case validation|release-routing|setup]' >&2; exit 2; }
    case "$2" in
        validation|release-routing|setup) case_selection="$2" ;;
        *) echo 'usage: test-local-make.sh [--case validation|release-routing|setup]' >&2; exit 2 ;;
    esac
fi
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES RELEASE_GROUP WIRE_LINT_GROUP
unset CARGO_TARGET_DIR RELEASE_GROUPS RELEASE_PUBLISH_TARGET RELEASE_BATCH_ACTIVE RELEASE_SETUP_TARGET RELEASE_SETUP_MODE
export RELEASE_DELIVERY=direct RELEASE_REMOTE=origin RELEASE_BRANCH=main

# Consumer qualification only. Git/index effects stay in this disposable export.
# Requires prepared development Rust, pinned cargo-sort, Git, Bash and GNU Make.
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/local-make.XXXXXX")"
# Make's CURDIR is physical and normalized; compare the same path identity.
fixture="$(cd "$fixture" && pwd -P)"
compiler="$(bash "$root/scripts/ci/read-cargo-workspace-rust-version.sh" "$root/Cargo.toml")"
export WIRE_FIXTURE_COMPILER="$compiler"
complete=false
trap 'status=$?; if [[ "$complete" == true ]]; then rm -rf "$fixture"; else printf "Local Make fixture retained: %s\n" "$fixture" >&2; [[ "$status" != 0 ]] || status=1; fi; exit "$status"' EXIT
mkdir "$fixture/repo" "$fixture/bin"
paths=(CHANGELOG.md ci/cargo-release-groups.json Makefile Cargo.toml Cargo.lock rust-toolchain.toml .gitignore .toko-tooling.snapshot docs/runtime.md .github/workflows/ci.yml)
paths+=(ci/release-validation.json scripts/ci/wire-validation-roster.pl scripts/ci/wire-release-validation.mjs scripts/ci/check-release-validation.mjs scripts/ci/rust-policy.sh scripts/ci/ic-wire-release.sh scripts/ci/fuzz-runtime.sh scripts/ci/check-diagnostics.sh scripts/ci/check-endpoint.sh scripts/ci/check-endpoint-ic.sh)
paths+=(scripts/ci/check-lint.sh scripts/ci/prepare-browser-tools.sh)
paths+=(scripts/ci/prepare-wire-metadata.sh)
paths+=(scripts/ci/check-testkit-boundary.mjs scripts/ci/check-testkit-boundary.test.mjs)
paths+=(scripts/ci/check-compatible-consumers.sh)
paths+=(scripts/benchmarks/generator/mod.rs scripts/dev/inspect-wire.mjs)
paths+=(scripts/ci/check-repository-documentation.sh)
paths+=(ci/candid-boundary.json)
# Include authored crate inputs automatically, including new untracked modules.
# Git exclusions keep ignored build/cache output out of the disposable export.
git -C "$root" ls-files --cached --others --exclude-standard -z -- crates/ examples/ qualification/ releases/ > "$fixture/crate-paths"
while IFS= read -r -d '' path; do
    [[ ! -f "$root/$path" ]] || paths+=("$path")
done < "$fixture/crate-paths"
while IFS=$'\t' read -r record _ _ path; do
    [[ "$record" == file ]] || continue
    paths+=("$path")
done < "$root/.toko-tooling.snapshot"
for path in "${paths[@]}"; do
    mkdir -p "$fixture/repo/$(dirname "$path")"
    cp -pP "$root/$path" "$fixture/repo/$path"
done

# New release preparation also checks the actual repository root for source cleanup.
git -C "$fixture/repo" init -q

# Exercise real root selection admission before substituting any execution.
make --no-print-directory -C "$fixture/repo" toko-tooling-check > "$fixture/tooling-valid.log" 2>&1
cp "$fixture/repo/Cargo.lock" "$fixture/tooling-lock-before"
perl -0777 -pe 's/(name = "toko-tooling"\nversion = "[^"\n]+"\nsource = "[^"\n]+"\nchecksum = ")[0-9a-f]{64}/$1 . ("0" x 64)/e' < "$fixture/tooling-lock-before" > "$fixture/repo/Cargo.lock"
for target in ci ci-core ci-typescript ci-benchmarks publish publish-check check-package release-batch-resume release-batch-reconcile; do
    if make -j4 --no-print-directory -C "$fixture/repo" "$target" RELEASE_GROUP=core BATCH=batch-fixture > "$fixture/$target-tooling-drift.log" 2>&1; then exit 1; fi
    grep -F 'Cargo.lock selects different toko-tooling bytes' "$fixture/$target-tooling-drift.log" >/dev/null
done
if (cd "$fixture/repo" && RELEASE_GROUP=core bash scripts/ci/ic-wire-release.sh preflight) > "$fixture/adapter-tooling-drift.log" 2>&1; then exit 1; fi
grep -F 'Cargo.lock selects different toko-tooling bytes' "$fixture/adapter-tooling-drift.log" >/dev/null
cp "$fixture/tooling-lock-before" "$fixture/repo/Cargo.lock"
# Routing probes deliberately replace exported runners. Admission was exercised
# above on pristine payloads; substitute only that gate for subsequent dispatch.
cat > "$fixture/repo/fixture-admission.mk" <<'STUB'
.PHONY: toko-tooling-check
toko-tooling-check:
	@:
STUB
printf '\ninclude fixture-admission.mk\n' >> "$fixture/repo/Makefile"

if [[ "$case_selection" == setup ]]; then
    cat >> "$fixture/repo/fixture-admission.mk" <<'STUB'
print-local-setup:
	@printf '%s\n' '$(LOCAL_TOOL_INSTALL_TARGETS)' '$(LOCAL_TOOL_CHECK_TARGETS)'
STUB
    make --no-print-directory -s -C "$fixture/repo" print-local-setup > "$fixture/local-setup"
    if grep -E 'install-release-editor|release-editor-check' "$fixture/local-setup"; then exit 1; fi
    cat > "$fixture/repo/scripts/ci/cargo-release-preflight.sh" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$*" > "$SETUP_CALLS"
STUB
    export SETUP_CALLS="$fixture/release-setup"
    for mode in prepare check; do
        make --no-print-directory -s -C "$fixture/repo" release-candidate-setup \
            RELEASE_GROUP=core "RELEASE_SETUP_MODE=$mode" RUST_POLICY=env
        grep -F -- '--prepare install-release-editor --prepare release-editor-check' "$SETUP_CALLS" >/dev/null
        grep -F -- '--check tools-check --check release-editor-check' "$SETUP_CALLS" >/dev/null
        grep -F -- '--pin qualification/Cargo.lock' "$SETUP_CALLS" >/dev/null
    done
    complete=true
    echo 'Setup split passed: ordinary setup excludes the editor; release prepare/check selects it explicitly.'
    exit 0
fi

check_ci_routing() {
    # Roster admission must work before bootstrap tools exist, including system Make.
    make --no-print-directory -C "$fixture/repo" help > "$fixture/bootstrap.log"
    [[ ! -e "$fixture/repo/.tools" ]]
    # Prove the actual consumer boundary target includes browser CBOR imports.
    # This is parser input in a disposable tree, never compiled or executed.
    make --no-print-directory -C "$fixture/repo" check-boundary > "$fixture/boundary.log" 2>&1
    printf '%s\n' "import { decode } from 'cbor-x';" > "$fixture/repo/examples/browser/boundary-probe.ts"
    if make --no-print-directory -C "$fixture/repo" check-boundary > "$fixture/boundary-refused.log" 2>&1; then exit 1; fi
    grep -F 'direct SDK/CBOR import' "$fixture/boundary-refused.log" >/dev/null
    rm "$fixture/repo/examples/browser/boundary-probe.ts"
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
    printf '%s\n' tools-check check-rust-policy check-typescript > "$fixture/expected-typescript"
    printf '%s\n' tools-check check-rust-policy check-benchmarks > "$fixture/expected-benchmarks"
    cmp "$fixture/expected-typescript" "$fixture/ci-typescript-roster"
    cmp "$fixture/expected-benchmarks" "$fixture/ci-benchmarks-roster"

    # A new declared stage reaches full CI without a second Make roster edit.
    cp "$fixture/repo/ci/release-validation.json" "$fixture/original-policy"
    perl -MJSON::PP -e 'local $/; my $p=decode_json(<>); push @{$p->{stages}}, "new-stage"; print JSON::PP->new->canonical->encode($p), "\n"' < "$fixture/original-policy" > "$fixture/repo/ci/release-validation.json"
    CI_SELECTION="$fixture/expanded-selection" make --no-print-directory -C "$fixture/repo" ci > "$fixture/expanded.log" 2>&1
    grep -Fx new-stage "$fixture/expanded-selection" >/dev/null
    cp "$fixture/original-policy" "$fixture/repo/ci/release-validation.json"

    # Partial stdout cannot turn a failed roster reader into an admitted CI gate.
    cp -p "$fixture/repo/scripts/ci/wire-validation-roster.pl" "$fixture/original-roster"
    printf 'print "tools-check\\n"; exit 37;\n' > "$fixture/repo/scripts/ci/wire-validation-roster.pl"
    for target in help ci; do
        if CI_SELECTION="$fixture/refused-selection" make --no-print-directory -C "$fixture/repo" "$target" > "$fixture/refused-$target.log" 2>&1; then exit 1; fi
        grep -F 'Wire validation roster admission failed' "$fixture/refused-$target.log" >/dev/null
        [[ ! -e "$fixture/refused-selection" ]]
    done
    cp -p "$fixture/original-roster" "$fixture/repo/scripts/ci/wire-validation-roster.pl"

    # Workflow topology, failure artifacts and required gates have one owning fixture:
    # scripts/ci/check-ci-selection.mjs. This fixture owns Make dispatch and hooks.
    cp -p "$root/scripts/ci/run-validation-targets.sh" "$fixture/repo/scripts/ci/run-validation-targets.sh"

}

if [[ "$case_selection" == validation ]]; then
    check_ci_routing
    complete=true
    echo "Local Make validation routing passed (disposable fixture; no application gates or releases)."
    exit 0
fi

# Check the real include graph and release admission, with an effect sentinel.
make --no-print-directory -C "$fixture/repo" help > "$fixture/help.log"
[[ ! -e "$fixture/repo/.tools" ]]
# Exercise the real group authority and ledger inventory before effect stubs.
for group in core endpoint; do
    expected="$(cd "$root" && bash scripts/ci/cargo-release-group.sh version "$group")"
    actual="$(PATH="$root/.tools/host/bin:$PATH" make --no-print-directory -s -C "$fixture/repo" release-version "RELEASE_GROUP=$group")"
    [[ "$actual" == "$expected" && ! -e "$fixture/repo/.tools" ]]
done
cat > "$fixture/repo/scripts/ci/run-release.sh" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$*" > "$RELEASE_SENTINEL"
exit 99
STUB
export RELEASE_SENTINEL="$fixture/release-ran"
for group_selection in missing empty unknown; do
    args=()
    case "$group_selection" in empty) args=(RELEASE_GROUP=) ;; unknown) args=(RELEASE_GROUP=unknown) ;; esac
    for target in release-patch release-minor release-major release-resume release-version publish publish-check; do
        if make --no-print-directory -C "$fixture/repo" "$target" ${args[@]+"${args[@]}"} > "$fixture/refuse-$group_selection-$target.log" 2>&1; then exit 1; fi
        [[ ! -e "$RELEASE_SENTINEL" && ! -e "$fixture/repo/.tools" ]]
        grep -F 'Select RELEASE_GROUP=core or endpoint explicitly' "$fixture/refuse-$group_selection-$target.log" >/dev/null
    done
done
cat > "$fixture/repo/scripts/ci/run-release-batch.pl" <<'STUB'
use strict;
use warnings;
die "normal Cargo target leaked into launcher\n" if exists $ENV{CARGO_TARGET_DIR};
open my $out, '>', $ENV{RELEASE_SENTINEL} or die $!;
print {$out} join(' ', @ARGV), "\n";
close $out or die $!;
exit 99;
STUB
# Default single-group selection must capture without an opt-in roster.
for group in core endpoint; do
    for target in release-patch release-minor release-major release-resume; do
        status=0
        make --no-print-directory -C "$fixture/repo" "$target" "RELEASE_GROUP=$group" VERSION=0.1.0 > "$fixture/$group-$target.log" 2>&1 || status=$?
        [[ "$status" == 2 && -e "$RELEASE_SENTINEL" ]]
        if [[ "$target" == release-resume ]]; then expected="resume 0.1.0 origin main";
        else expected="--root $fixture/repo --groups $group --kind ${target#release-} --remote origin --branch main --delivery direct --setup-target release-candidate-setup"; fi
        [[ "$(cat "$RELEASE_SENTINEL")" == "$expected" ]]
        rm "$RELEASE_SENTINEL"
    done
done
# Old public selectors must refuse, never silently narrow or publish a batch.
for retired in RELEASE_FROZEN_GROUPS=core RELEASE_FROZEN_PUBLISH_TARGET=publish FROZEN_RELEASE_ACTIVE=1; do
    if make --no-print-directory -C "$fixture/repo" release-patch RELEASE_GROUP=core "$retired" > "$fixture/retired.log" 2>&1; then exit 1; fi
    grep -F 'Retired release settings' "$fixture/retired.log" >/dev/null
    [[ ! -e "$RELEASE_SENTINEL" ]]
done
for roster in core endpoint core,endpoint; do
    group="${roster%%,*}"
    status=0
    make --no-print-directory -C "$fixture/repo" release-patch "RELEASE_GROUP=$group" \
        "RELEASE_GROUPS=$roster" > "$fixture/frozen-$group.log" 2>&1 || status=$?
    [[ "$status" == 2 && "$(cat "$RELEASE_SENTINEL")" == "--root $fixture/repo --groups $roster --kind patch --remote origin --branch main --delivery direct --setup-target release-candidate-setup" ]]
    rm "$RELEASE_SENTINEL"
done
for roster in unknown endpoint,core core,core 'core endpoint' core,unknown; do
    if make --no-print-directory -C "$fixture/repo" release-patch RELEASE_GROUP=core \
        "RELEASE_GROUPS=$roster" > "$fixture/frozen-invalid.log" 2>&1; then exit 1; fi
    [[ ! -e "$RELEASE_SENTINEL" ]]
done
if make --no-print-directory -C "$fixture/repo" release-patch RELEASE_GROUP=endpoint \
    RELEASE_GROUPS=core,endpoint > "$fixture/frozen-mismatch.log" 2>&1; then exit 1; fi
[[ ! -e "$RELEASE_SENTINEL" ]]
for source in environment command; do
    args=()
    if [[ "$source" == command ]]; then args=(CARGO_TARGET_DIR=caller-target); fi
    if CARGO_TARGET_DIR=caller-target make --no-print-directory -C "$fixture/repo" \
        release-patch RELEASE_GROUP=core RELEASE_GROUPS=core \
        ${args[@]+"${args[@]}"} > "$fixture/frozen-target-$source.log" 2>&1; then exit 1; fi
    [[ ! -e "$RELEASE_SENTINEL" ]]
    grep -F 'Unset CARGO_TARGET_DIR' "$fixture/frozen-target-$source.log" >/dev/null
done
# The candidate uses its ordinary runner and its own Cargo target, not a nested batch.
status=0
make --no-print-directory -C "$fixture/repo" release-patch RELEASE_GROUP=endpoint \
    RELEASE_GROUPS=core,endpoint RELEASE_BATCH_ACTIVE=1 \
    > "$fixture/frozen-active.log" 2>&1 || status=$?
[[ "$status" == 2 && "$(cat "$RELEASE_SENTINEL")" == 'patch origin main' ]]
rm "$RELEASE_SENTINEL"
cp -p "$root/scripts/ci/run-release-batch.pl" "$fixture/repo/scripts/ci/run-release-batch.pl"
if [[ "$case_selection" == release-routing ]]; then
    complete=true
    echo "Local Make release routing passed (both groups; disposable fixture; release effects substituted)."
    exit 0
fi

# Admit the actual local Cargo recipes without compiling or running tests.
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == -V ]]; then printf 'cargo %s (fixture)\n' "${FIXTURE_CARGO_VERSION:-$WIRE_FIXTURE_COMPILER}"; exit; fi
printf '%s|%s|%s\n' "${RUSTUP_TOOLCHAIN:-development}" "${CARGO_TARGET_DIR:-}" "$*" >> "$CARGO_CALLS"
STUB
chmod +x "$fixture/bin/cargo"
export CARGO_CALLS="$fixture/cargo-calls"
for target in build check test lint check-wasm; do
    PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" "$target"
done
cat > "$fixture/expected-calls" <<EOF
$compiler|$fixture/repo/target|build --locked -p ic-wire
$compiler|$fixture/repo/target|check --locked -p ic-wire --all-targets
$compiler|$fixture/repo/target|test --locked -p ic-wire
$compiler|$fixture/repo/target|clippy --locked -p ic-wire -p ic-wire-derive -p ic-wire-endpoint-macros -p ic-wire-endpoint --all-targets --features ic-wire-endpoint/typescript -- -D warnings
$compiler|$fixture/repo/target|check --locked -p ic-wire --lib --target wasm32-unknown-unknown
EOF
# Policy selection overrides the caller's ambient compiler lane.
cmp "$fixture/expected-calls" "$CARGO_CALLS"
# The selected group reaches Clippy; workspace fallback overrides a narrow group.
for group in core endpoint; do
    : > "$CARGO_CALLS"
    PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" lint "RELEASE_GROUP=$group"
    if [[ "$group" == core ]]; then
        expected='-p ic-wire -p ic-wire-derive --all-targets --features ic-wire/typescript'
    else
        expected='-p ic-wire-endpoint-macros -p ic-wire-endpoint --all-targets --features ic-wire-endpoint/typescript'
    fi
    grep -Fx "$compiler|$fixture/repo/target|clippy --locked $expected -- -D warnings" "$CARGO_CALLS"
done
: > "$CARGO_CALLS"
WIRE_LINT_GROUP= PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" lint RELEASE_GROUP=endpoint
grep -F '|clippy --locked -p ic-wire -p ic-wire-derive -p ic-wire-endpoint-macros -p ic-wire-endpoint --all-targets --features ic-wire-endpoint/typescript -- -D warnings' "$CARGO_CALLS"
if WIRE_LINT_GROUP=unknown PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" lint > "$fixture/invalid-lint-group.log" 2>&1; then exit 1; fi
if PATH="$fixture/bin:$PATH" FIXTURE_CARGO_VERSION=0.0.0 bash "$fixture/repo/scripts/ci/rust-policy.sh" > "$fixture/invalid-compiler.log" 2>&1; then exit 1; fi
grep -F 'do not match Wire compiler policy' "$fixture/invalid-compiler.log" >/dev/null
cp "$fixture/repo/rust-toolchain.toml" "$fixture/toolchain-before"
printf '[toolchain]\nchannel = "0.0.0"\n' > "$fixture/repo/rust-toolchain.toml"
if PATH="$fixture/bin:$PATH" bash "$fixture/repo/scripts/ci/rust-policy.sh" > "$fixture/invalid-pin.log" 2>&1; then exit 1; fi
grep -F 'same exact Rust floor and pinned toolchain' "$fixture/invalid-pin.log" >/dev/null
cp "$fixture/toolchain-before" "$fixture/repo/rust-toolchain.toml"

# Real workspace metadata and Git identity; Cargo upload remains a sentinel.
export FIXTURE_CARGO_METADATA="$fixture/publication-metadata.json"
(cd "$fixture/repo" && cargo metadata --no-deps --locked --offline --format-version 1) > "$FIXTURE_CARGO_METADATA"
git -C "$fixture/repo" init -q
git -C "$fixture/repo" add -- .
git -C "$fixture/repo" -c user.name=Fixture -c user.email=fixture@example.invalid -c core.hooksPath=/dev/null commit -qm 'Disposable publication baseline'
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == -V ]]; then printf 'cargo %s (fixture)\n' "${FIXTURE_CARGO_VERSION:-$WIRE_FIXTURE_COMPILER}"; exit; fi
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
    if FIXTURE_LINT_STATUS=41 PATH="$fixture/bin:$PATH" make -j4 --no-print-directory -C "$fixture/repo" "$target" RELEASE_GROUP=core > "$fixture/$target-lint.log" 2>&1; then exit 1; fi
    grep -F '|clippy --locked -p ic-wire -p ic-wire-derive --all-targets --features ic-wire/typescript -- -D warnings' "$CARGO_CALLS" >/dev/null
    if grep -Eq '\|(publish|package) ' "$CARGO_CALLS"; then exit 1; fi
done

# Archive verification and its prerequisite lint select the same group.
for group in core endpoint; do
    : > "$CARGO_CALLS"
    PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" check-package "RELEASE_GROUP=$group" > "$fixture/archive-$group.log" 2>&1
    if [[ "$group" == core ]]; then expected='-p ic-wire-derive -p ic-wire'; else expected='-p ic-wire-endpoint-macros -p ic-wire-endpoint'; fi
    grep -Fx "$compiler|$fixture/repo/target|package --locked --allow-dirty $expected --all-features --target-dir target/publication-check" "$CARGO_CALLS"
    [[ "$(wc -l < "$CARGO_CALLS")" == 2 ]]
done
# Both defaults and explicitly reconciled subsets resolve exact workspace
# versions, retain the registry and dispatch once; publish-check is always dry.
for target in publish publish-check; do
    for group in core endpoint; do
        if [[ "$group" == core ]]; then selections=(default 'ic-wire-derive ic-wire' ic-wire ic-wire-derive); else selections=(default 'ic-wire-endpoint-macros ic-wire-endpoint' ic-wire-endpoint-macros ic-wire-endpoint); fi
        for selection in "${selections[@]}"; do
            : > "$CARGO_CALLS"
            args=("RELEASE_GROUP=$group")
            packages="$selection"
            if [[ "$selection" == default ]]; then
                if [[ "$group" == core ]]; then packages='ic-wire-derive ic-wire'; else packages='ic-wire-endpoint-macros ic-wire-endpoint'; fi
            else args+=("PACKAGE=$selection"); fi
            PATH="$fixture/bin:$PATH" make -j4 --no-print-directory -C "$fixture/repo" "$target" "${args[@]}" > "$fixture/publication-selection.log" 2>&1
            [[ "$(wc -l < "$CARGO_CALLS")" == 3 ]]
            expected='publish --locked --registry crates-io'
            for package in $packages; do version="$(jq -r --arg name "$package" '.packages[] | select(.name == $name) | .version' "$FIXTURE_CARGO_METADATA")"; expected+=" --package $package@$version"; done
            [[ "$target" != publish-check ]] || expected+=' --dry-run'
            [[ "$(tail -n 1 "$CARGO_CALLS")" == "${RUSTUP_TOOLCHAIN:-development}|$fixture/repo/target|$expected" ]]
        done
    done
done
for selection in '' ic-wire-qualification ic-wire-endpoint 'ic-wire ic-wire'; do
    : > "$CARGO_CALLS"
    if PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" publish RELEASE_GROUP=core "PACKAGE=$selection" > "$fixture/publish-invalid.log" 2>&1; then exit 1; fi
    [[ "$(wc -l < "$CARGO_CALLS")" == 1 ]]
    grep -F '|metadata --no-deps --locked --offline --format-version 1' "$CARGO_CALLS" >/dev/null
done
# A valid endpoint selection cannot publish a core package either.
: > "$CARGO_CALLS"
if PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" publish RELEASE_GROUP=endpoint PACKAGE=ic-wire > "$fixture/endpoint-cross-group.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$CARGO_CALLS")" == 1 ]]
: > "$CARGO_CALLS"
if PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" publish RELEASE_GROUP=core PUBLISH_PACKAGES=ic-wire > "$fixture/retired-selector.log" 2>&1; then exit 1; fi
[[ ! -s "$CARGO_CALLS" ]]
grep -F 'PUBLISH_PACKAGES was replaced by PACKAGE' "$fixture/retired-selector.log" >/dev/null

# A failed attempt stays failed with one dispatch and retained status; no retry.
: > "$CARGO_CALLS"
if FIXTURE_PUBLISH_STATUS=37 PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" publish RELEASE_GROUP=core PACKAGE=ic-wire > "$fixture/publish-failed.log" 2>&1; then exit 1; fi
[[ "$(grep -c '|publish ' "$CARGO_CALLS")" == 1 ]]
grep -rlx 37 "$fixture/repo/target/publication" | grep '/cargo-status$' >/dev/null
# Dirty source is refused before the simulated Cargo upload as well.
printf '\nDirty publication fixture.\n' >> "$fixture/repo/docs/runtime.md"
: > "$CARGO_CALLS"
if PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" publish RELEASE_GROUP=core > "$fixture/publish-dirty.log" 2>&1; then exit 1; fi
if grep -q '|publish ' "$CARGO_CALLS"; then exit 1; fi
git -C "$fixture/repo" restore docs/runtime.md

check_ci_routing

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
    printf '\nUnrelated fixture edit.\n' >> TOKO-TOOLING.md
    cp TOKO-TOOLING.md "$fixture/unrelated-before"
    bash .githooks/pre-commit > "$fixture/selected-hook.log" 2>&1
    grep -F 'const _: u8 = 1;' crates/ic-wire/src/lib.rs >/dev/null
    cmp "$fixture/unrelated-before" TOKO-TOOLING.md
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
