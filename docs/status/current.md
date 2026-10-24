# Current handoff

## Released baseline and pending 0.7.4

0.7.3 is released at `2240f27c9e3f8074a533ed5d030685ee068a03b9`; package versions
remain 0.7.3. The checkout was clean at the start of this batch. Pending 0.7.4
retains the standalone one-byte varint shortcut after rejecting shared-loop and
output experiments. Incoming staged documentation/evidence was preserved.
Dependencies, build profiles and protocol are unchanged. Staging,
commits, versions, publication and pushes remain maintainer-owned.

## Tooling and qualification

Build/Testkit and the release editor select 0.68.1; Platform selects 0.6.15.
Shared Tooling remains 0.26.10 at `4c0f198e56d4c6d27b12a5c2d515e0e01070151e`.
The 0.7.3 adoption preserved the normal Wasm dependency graph. Its focused
boundary, diagnostics, benchmark, schema-forwarding and disposable release/Make
checks passed; retained bulk Wasms reproduced all 2,180 rows through updated host
tooling. This was host integration qualification, not fresh codec performance.

0.7.3 includes the Bash 3.2 schema-target fix and benchmark-report validation.
The macOS fix is now delivered locally in the release; hosted macOS results and
issue #17 closure have not been rechecked in this batch. Linux checks alone do
not establish macOS success. No sibling repository was changed.

## Wasm research decisions

Keep one production codec. Consumer optimization levels are test conditions, not
an ic-wire-controlled fix. Preserve both total uncompressed Wasm and code-section
measurements; gzip and symbol attribution are not savings criteria.

- [Budget sharing](../evidence/wasm-budget-073/sizes.json): rejected. About 1%
  total savings at 100 types cost 17.7–28.8% more decode instructions. The
  [raw evidence](../evidence/wasm-budget-073/raw.tar.gz) retains reproduction inputs.
- [Scalar sharing and initial wrapper/profile probes](../evidence/wasm-scalar-073/sizes.json):
  scalar sharing did not save size; profile gains varied by workload. The
  [raw evidence](../evidence/wasm-scalar-073/raw.tar.gz) retains those controls.
- [Encoding-wrapper follow-up](../evidence/encoder-inlining-073/sizes.json):
  rejected across standard, Toko, bulk and 1/100-type controls at opt-z and opt-3.
  The small-u32 list's encode cost rose from 1,606,561.25 to 3,228,721.25 IC
  instructions (+100.97%) at opt-3. Decode counts were unchanged and all 6,246
  paired rows preserved payloads. [Per-case results](../evidence/encoder-inlining-073/instructions.csv)
  and [raw reproduction evidence](../evidence/encoder-inlining-073/raw.tar.gz) remain.

### Output preparation (0.7.4 research, rejected)

The isolated candidate moves reservation and frame-prefix copying into one
non-generic Wasm-outlined helper. Payload traversal, ordering of fallible work,
bytes and admission rules are unchanged. It fails the size screen:

| Types / profile | Total Wasm, before → after | Code section, before → after |
| --- | ---: | ---: |
| 1 / opt-z | 53,841 → 53,878 | 42,841 → 42,877 |
| 1 / opt-3 | 74,844 → 75,475 | 48,102 → 48,731 |
| 100 / opt-z | 310,664 → 310,489 | 291,897 → 291,721 |
| 100 / opt-3 | 398,712 → 401,895 | 364,218 → 367,399 |

The 175-byte (0.056%) total saving at 100 types/opt-z does not justify growth
elsewhere, especially 3,183 bytes (+0.798%) at opt-3. No candidate was retained;
IC/native instruction or timing tests were not run after this failed size screen.
This is not a performance improvement claim.

Both variants use release 0.7.3 source and the same lock, Rust 1.99.1 / LLVM 23.1.1,
wasm32-unknown-unknown, LTO, one codegen unit, panic abort and stripped symbols.
[Sizes and artifact hashes](../evidence/output-preparation-074/sizes.json) and
[source snapshots, compiler, build log and runner](../evidence/output-preparation-074/source.tar.gz)
retain reproduction inputs. Extract and run `bash replay.sh` with prepared
Rust/dependencies; replay creates a fresh temporary directory. Original artifacts
remain at `/tmp/wire-output-074/`. Doc links, formatting and diff checks pass.

