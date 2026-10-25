#!/usr/bin/env bash
set -euo pipefail
run_root="${1:?absolute run root required}"
kind="${2:?normalized or release required}"
case "$run_root" in /*) ;; *) echo 'run root must be absolute' >&2; exit 1;; esac
case "$kind" in
    normalized) source=baseline; results=baseline-results ;;
    release) source=release-baseline; results=release-baseline-results ;;
    *) echo 'expected normalized or release' >&2; exit 1 ;;
esac
: "${CARGO_TARGET_DIR:?set an isolated target directory}"
: "${POCKET_IC_BIN:?set the matched explicitly prepared server}"
: "${CODEC_REPORT:?absolute path to the built codec-report executable}"
cd "$run_root/$source"
export CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0
out="$run_root/$results"
mkdir "$out"
report="$CODEC_REPORT"
printf 'cohort,profile,codec,before_bytes,after_bytes,before_code,after_code,identical,before_sha256,after_sha256\n' > "$out/comparison.csv"
while IFS='|' read -r name features primary; do
    mkdir "$out/$name"
    if [ "$name" = hotspots ]; then current="$run_root/hotspots"; else current="$run_root/current/$name"; fi
    extra="${features:+,$features}"
    for profile in evaluation benchmark-speed; do
        for codec in wire candid; do
            cargo build --offline --locked --profile "$profile" -p ic-wire-benchmarks --example codec-wasm --target wasm32-unknown-unknown --features "$codec$extra" --message-format=json > "$out/$name/$profile-$codec.jsonl" 2> "$out/$name/$profile-$codec.log"
            artifact="$("$report" --artifact "$out/$name/$profile-$codec.jsonl")"
            cp "$artifact" "$out/$name/$profile-$codec.wasm"
            before_bytes="$(wc -c < "$artifact")"
            after_bytes="$(wc -c < "$current/$profile-$codec.wasm")"
            before_code="$("$report" --code-size "$artifact")"
            after_code="$("$report" --code-size "$current/$profile-$codec.wasm")"
            identical=false
            if cmp -s "$artifact" "$current/$profile-$codec.wasm"; then identical=true; fi
            before_sha="$(shasum -a 256 "$artifact")"
            after_sha="$(shasum -a 256 "$current/$profile-$codec.wasm")"
            printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' "$name" "$profile" "$codec" "$before_bytes" "$after_bytes" "$before_code" "$after_code" "$identical" "${before_sha%% *}" "${after_sha%% *}" >> "$out/comparison.csv"
        done
    done
    if [ "$(tail -4 "$out/comparison.csv" | cut -d, -f8 | sort -u)" != true ]; then
        cargo run --offline --locked -p ic-wire-benchmarks --example codec-instructions --features "wire,candid,ic$extra" -- "$out/$name" > "$out/$name/instructions.log" 2>&1
    fi
    printf 'Compared baseline %s\n' "$name"
done < scripts/ci/codec-cohorts.psv
