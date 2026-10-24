# Candid-inspired decode follow-up

No runtime candidate is retained. The existing 64-byte canonical comparison
blocks remain selected: larger blocks save some large-buffer instructions but
increase Wasm size and regress small buffers. Production encoding, admission,
canonical checks, hashing and retry bytes are unchanged. The maintained hotspot
corpus expands from 50 to 82 cases to expose these boundary regressions.

## Audit

Candid's [blob fix](https://github.com/dfinity/candid/pull/194) supports our existing
Serde byte-buffer path. ByteBuf decoding already makes one owned copy after byte
admission; borrowed buffers retain the input. Packed decoding also reserves once
after width validation. There is no duplicate owned buffer to remove here.

Candid's [batched vector accounting](https://github.com/dfinity/candid/pull/716)
has its analogue in Wire byte-oriented fields: they charge the buffer once.
Ordinary sequences still require per-element admission because arbitrary Serde
visitors determine element types and nested work. Candid's
[length hints](https://github.com/dfinity/candid/pull/570) cannot simply be
forwarded: Wire intentionally suppresses hints before element admission to stop
untrusted counts from causing early allocation. No admission change is proposed.
Candid's option/type-environment cloning fix does not apply to Wire's decoder.

The remaining large owned-blob gap is principally exact canonical verification.
The previous [ablation](../../../cost-breakdown-0611.md) measured its cost;
removing it would change the custom-Serde acceptance contract. This experiment
changes only the comparison loop's block size, retaining the exact comparison.

## Measured results

IC instructions per owned blob decode, including destruction and loop overhead:

| Payload / profile | Baseline 64-byte blocks | 128-byte blocks | Change | 256-byte blocks | Change |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 MiB / opt-z | 2,574,059 | 2,467,563 | -4.14% | 2,414,315 | -6.21% |
| 1 MiB / opt-3 | 2,524,402 | 2,442,491 | -3.24% | 2,401,531 | -4.87% |
| 128 bytes / opt-z | 1,810.5 | 1,797.5 | -0.72% | 2,040.5 | +12.70% |
| 64 bytes / opt-z | 1,457.5 | 1,572.5 | +7.89% | not run | — |
| 127 bytes / opt-3 | 1,408.5 | 1,492.5 | +5.96% | not run | — |

Empty and singleton blob costs are unchanged in both profiles. Large generic
byte vectors do not benefit: they still serialize individual bytes. The expanded
82-case run also catches 192/193/255-byte regressions with 128-byte blocks.
Candid's 1 MiB owned-blob decode control is 1,073,455 instructions at opt-z and
1,064,976 at opt-3; even the rejected candidates remain over twice that cost.
The Wire/Candid frames remain 1,048,590 / 1,048,588 bytes for that case.

Total / code-section uncompressed Wasm bytes for the 50-case screen:

| Profile | Baseline | 128-byte blocks | 256-byte blocks |
| --- | ---: | ---: | ---: |
| opt-z | 76,520 / 66,243 | 76,682 / 66,405 | 77,002 / 66,725 |
| opt-3 | 97,507 / 87,860 | 97,766 / 88,119 | 98,086 / 88,439 |

These are whole fixture artifacts, not installed-application savings. No gzip
objective or Wasm savings claim. Native timing was not rerun: the modified bulk
branch is Wasm/test-only, and both candidates were rejected on IC costs and size.

## Identity and reproduction

Base is f470bcd9 plus the incoming dirty 0.6.13 work, including ByteBuf and the
retained i16 writer. The archive freezes that source; comparisons are against the
same working-tree baseline, not the pristine commit. Rust 1.99.1 / LLVM 23.1.1,
wasm32-unknown-unknown; evaluation opt-z and benchmark-speed opt-3, LTO, one codegen
unit, panic abort, stripped symbols. Same lock: Candid 0.10.39, Postcard 1.1.3,
Serde 1.0.229, Platform 0.6.14, Host/Testkit 0.67.0, PocketIC 16.1.0.

Three equal samples per row, four operations/sample through 64 KiB, one above;
setup and transport excluded. All 4,004 before/candidate row pairs validate exact
input/frame hashes, lengths and repetition counts. Candid counters are unchanged.
The 50-case screen reused the prior hotspot Candid binaries and native oracle
with the same lock/fixtures; their original sources are archived. The 82-case
boundary driver and Candid binaries were rebuilt. All Wire variants were rebuilt,
touching local Rust inputs when switching source copies in the shared target.

[Screen comparisons](comparison.csv), [boundary comparisons](boundary-comparison.csv),
[Wasm sizes](wasm.json) and [raw evidence](raw.tar.gz) retain the results.
Extract the archive and run `bash replay.sh /absolute/fresh/scratch` with prepared
`POCKET_IC_BIN` and cached dependencies. Replay rebuilds all drivers and Candid
controls from the frozen source; the measured commands/logs remain alongside it.
The consolidated replay was syntax-checked; its constituent builds/runs were
executed, but the consolidated replay was not run a second time.

The retained 82-case corpus passes roundtrip/damaged-frame checks and strict
Clippy. The screen includes header/truncation rejection controls, but rejected
runtime candidates have not been qualified for release. No broader matrix,
TypeScript, macOS or installed-consumer qualification is claimed for this research.
