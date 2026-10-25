# Multi-byte parser sharing, admission budgets and Cargo advice

This follow-up starts from released 0.7.11
`a99a99cc3fadbde004e69d9ee4888c99711b3bee` plus the preceding uncommitted
[adversarial qualification](../followup-0712/README.md). That work is preserved.
No production runtime or protocol changes are retained. New code is confined to
unpublished measurement/qualification fixtures and their check integration.

## Multi-byte sharing: rejected

The candidate leaves each u16/u32/u64 reader's first-byte check in place, moving
only its multi-byte tail into a helper forced out of line on Wasm. Native helpers
use an always-inline hint; native timing is not qualified because the candidate
was rejected. The u128 reader is unchanged. The isolated [runner](measure.mjs)
constructs the exact candidate; no candidate code is installed in the library.

Same compiler (Rust 1.99.1), lock, source directory, input fixtures and profiles
are used across variants. Both profiles use LTO and one codegen unit. No extra
RUSTFLAGS were supplied. [Sizes/hashes](wasm-sizes.json) distinguish total Wasm
from code sections; gzip is not a footprint metric.

| Fixture/profile | Total before → candidate | Code before → candidate |
|---|---:|---:|
| Standard/opt-z | 377,811 → 377,990 | 337,782 → 337,958 |
| Standard/opt-3 | 573,482 → 568,516 | 518,734 → 513,769 |
| 100 types/opt-z | 310,738 → 310,968 | 291,971 → 292,198 |
| 100 types/opt-3 | 398,739 → 399,800 | 364,245 → 365,303 |

The only total saving is 4,966 bytes (0.87%) in standard opt-3; its code section
shrinks 4,965 bytes (0.96%). Other totals grow 179/230/1,061 bytes. Opt-3 owned
u16-list decode regresses from 815,349.5 to 1,012,002.5 instructions for 16,384
small values (+24.12%), and from 1,839,374.5 to 2,543,931.5 for wide values
(+38.30%). Empty/single-element small-list controls rise from 298.5 to 300.5
(+0.67%) and 693.5 to 707.5 (+2.02%). Scalar zero rises 256.5 → 266.5 (+3.90%).
Other paths also regress: option-none decode rises 245.5 → 354.5 (+44.40%).
Preserving a source-level fast branch did not preserve the complete fixture's cost.

Standard qualification covers 298 cases and 7,006 operation rows per variant,
with three equal repeated instruction counters per row. Baseline Wasm artifacts
are byte-identical to the preceding experiment, so its measured baseline counters
are reused; candidate counters are fresh. Candid controls remain unchanged.
Paired input/frame hashes match. The 100-type fixture is size-only. The
[raw archive](instructions.json.gz) retains samples and identities; the
[owned comparison](instruction-changes.csv) includes regressions and improvements.
Use the preceding [analyzer](../followup-0712/analyze.mjs) to verify pairing.

## Admission budgets: measured tradeoffs

The new dedicated probe uses ordinary u8/u64 vectors and byte buffers, with
0/1/64/1,024/65,536/1,048,576 elements. Each valid and last-byte-truncated frame
runs under four policies. These are illustrative settings, not application advice
or equivalent Candid quotas. No defaults change and no Candid quota comparison
is claimed. The prior full suite remains the codec comparison control.

The generous policy permits 16 MiB frames, 256 MiB allocation allowance,
8,388,608 nodes and depth 64. The other policies change exactly one setting:
nodes to 1,024; allocation to the frame allowance plus 64 KiB; or frame size to
64 KiB. Allocation allowance is conservative accounting, not measured heap usage.

| Truncated ordinary u8 vector, 1,048,576 elements | IC instructions | Result |
|---|---:|---|
| Generous | 69,211,213 | Malformed |
| 1,024 nodes | 36,030 | Limit |
| 64 KiB allocation headroom | 272,998 | Limit |
| 64 KiB frame cap | 226 | Limit |

Node and allocation policies reduce this refusal cost by 99.95% and 99.61%.
They also refuse the valid large ordinary vector: this is an acceptance tradeoff,
not a faster way to accept it. All valid 0/1/64-element controls pass all policies.
For truncated ordinary u8 controls of those sizes, every policy costs the same
337/436/5,701 instructions respectively. The 1 MiB truncated byte buffer costs
417 under the first three policies and 226 when refused by the frame cap.
The node policy still accepts the valid 1 MiB byte buffer; the allocation/frame
policies refuse it. Wide-u64 results and all other controls are retained too.

All 144 configurations have three identical repeated samples. Host and Wasm
outcomes agree, including destruction; setup and transport are excluded. Result
codes are 0=success, 1=Limit, 2=Malformed. Input hashes and every budget are in
[raw samples](admission-samples.jsonl), with [artifact/server identity](admission-identity.json).
Absolute counters differ from the broad suite's fixtures and budget settings;
compare policies within this probe, not these counts against earlier suite counts
as a runtime regression. No native-time or allocation-count savings are claimed.

## Cargo warning qualification

The Wire-owned disposable build-script consumer verifies default Cargo warning
visibility, independent per-field suppression, complete acknowledgement and stale
acknowledgements. It passes on Rust 1.99.1 and 1.91.0. The check runs inside
`check-typescript`, or directly with `bash scripts/ci/check-bulk-advice.sh`.
No optional warning feature was added, and no Canic checkout was edited.
This proves Wire's integration pattern, not adoption in an installed consumer.

New benchmark examples pass strict Clippy and native/Wasm Rust 1.91 checks.
Formatting and local documentation links pass. Full CI and the complete Candid
matrix were not rerun; runtime code is unchanged.

## Reproduce

For the parser study, follow the preceding experiment's commands using this
[measurement runner](measure.mjs), then its instruction runner/analyzer commands.
Use the baseline commit above, cached dependencies, the pinned toolchain and
prepared PocketIC 16.0.0 server recorded in the identities. Run baseline and
candidate measurements serially; the runner restores the isolated parser.

For the budget study, reconstruct the new unpublished examples from the
[frozen patch](admission.patch); [source hashes](sources.sha256) identify the inputs:

```sh
run=$(mktemp -d)
mkdir "$run/source"
git archive a99a99cc3fadbde004e69d9ee4888c99711b3bee | tar -x -C "$run/source"
git -C "$run/source" apply "$PWD/docs/evidence/tail-and-budgets-0712/admission.patch"
export CARGO_TARGET_DIR="$run/target"
cargo +1.99.1 build --offline --locked --manifest-path "$run/source/Cargo.toml" \
  --profile benchmark-speed -p ic-wire-benchmarks --example admission-wasm \
  --target wasm32-unknown-unknown
cargo +1.99.1 build --offline --locked --manifest-path "$run/source/Cargo.toml" \
  -p ic-wire-benchmarks --example admission-instructions --features wire,candid,ic
export POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic
"$run/target/debug/examples/admission-instructions" \
  "$run/target/wasm32-unknown-unknown/benchmark-speed/examples/admission_wasm.wasm" \
  "$run/results"
```
