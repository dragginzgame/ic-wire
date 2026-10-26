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
paths=(Makefile Cargo.toml Cargo.lock rust-toolchain.toml .gitignore .shared-tooling.snapshot docs/runtime.md)
paths+=(scripts/ci/fuzz-runtime.sh scripts/ci/check-diagnostics.sh)
# Include authored crate inputs automatically, including new untracked modules.
# Git exclusions keep ignored build/cache output out of the disposable export.
git -C "$root" ls-files --cached --others --exclude-standard -z -- crates/ > "$fixture/crate-paths"
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
development|$fixture/repo/target|clippy --locked -p ic-wire -p ic-wire-derive --all-targets -- -D warnings
development|$fixture/repo/target|check --locked -p ic-wire --lib --target wasm32-unknown-unknown
1.91.0|$fixture/repo/target|check --locked -p ic-wire --lib
1.91.0|$fixture/repo/target|check --locked -p ic-wire --lib --target wasm32-unknown-unknown
EOF
# The ambient development selection may be explicit (for example in CI).
sed "s/^development|/${RUSTUP_TOOLCHAIN:-development}|/" "$fixture/expected-calls" > "$fixture/expected-environment"
cmp "$fixture/expected-environment" "$CARGO_CALLS"

# Publication/package entrypoints must stop at lint, including parallel Make.
cp "$fixture/bin/cargo" "$fixture/cargo-success"
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$CARGO_CALLS"
[[ "$1" != clippy ]] || exit 41
exit 0
STUB
for target in publish check-package; do
    : > "$CARGO_CALLS"
    if PATH="$fixture/bin:$PATH" make -j4 --no-print-directory -C "$fixture/repo" "$target" > "$fixture/$target-lint.log" 2>&1; then exit 1; fi
    [[ "$(wc -l < "$CARGO_CALLS")" == 1 ]]
    grep -Fx 'clippy --locked -p ic-wire -p ic-wire-derive --all-targets -- -D warnings' "$CARGO_CALLS" >/dev/null
done
cp "$fixture/cargo-success" "$fixture/bin/cargo"

# Resume partial uploads by explicitly selecting only unpublished packages.
# Cargo is a sentinel: these checks cannot contact or upload to a registry.
for selection in 'ic-wire-derive ic-wire' ic-wire ic-wire-derive; do
    : > "$CARGO_CALLS"
    PATH="$fixture/bin:$PATH" make -j4 --no-print-directory -C "$fixture/repo" publish PUBLISH_PACKAGES="$selection" > "$fixture/publish-selection.log" 2>&1
    [[ "$(wc -l < "$CARGO_CALLS")" == 2 ]]
    expected='publish --locked'
    for package in $selection; do expected+=" -p $package"; done
    [[ "$(tail -n 1 "$CARGO_CALLS")" == "${RUSTUP_TOOLCHAIN:-development}|$fixture/repo/target|$expected" ]]
done
for selection in '' ic-wire-qualification; do
    : > "$CARGO_CALLS"
    if PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" publish PUBLISH_PACKAGES="$selection" > "$fixture/publish-invalid.log" 2>&1; then exit 1; fi
    [[ ! -s "$CARGO_CALLS" ]]
done

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
