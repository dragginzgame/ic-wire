# One-pass production decoder: 0.7.0 qualification

The maintainer approved removing reserialization after reviewing map/set/custom
Serde normalization examples. This is the implemented hard cut, not another
codec mode. The [earlier study](../strict-decoding-0614/README.md) remains historical
research; its locally patched dependency is not shipped.

## Contract and implementation

Encoding and the eleven-byte envelope are unchanged. Decode requires only
Deserialize, checks byte/allocation/node/depth limits before visited allocations,
uses a strict Postcard reader and rejects trailing bytes. It does not serialize
the decoded value or apply the encoder's traversal budget. The obsolete Compare
implementation, bulk equality helper, comparison-only tests and Noncanonical
error are removed. There is no compatibility reader or alternate runtime feature.

The private reader is adapted from Postcard 1.1.3's deserializer with its MIT
notice included in the package. Local additions reject nonminimal varints in all
four unsigned/ZigZag widths and require char payloads to contain exactly one
Unicode scalar. The generic flavor machinery is private; only byte-slice creation
is exposed inside Wire. Upstream Postcard still supplies encoding and Slice/error
helpers. No dependency override, workspace crate or public parser is introduced.

| Input / target | 0.6.13 | 0.7 behavior |
| --- | --- | --- |
| `02 02 14 01 0a` into BTreeMap<u8,u8> | Noncanonical | `{1:10,2:20}`; re-encodes `02 01 0a 02 14` |
| `02 01 0a 01 14` into that map | Noncanonical | `{1:20}`; duplicates collapse |
| `02 01 01` into BTreeSet<u8> | Noncanonical | `{1}` |
| `11` into a custom modulo-10 visitor | Noncanonical | Value 7, re-encodes `07` |
| `80 00` into u64 | Noncanonical | Malformed |
| `02 61 62` into char | Noncanonical | Malformed |
| u64 at 10 bytes/74 allocation/1 node/depth 1 | Limit | Accepted under the decode allowance |

Consumers own normalization, ordering and uniqueness constraints. Retain original
immutable frames for exact retries/raw-byte signatures/hashes; decoding no longer
promises identical re-encoding. Update exhaustive error matches and use separate
encode/decode allowances. Hash algorithms, method IDs and emitted valid bytes do
not change. TypeScript already uses strict one-pass parsing for its supported
projection; maps/custom normalization require consumer-owned projection and checks.

## IC instruction and Wasm results

Matched 87-case corpus, including 0/1-byte controls, comparison-block boundaries,
large ordinary vectors and five mixed blob/packed/string/optional records.
IC performance counter 0 includes result destruction and loop overhead, excludes
setup/transport, with three equal samples per row. Four repetitions through
64 KiB and one above. All 1,904 before/after rows retain input/frame hashes,
lengths and repetition counts; Candid counters are identical across the two runs.

| Opt-3 owned decode | 0.6.13 | Candidate | Change | Candid |
| --- | ---: | ---: | ---: | ---: |
| Empty blob | 636.5 | 402.5 | -36.76% | see CSV |
| 64-byte blob | 1,203.5 | 772.5 | -35.81% | see CSV |
| 1 MiB blob | 2,524,535 | 1,049,510 | -58.43% | 1,065,023 |
| 1 MiB mixed record | 2,550,066 | 1,061,694 | -58.37% | 1,479,525 |
| 16,384 wide i16 values | 4,510,280.5 | 2,052,426.5 | -54.49% | see CSV |

Opt-z large-blob decode falls 2,574,293 to 1,049,847 (-59.22%). All primary
encoding counters are unchanged; successful decode does not regress in this
corpus. Some malformed inputs cost more: opt-3 truncated wide-u16 increases
1,610,009.75 to 1,872,155.75 (+16.28%), and wide-i16 truncation increases 13.60%.
Minimality checks execute before those truncation errors. Ordinary-vector
encoding gaps remain; this cut does not provide implicit bulk encoding.

Total / code-section uncompressed Wasm bytes:

| Cohort / profile | Before | After | Total reduction |
| --- | ---: | ---: | ---: |
| 87 hot spots / opt-z | 88,602 / 77,581 | 83,401 / 72,408 | 5,201 (5.87%) |
| 87 hot spots / opt-3 | 113,275 / 102,926 | 104,440 / 94,114 | 8,835 (7.80%) |
| 100 ordinary records / opt-z | 324,352 / 305,537 | 310,664 / 291,897 | 13,688 (4.22%) |
| 100 ordinary records / opt-3 | 424,179 / 389,650 | 398,712 / 364,218 | 25,467 (6.00%) |

