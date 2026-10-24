# Bulk advice and signed-byte qualification (0.7.1)

Baseline: released 0.7.0, `aa7cc32acbd54a58d77720bedfb1ae530926f57e`.
This batch adds opt-in host schema advice and the missing Packed<i8> field.
It does not rewrite Serde or silently change ordinary vectors. Named metadata
roots, resolved aliases and nested containers are inspected; acknowledgements
require exact paths and nonempty reasons, and stale entries remain visible.
The example command optionally fails CI with `--deny-warnings`.

## Contract

Packed<i8> uses the existing packed implementation and Int8Array projection.
Ordinary signed-byte payloads are unchanged: [-128,127] is `02 80 7f` in both
representations. Adoption changes traversal accounting, JSON projection and
client types; regenerate clients and preserve consumer constraints. No existing
field bytes, frame identities, hashing or retry contracts change. Wider packed
fields retain their existing explicit byte-count/fixed-LE format, which differs
from ordinary integer vectors. There is no automatic conversion or legacy lane.

## Matched IC comparison

The bulk corpus now includes i8 and has 109 cases, with empty/single/small/large
controls, signed and wide patterns, hashes and mixed records. Both source trees
use the exact same expanded fixture and Cargo.lock. Baseline selects ordinary
Vec fields (`bulk,bulk-baseline`); candidate selects Packed fields (`bulk`).
Thus these numbers measure explicit field adoption, not a transparent upgrade
of existing Vec declarations. The earlier 87-case hotspot study is a different
artifact/cohort; do not combine its counters with these results.

All 2,180 paired rows have matching logical input hashes. Each has three equal
IC counter samples; four operations per sample through 64 KiB, one above. The
counter includes destruction/loop overhead and excludes setup and reply transport.
Exact encoded frames are checked against native fixtures. Candid frame hashes
and counters match across variants. Packed layouts change frame hashes/lengths
where expected; i8 keeps its bytes. Allocation instrumentation and native codec
timing were not repeated in this batch.

Opt-3 IC instructions per operation:

| Field | Operation | Ordinary | Packed | Candid |
| --- | --- | ---: | ---: | ---: |
| i8, empty | Encode | 659.25 | 651.25 | 13,496.25 |
| i8, empty | Decode | 304.5 | 351.5 | 14,960.5 |
| i8, 1 value | Encode | 717.25 | 761.25 | 13,608.25 |
| i8, 1 value | Decode | 696.5 | 755.5 | 15,249.5 |
| i8, 128 values | Encode | 7,090.25 | 917.25 | 14,878.25 |
| i8, 128 values | Decode | 6,936.5 | 917.5 | 24,657.5 |
| i8, 16,384 values | Encode | 820,017.25 | 17,298.25 | 47,643.25 |
| i8, 16,384 values | Decode | 667,757.5 | 17,434.5 | 1,244,403.5 |
| Wide i16, 16,384 values | Encode | 3,212,184.25 | 33,823.25 | 80,505.25 |
| Wide i16, 16,384 values | Decode | 3,035,145.5 | 33,973.5 | 1,408,288.5 |
| Wide u64, 16,384 values | Encode | 6,702,148 | 132,307 | 277,329 |
| Wide u64, 16,384 values | Decode | 6,722,205 | 132,445 | 1,392,055 |

Large i8 encoding drops 97.89%, decoding 97.39%, with unchanged 16,398-byte
Wire frames (Candid 16,396). Small-input regressions remain: single-i8 encode
+6.13%, decode +8.47%; empty-i8 decode +15.44%. Across this corpus, the largest
successful-operation regression is empty fixed-word decoding at +19.58% opt-3.
Every selected packed encode/decode case costs fewer instructions than Candid
in both profiles; the highest ratio is 0.4772. This is not a universal guarantee
for arbitrary payloads, nor a claim that ordinary-vector gaps disappeared.
Malformed rejection counters are also retained in the CSV, without equating
rejection work across representations with different bytes/budgets.