### Byte-output routines (0.7.4 research, rejected)

The next isolated candidate forces `Output::try_push` and `try_extend` inline on
Wasm. A shared-call alternative was also size-screened. The inline candidate
shrinks the 1/100-type controls, but broader instruction qualification rejects it.

| Cohort | Total Wasm delta, opt-z / opt-3 | Worst fresh-encode IC regression, opt-3 |
| --- | ---: | ---: |
| Standard | -56 / +2,372 bytes | +98.18% |
| Toko | -402 / -720 bytes | +5.05% |
| Bulk | -76 / +441 bytes | +17.81% |
| 1 type | -460 / -271 bytes | -0.84% (improvement) |
| 100 types | -451 / -2,796 bytes | +18.91% |

The standard wide-u16 list (16,384 elements) rises from 1,852,251.25 to
3,670,875.25 encode instructions at opt-3; reuse regresses 98.20%. At opt-z,
fresh encode regresses up to 9.64%, and decode up to 3.53%; opt-3 decode is
unchanged. All 6,246 paired rows preserve input/frame hashes and lengths.

Baseline: released 0.7.3 runtime and identical dependencies for both variants.
Rust 1.99.1 / LLVM 23.1.1, Wasm, LTO, one codegen unit, panic abort, stripped
symbols; opt-z and opt-3 remain test conditions, not consumer recommendations.
PocketIC 16.0.0, three samples of four repetitions (one above 64 KiB), including
loop/counter overhead. No native-time claim or production change. These results
reinforce that small type-count controls alone cannot select an inlining policy.

[Total/code-section sizes and hashes](../evidence/output-writes-074/sizes.json),
[absolute instruction pairs](../evidence/output-writes-074/instructions.csv) and
[source, compiler, raw samples and scripts](../evidence/output-writes-074/raw.tar.gz)
retain evidence. Extract and run `bash replay.sh` with prepared dependencies and
explicit `POCKET_IC_BIN`; it uses a fresh directory. Wasms are reproducible from
the source and hashes; original artifacts remain at `/tmp/wire-output-writes-074/`.
Protocol and production runtime are unchanged. Doc links and formatting pass.

### Shared varint readers (0.7.4 research, rejected)

Two isolated structural variants share the u16/u32/u64 checked read loop; u128
stays unchanged. The first shares every value; the refinement handles one-byte
values locally and shares only multi-byte tails. Both preserve width-specific
maximum groups, final-byte masks and redundant-group rejection. Each passes the
two library and 21 native tests. Neither is retained.

The first variant saves 11,198 total / 11,194 code bytes in the standard opt-3
Wasm (573,672 → 562,474 total, -1.95%), but small-u16 list decode rises from
913,655.5 to 2,093,413.5 instructions (+129.13%). The refinement reduces that case
to 848,199.5 (-7.16%), but wide-u16 decode rises from 1,872,144.5 to 3,314,016.5
(+77.02%). Its standard opt-3 Wasm saves 5,418 total / 5,415 code bytes (-0.94%);
its opt-z and both 100-type builds grow. These tradeoffs reject shared loops as
implemented. The standalone one-byte shortcut is separately qualified below.

Same 0.7.3 source/lock, Rust 1.99.1 / LLVM 23.1.1, Wasm, LTO, one codegen unit,
panic abort and stripped symbols for each matched profile. Initial IC runs cover
standard and 1/100-type controls at opt-z/opt-3; the refinement's IC run covers
the standard 298-case corpus in both profiles after the three-cohort size screen.
PocketIC 16.0.0, three samples of four operations (one above 64 KiB), no overhead
subtraction. All 9,062 paired rows match input/frame identities and lengths.
No native-time claim. [Sizes/hashes](../evidence/shared-varint-074/sizes.json),
[absolute instruction pairs](../evidence/shared-varint-074/instructions.csv) and
[source, tests, samples and replay script](../evidence/shared-varint-074/raw.tar.gz)
retain reproduction inputs; extract and run `bash replay.sh` with prepared
dependencies and explicit `POCKET_IC_BIN`. Original Wasms remain in
`/tmp/wire-varint-074/`. Production and protocol are unchanged; doc checks pass.

