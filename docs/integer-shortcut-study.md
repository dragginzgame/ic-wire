# Integer shortcuts and canonical comparison (pending 0.6.6)

Retain the small-u16/u32 shortcut; reject the signed, streamed-u64 and larger
blob-comparison candidates. Production remains one bounded Serde/Postcard codec.
The private adapter uses Postcard's u8 writer for unsigned values below 128,
just as the selected u64 optimization already does. Original type-size, node
and depth admission remain intact. For both u16 and u32, 127 remains `7f` and
128 remains `80 01`. Frames, canonical acceptance, retry bytes, hashes and client
layouts are unchanged; consumers need no migration.

## Selected result

Opt-3 IC instructions per operation, ordinary owned representation. Small vectors
cycle through 0–63; wide vectors repeat the type's maximum. Values include empty,
one-element, 1,024-element and 16,384-element controls. Decode includes canonical
reserialization. Setup is excluded; destruction and loop overhead are included.

| Input / operation | Wire before | Wire after | Change | Candid |
| --- | ---: | ---: | ---: | ---: |
| Empty u16 list / encode | 794.25 | 794.25 | 0% | 13,609.25 |
| One u16 / encode | 1,033.25 | 914.25 | -11.52% | 13,723.25 |
| 1,024 small u16s / encode | 200,528.25 | 78,672.25 | -60.77% | 19,045.25 |
| 16,384 small u16s / encode | 3,195,750.25 | 1,246,054.25 | -61.01% | 80,555.25 |
| 16,384 small u16s / decode | 2,666,845.5 | 2,011,484.5 | -24.57% | 1,375,622.5 |
| Empty u32 list / encode | 797.25 | 801.25 | +0.50% | 13,609.25 |
| One u32 / encode | 1,002.25 | 887.25 | -11.47% | 13,738.25 |
| 1,024 small u32s / encode | 193,348.25 | 71,496.25 | -63.02% | 23,141.25 |
| 16,384 small u32s / encode | 3,081,052.25 | 1,131,360.25 | -63.28% | 146,244 |
| 16,384 small u32s / decode | 4,313,204.5 | 3,641,464.5 | -15.57% | 1,375,592 |

The large small-value frames remain 16,398 bytes for Wire, versus 32,780 for
Candid u16 and 65,548 for Candid u32. Encoding still costs 15.47x and 7.74x Candid
respectively. This narrows an important gap without claiming parity.

Worst measured regression is opt-z wide-u16 reuse: 10,126,513.5 to 10,519,729.5
instructions (+3.883%). Fresh encoding similarly rises from 10,126,983.5 to
10,520,199.5 (+3.883%). Large small-u32 opt-z fresh encoding improves from
8,783,494.5 to 2,393,734.5 (-72.75%). Large u64 vectors, generic byte vectors and
owned blobs are unchanged at opt-3. The modest wide-value regression and bounded
code increase are accepted for the substantial small-integer gains.

## Uncompressed Wasm

Matched complete artifacts; code-section payload bytes are reported separately.
The expanded 154-case corpus includes forty new integer-vector cases on both
sides. Its size must not be compared directly to earlier 114-case artifacts.

| Fixture/profile | Wire total before → after | Wire code before → after | Candid total / code |
| --- | ---: | ---: | ---: |
| Expanded / opt-z | 251,371 → 251,552 (+181) | 215,377 → 215,556 (+179) | 615,602 / 557,804 |
| Expanded / opt-3 | 358,596 → 359,197 (+601) | 307,940 → 308,541 (+601) | 932,103 / 859,605 |
| 100 records / opt-z | 324,055 → 324,055 | 305,232 → 305,232 | 1,416,786 / 1,362,770 |
| 100 records / opt-3 | 426,626 → 426,626 | 392,100 → 392,100 | 2,079,944 / 2,010,763 |

The 100-record fixture uses u64/i64 fields, not u16/u32 fields: it is an unaffected
control, not proof of the new shortcut's scaling on 100 affected types. Artifact
sizes include benchmark dispatch and fixtures; they do not predict application
size. Candid binaries and counters remain identical between paired runs.

## Rejected experiments

Each candidate was measured separately against the same baseline.

- **Signed i16/i32/i64:** forward -64 through 63 as the equivalent zigzag byte.
  Large small-i64 opt-3 encoding improves 4,064,078.25 → 2,016,078.25 (-50.39%),
  but wide-i16 fresh encoding regresses 4,014,941.25 → 4,391,773.25 (+9.39%).
  Total Wasm grows 296 bytes at opt-z and 884 at opt-3. Do not adopt this version.
- **Stream wide u64:** write varint bytes through an unframed Postcard tuple.
  Large wide-u64 opt-3 encoding regresses 7,750,801 → 10,142,820 (+30.86%);
  decode regresses 12,308,855 → 15,896,952 (+29.15%). Total opt-3 Wasm grows
  2,729 bytes. Small-u64 gains do not justify these regressions.
- **128-byte canonical scan:** retain exact comparison but double bulk width.
  One-MiB owned opt-3 blob decode improves 2,524,535 → 2,442,624 (-3.24%), while
  a 127-byte owned blob regresses 1,529.5 → 1,613.5 (+5.49%); the bounded case
  regresses 5.64%. Total Wasm grows 162/259 bytes at opt-z/opt-3. Keep the current
  64-byte scan. Skipping canonical comparison remains invalid for custom Serde.

## Qualification and reproduction

Baseline: release 0.6.5 (`e10196e`) plus the adopted Tooling 0.65.0 lock and the
same expanded corpus on both sides. Rust 1.99.1 / LLVM 23.1.1, wasm32-unknown-unknown,
LTO, one codegen unit, panic abort, stripped symbols; evaluation uses opt-z and
benchmark-speed uses opt-3. PocketIC 16.1.0 measures performance counter 0 with
three identical samples, four operations through 64 KiB and one above. No overhead
is subtracted. Input/frame hashes, lengths and repetition counts are matched. The five standard
runs and two 100-record runs retain 23,190 rows in total. All 100-record counts
are unchanged.

The [raw archive](evidence/integer-shortcuts-066/raw.json.gz) contains baseline
workspace sources, all four candidate implementations, build scripts, full samples,
run identities and validation logs. Extract `source` entries to a temporary
workspace, replace the scripts' `/tmp/wire-opt066` and target paths, then run
`run-build.sh`. Run the built `codec-instructions OUTPUT` for baseline, unsigned
and signed, setting `POCKET_IC_BIN` to the prepared server. Run `extra-build.sh`
and measure stream/bulk128. Finally run `scale-build.sh`, which rebuilds the driver
for 100 records, and measure scale-baseline/scale-unsigned. Keep each output's
`instructions` directory absent before running. Build and measurement ordering
matters because the standard and scale drivers share a binary path.

[Instruction table](evidence/integer-shortcuts-066/instructions.csv),
[artifact sizes/hashes](evidence/integer-shortcuts-066/artifacts.csv),
[aggregate results](evidence/integer-shortcuts-066/summary.csv) and
[compiler/dependency identity](evidence/integer-shortcuts-066/identity.json)
retain the detailed results. All emitted IC frames match the host oracle.
The retained implementation additionally passes exhaustive u16 and u32 varint
boundary checks with exact/insufficient budgets, runtime/borrowing tests,
benchmark roundtrips, TypeScript/lint and native/Wasm MSRV. The new Host/Testkit
startup path passes actual PocketIC runs and the host-driver MSRV check.
No new native-time, allocation, heap, macOS, installed-consumer or full-CI claim.