## Wasm and wire-size tradeoffs

Total / code-section uncompressed Wasm bytes for the matched bulk artifacts:

| Profile | Ordinary Wire | Packed Wire | Candid |
| --- | ---: | ---: | ---: |
| opt-z | 104,837 / 93,622 | 98,377 / 87,195 | 442,247 / 407,050 |
| opt-3 | 136,393 / 125,879 | 122,614 / 112,140 | 679,957 / 645,666 |

Packed adoption reduces total Wasm by 6,460 bytes (6.16%) at opt-z and
13,779 bytes (10.10%) at opt-3. These are whole benchmark artifacts, not installed
application savings, and include adoption of all supported packed fields.
An unchanged 100-record consumer control produces byte-identical before/after
Wasm: 398,712 total bytes at opt-3. Merely adding the unused i8 implementation
and host lint has no size effect in that control. Gzip is not an objective.

Wire size is field-dependent: 16,384 wide i16 values fall from 49,166 to
32,782 frame bytes; wide u64 falls from 163,854 to 131,086. Small-value u64
vectors can instead grow toward eight bytes per element. The lint reports
candidates, not promised wins; preserve bounds and measure consumer payloads.

## Host lint cost

[Host probe](host.rs), opt-3, Linux x86_64, Threadripper 7970X. It times unchanged
registry generation with and without optional linting, including validation and
report construction. Each record has one i64 list and one byte list; inputs are
constructed before timing. Eleven samples, 100 operations per sample, alternating
order, warm unpinned CPU; no concurrent build or IC run. Medians:

| Schema roots | Generation | Generation + lint | Extra host time |
| --- | ---: | ---: | ---: |
| 1 | 2.536 µs | 3.498 µs | 0.962 µs (+37.91%) |
| 10 | 21.445 µs | 32.388 µs | 10.943 µs (+51.03%) |
| 100 | 257.943 µs | 376.821 µs | 118.878 µs (+46.09%) |

This is additive host work, not native codec or IC instruction cost. The lint
reuses metadata validation and has no Wasm feature dependency. The existing
generator does not invoke it automatically; consumers choose their CI policy.

## Validation and reproduction

Native/lib tests, strict all-target Clippy and default doctests pass. Coverage
includes all 256 signed-byte values, byte parity, bulk budgets, malformed lengths,
unaligned input, nested aliases, named recursion, invalid metadata, stale/empty
acknowledgements and CLI exit codes. TypeScript passes 159 vectors, 33,560 mutation
checks, 20 native projection vectors and 54 simulated-endian cases. Real big-endian
execution remains unqualified. Native/no_std/Wasm feature forwarding and Rust
1.91 checks pass. ASan fuzzing passes 100,000 inputs with leak detection disabled
for the sandbox, including the new signed-byte variant. No sibling is changed.

Rust 1.99.1 / LLVM 23.1.1, Wasm target wasm32-unknown-unknown, Serde 1.0.229,
Postcard 1.1.3, Candid 0.10.39, Platform 0.6.14, Host/Testkit 0.67.0. Evaluation
uses opt-z; benchmark-speed uses opt-3; LTO, one codegen unit, panic abort and
stripped symbols. PocketIC 16.1.0 is explicitly prepared and Testkit-owned.
Source switching touches local Rust files before reusing the target directory.

[IC results](comparison.csv), [artifact sizes](wasm.json), [host samples](host.csv)
and [raw evidence](raw.tar.gz) retain identities, sources, artifacts, samples,
logs and reproduction scripts. Extract the archive and run
`bash replay.sh /absolute/fresh-directory` with prepared POCKET_IC_BIN, toolchain,
Wasm target and cached dependencies. The consolidated replay is syntax-checked;
its constituent commands were executed but the entire replay was not repeated.
Host probe documentation was corrected after timing; executable logic is unchanged.
No release, staging, commit, push or package publication was performed.