### Standalone one-byte shortcut (0.7.4, retained)

u16/u32/u64 now return directly for the first byte below 128. Their existing
width-specific loops handle subsequent bytes; u128 is unchanged. There is no
shared-reader mode or new API. Bytes remain `127 → 7f`, `128 → 80 01`; redundant
`80 00` stays malformed. Limits, rejection rules, retry bytes and hashing inputs
are unchanged. The retained parser exactly matches the measured candidate.

| Cohort | Total Wasm delta, opt-z / opt-3 | Decode IC change, opt-z / opt-3 |
| --- | ---: | ---: |
| Standard | +205 / -208 bytes | -10.97% to 0% / -10.76% to +0.80% |
| Toko | +114 / -9 bytes | -6.75% to -1.14% / -1.34% to 0% |
| Bulk | +83 / -43 bytes | -5.85% to -0.007% / -1.14% to -0.002% |
| 1 type | +157 / +32 bytes | +3.58% / -0.32% |
| 100 types | +74 / +32 bytes | -3.82% to -2.16% / -0.27% to -0.12% |

This is a modest decode optimization with nearly neutral Wasm, not a universal
size reduction. The opt-z small-i32 list drops from 5,527,409.5 to 4,921,173.5
instructions; the opt-3 small-u16 list drops from 913,655.5 to 815,349.5. The main
regression is one-type opt-z decode: 2,316.5 → 2,399.5 (+83 instructions). The
worst standard opt-3 regression is fixed-u16-16: 2,754.5 → 2,776.5. Encode/reuse
instruction counts are unchanged throughout all 6,246 matched rows; every pair
preserves input/frame hashes and lengths.

Baseline is released 0.7.3 runtime with identical lock and parameters per pair:
Rust 1.99.1 / LLVM 23.1.1, Wasm, opt-z/opt-3, LTO, one codegen unit, panic abort,
stripped symbols. PocketIC 16.0.0; three samples of four repetitions (one above
64 KiB), overhead retained. No native timing claim. [Absolute total/code sizes](../evidence/short-varint-074/sizes.json),
[instruction pairs](../evidence/short-varint-074/instructions.csv) and
[source, compiler, tests, samples and replay script](../evidence/short-varint-074/raw.tar.gz)
retain reproduction evidence. Extract and run `bash replay.sh` with prepared
dependencies and explicit `POCKET_IC_BIN`. Original Wasms: `/tmp/wire-short-varint-074/`.

Validation passed: complete ic-wire package tests/doctests, focused Clippy,
Wasm/MSRV checks, and TypeScript conformance (159 vectors, 33,560 differential
mutations, 20 native-schema vectors, 54 simulated-endian cases and schema
forwarding). Real big-endian execution remains unqualified. Formatting, links
and diff checks pass. No staging, commit, push or release was performed.

## Production direction

One Serde/Postcard codec remains. Decode validates primitives in one pass;
collection ordering/uniqueness and custom Serde normalization are consumer-owned.
Retain original frames for exact retries and raw-byte signatures/hashes. Decoding
does not guarantee byte-identical re-encoding. No legacy reader or migration shim.

Use ByteBuf/Bytes for binary data and Packed<T> for supported bulk integer/hash
fields, including i8. The optional host bulk lint reports ordinary primitive
lists and supports reasoned exact-path acknowledgements with stale detection.
It does not rewrite fields. Preserve bounds and validation when adopting packed
fields; tiny fields/small varints can favor ordinary vectors.

The [0.7.1 bulk qualification](../evidence/bulk-advice-071/README.md) retains 109
cases and 2,180 matched IC rows. The [one-pass study](../evidence/one-pass-070/README.md)
and [0.6.13 full matrix](../full-comparison-0613.md) retain their historical scope.
See the [bulk guide](../packed-fields.md) and [runtime contract](../runtime.md).
Real big-endian execution remains unqualified; installed-consumer integration and
storage contracts remain consumer-owned.