These are whole benchmark artifacts, not installed canister savings. The
100-record probes measure Wasm size only. Gzip is not an optimization objective.
Savings accompany an explicit acceptance-contract change; they are not a free
implementation optimization preserving all previous guarantees.

## Native timing

Two serial runs per variant/profile, reversing variant order on run two; each
row has 11 calibrated samples with batches of at least 2 ms. All eight runs
contain 778 rows with matching input/frame identities. Linux x86_64, AMD Ryzen
Threadripper 7970X, warm caches, unpinned CPU; no concurrent build or IC run.
Numbers below are ranges of the two run medians, not confidence intervals.
Percentages compare their midpoints. Raw samples include Candid controls.

| Opt-3 owned decode | Before ns | After ns | Midpoint change |
| --- | ---: | ---: | ---: |
| Empty blob | 59.49–69.05 | 8.62–8.91 | -86.36% |
| 64-byte blob | 30.65–37.20 | 13.97–14.94 | -57.39% |
| 1 MiB blob | 32,119.98–33,257.19 | 17,308.22–17,837.83 | -46.24% |
| 1 MiB mixed record | 33,781.70–34,324.91 | 17,494.97–17,623.69 | -48.44% |
| 16,384 wide i16 values | 102,419.50–105,536.28 | 28,168.66–28,812.90 | -72.60% |

Opt-z 1 MiB blob decode falls from 32,568.95–32,576.95 ns to
17,195.47–17,198.98 ns (-47.20%). Native truncation rejection also regresses:
opt-3 wide-u16 rises from 19,257.41–20,260.85 ns to 25,611.59–25,628.26 ns
(+29.66%), and wide-i16 rises 22.31%. The largest midpoint regression is
128 generic bytes rejected as truncated (+50.21%: 125.34–129.11 ns to
138.89–243.32 ns), with substantial run-to-run spread. Encoding also has noisy
regressions despite unchanged IC counts: opt-z i8-wide-128 encode-reuse is
330.91–355.40 ns versus 360.25–552.09 ns (+32.93%). These host timings do not
establish stable encoding savings or replace deterministic IC counters.

## Validation and identity

Base is released `1003261d481a65cf3dd87b5f1e249bc82103ddbc` plus the incoming
research regression tests/docs; production baseline code is unchanged from that
release. Both variants use the same frozen fixtures and Cargo.lock. Rust
1.99.1 / LLVM 23.1.1, Serde 1.0.229, Postcard 1.1.3, Candid 0.10.39, Platform
0.6.14, Host/Testkit 0.67.0. Wasm target is wasm32-unknown-unknown. Evaluation
uses opt-z and benchmark-speed uses opt-3; LTO, one codegen unit, abort panics,
stripped symbols. PocketIC 16.1.0 is explicit and lifecycle-owned by Testkit.
Each source tree's Rust files were touched when switching a shared build target.

Runtime/native/borrowed tests and strict all-target Clippy pass. Exhaustive u16
redundant-varint checks, wide integer boundaries, normalization, Deserialize-only
types, independent decode budgets and char/set tests pass. TypeScript qualification
passes 157 vectors, 32,156 mutations, 20 native-projection vectors and 48 simulated
big-endian cases. No_std consumer/default/schema Wasm and Rust 1.91 checks pass.
Default doctests pass. The 100,000-input ASan fuzz run passes with leak detection
disabled for the sandbox; fuzz re-encoding uses its own allowance for stable types.
The package file list includes the reader and MIT notice. Local document links,
formatting and diff checks pass. No release, push or package publication is run.

[IC comparisons](comparison.csv), [Wasm sizes](wasm.json), [native samples summary](native.csv)
and [raw evidence](raw.tar.gz) preserve sources, parameters, identities, raw
samples, artifacts, commands and logs. Extract and run
`bash replay.sh /absolute/fresh /absolute/original/repository` with prepared
POCKET_IC_BIN, Rust targets and cached dependencies. The consolidated replay is
syntax-checked; its constituent commands were executed, but the entire replay
was not repeated. Archive compression is storage only. No full 21-cohort rerun,
macOS, real big-endian or installed-consumer performance qualification is claimed.
