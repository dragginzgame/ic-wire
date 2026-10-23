# Serde short-write experiment (0.6.5 research)

No runtime change is retained. Five isolated candidates target small Postcard
varint writes in the production Serde output writer. The best small-integer-list
result saves about 40% of opt-3 encode instructions, but the variants regress
other cases and grow opt-3 Wasm. The existing codec remains selected; no protocol,
resource admission, canonical verification or consumer migration changes.

Baseline is release 0.6.4 (`0300e81`) with the incoming dirty Cargo.lock, including
Tooling 0.64.2 and transport 0.49.3. Both sides use that exact lock; this experiment
does not adopt or revert its dependency changes. Rust 1.99.1 / LLVM 23.1.1 targets
wasm32-unknown-unknown, LTO, one codegen unit, panic abort and stripped symbols.
`evaluation` uses opt-z; `benchmark-speed` uses opt-3. PocketIC 16.1.0 records
ic0.performance_counter(0), three identical samples per operation, four operations
through 64 KiB and one above, with no loop/counter subtraction. Setup is excluded;
result destruction is included. The unchanged standard corpus owns inputs/limits.

The variants are:

- `candidate`: a one-byte slice calls the existing byte writer before the bulk check.
- `shared`: perform the existing bounds check, then push one byte or copy the slice.
- `inline`: first candidate plus Wasm-only forced inlining of the byte writer.
- `inline-both`: first candidate with both writer methods inlined on Wasm.
- `inline-only`: no new branch; force both original methods inline on Wasm.

## Measured tradeoffs

Opt-3 fresh encoding instructions, ordinary owned Serde representation:

| Input | Baseline | `inline` | Change |
| --- | ---: | ---: | ---: |
| Empty small-u64 list | 924.25 | 806.25 | -12.77% |
| One small u64 | 1,243.25 | 1,005.25 | -19.14% |
| 16,384 small u64s | 4,883,417.25 | 2,917,336.25 | -40.26% |
| 16,384 wide u64s | 7,865,489 | 7,849,104 | -0.21% |
| 1 MiB generic byte vector | 78,644,290 | 75,498,558 | -4.00% |
| Relay batch, 128 messages | 220,235.25 | 206,666.25 | -6.16% |

However, `inline` raises opt-z 1 MiB generic-vector encoding from 148,899,760 to
158,336,945 instructions (+6.34%). Its opt-3 Wasm grows by 3,392 total bytes and
3,380 code-section bytes. Small-u64 encoding remains substantially slower than
Candid even with this candidate. Owned blob decoding receives no meaningful fix.

Whole standard Wire fixture bytes (total / code section):

| Variant | opt-z | opt-3 |
| --- | ---: | ---: |
| Baseline | 228,821 / 193,545 | 321,261 / 271,236 |
| `candidate` | 228,872 / 193,595 | 323,094 / 273,069 |
| `shared` | 228,813 / 193,537 | 323,189 / 273,164 |
| `inline` | 228,861 / 193,585 | 324,653 / 274,616 |
| `inline-both` | 228,816 / 193,542 | 323,228 / 273,174 |
| `inline-only` | 228,477 / 193,203 | 322,952 / 272,897 |

`candidate` regresses opt-z generic-vector encoding by 10.56%; `shared` still
regresses it by 6.34%. `inline-both` adds a 17.20% opt-3 small principal-byte reuse
regression (465.25 to 545.25 instructions). Inlining alone leaves small-u64 costs
unchanged while raising anonymous-principal byte reuse from 437.25 to 632.25
(+44.60%). These cases measure byte representations, not principal validation.
None is retained given Wasm-size priority and the additional regressions.

## Evidence and reproduction

All six runs cover the same 114 cases in both profiles: 2,958 rows per run,
17,748 total. Every pair retains input/frame hashes, lengths and repetitions;
all three instruction samples agree. Candid binaries are copied unchanged to
candidate runs, and the host driver verifies exact returned frames against the
native fixture oracle. Bad-header and truncation operations are included. These
are instruction and Wasm measurements, not new native-time, heap, installed-app,
100-type scaling or exhaustive custom-Serde correctness qualification.

[Selected operations and controls](evidence/serde-short-writes-065/instructions.csv),
[all artifact sizes/hashes](evidence/serde-short-writes-065/artifacts.csv),
[aggregate changes](evidence/serde-short-writes-065/summary.csv) and
[compiler/revision/lock identity](evidence/serde-short-writes-065/identity.json)
retain the comparison. The [raw archive](evidence/serde-short-writes-065/raw.json.gz)
contains all rows, per-run identities, the complete source/lock snapshot, each
candidate contract module and the executed build scripts. Local binaries/logs
are under `/tmp/wire-opt065/`; nothing there is a production mode.

To reproduce, extract the archive's `source` path/text mapping into a fresh
workspace; replace `crates/ic-wire/src/contract/mod.rs` with the selected
`contracts` entry. Use an isolated target directory per source variant. Build
`codec-wasm` with `cargo build --locked -p ic-wire-benchmarks --example codec-wasm
--target wasm32-unknown-unknown --profile PROFILE --features CODEC` for both named
profiles and wire/candid. Copy outputs as PROFILE-CODEC.wasm in each result folder.
Run `cargo run --locked -p ic-wire-benchmarks --example codec-instructions
--features wire,candid,ic -- RESULT_FOLDER` with the prepared POCKET_IC_BIN.
Use `codec-report --code-size WASM_PATH` for code-section payload sizes.
