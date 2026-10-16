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
paths=(Makefile Cargo.toml Cargo.lock rust-toolchain.toml .gitignore .shared-tooling.snapshot crates/ic-wire/Cargo.toml crates/ic-wire/src/lib.rs)
paths+=(crates/ic-wire/examples/codec_evaluation/mod.rs crates/ic-wire/examples/codec_evaluation/bounded/mod.rs crates/ic-wire/examples/codec_evaluation/fixtures/mod.rs crates/ic-wire/examples/codec_evaluation/fixed/mod.rs)
paths+=(docs/evidence/codec-screen.json)
paths+=(crates/ic-wire/examples/typed_journey/mod.rs)
paths+=(crates/ic-wire/examples/typed_journey/schema/mod.rs)
paths+=(crates/ic-wire-canister-evaluation/Cargo.toml crates/ic-wire-canister-evaluation/src/lib.rs crates/ic-wire-canister-evaluation/examples/measure/mod.rs)
paths+=(crates/ic-wire-canister-evaluation/examples/browser_fixture/mod.rs crates/ic-wire-canister-evaluation/src/authority/mod.rs crates/ic-wire-canister-evaluation/src/peer/mod.rs)
paths+=(crates/ic-wire-canister-evaluation/examples/replay_fixture/mod.rs crates/ic-wire-canister-evaluation/src/replay/mod.rs crates/ic-wire-canister-evaluation/src/replay/tests/mod.rs)
while IFS=$'\t' read -r record _ _ path; do
    [[ "$record" == file ]] || continue
    paths+=("$path")
done < "$root/.shared-tooling.snapshot"
for path in "${paths[@]}"; do
    mkdir -p "$fixture/repo/$(dirname "$path")"
    cp -p "$root/$path" "$fixture/repo/$path"
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
for target in build check test lint check-wasm check-msrv evaluate-codecs evaluate-journey; do
    PATH="$fixture/bin:$PATH" make --no-print-directory -C "$fixture/repo" "$target"
done
cat > "$fixture/expected-calls" <<EOF
development|$fixture/repo/target|build --locked -p ic-wire
development|$fixture/repo/target|check --locked -p ic-wire --all-targets
development|$fixture/repo/target|test --locked -p ic-wire
development|$fixture/repo/target|clippy --locked -p ic-wire --all-targets -- -D warnings
development|$fixture/repo/target|check --locked -p ic-wire --lib --target wasm32-unknown-unknown
1.91.0|$fixture/repo/target|check --locked -p ic-wire --lib
1.91.0|$fixture/repo/target|check --locked -p ic-wire --lib --target wasm32-unknown-unknown
development|$fixture/repo/target|run --quiet --locked -p ic-wire --example codec-evaluation
development|$fixture/repo/target|run --quiet --locked -p ic-wire --example typed-journey
EOF
# The ambient development selection may be explicit (for example in CI).
sed "s/^development|/${RUSTUP_TOOLCHAIN:-development}|/" "$fixture/expected-calls" > "$fixture/expected-environment"
cmp "$fixture/expected-environment" "$CARGO_CALLS"

# Restore the reviewed runner before exercising the real formatting hook.
cp -p "$root/scripts/ci/run-release.sh" "$fixture/repo/scripts/ci/run-release.sh"
git -C "$fixture/repo" init -q
git -C "$fixture/repo" add -- "${paths[@]}"
make --no-print-directory -C "$fixture/repo" install-hooks > "$fixture/install-hooks.log"
[[ "$(git -C "$fixture/repo" config --local core.hooksPath)" == .githooks ]]
(
    cd "$fixture/repo"
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
