# One-pass decoding research after 0.6.13

**Research only, not a production mode or an approved contract change.** Released
base is `1003261d481a65cf3dd87b5f1e249bc82103ddbc` (0.6.13). The maintained runtime
still verifies exact reserialization. This study changes no production bytes,
admission, canonical acceptance, hashing or retry contract. New regression tests
cover normalization by custom Serde visitors, sets and character decoding.

## Proposed hard cut and concrete differences

The scratch candidate removes post-decode reserialization and uses a locally
patched Postcard 1.1.3 parser. All four varint readers reject a redundant final
zero group; the char reader requires exactly one Unicode scalar. Both retain
ordinary encoding, framing, decode admission and trailing-input rejection.
The baseline uses an identical local copy of unmodified Postcard, the same lock,
fixtures and settings. No patch or fork is added to the production workspace.

The candidate validates primitive representation, not the identity between input
bytes and serialization of the resulting Rust value. These are different contracts:

| Input and target | Current Wire | Prototype | Why a consumer could care |
| --- | --- | --- | --- |
| `02 02 14 01 0a` into `BTreeMap<u8,u8>` | Noncanonical | `{1:10, 2:20}` | Re-encoding produces `02 01 0a 02 14`, changing retry/signature/hash bytes |
| `02 01 0a 01 14` into the same map | Noncanonical | `{1:20}` | The second key overwrites the first; multiplicity is lost |
| `02 01 01` into `BTreeSet<u8>` | Noncanonical | `{1}` | Re-encoding produces `01 01`; distinct inputs collapse |
| `11` into a custom `u8 % 10` visitor | Noncanonical | Value 7 | Re-encoding produces `07` despite shortest-form input |
| `80 00` into u64 | Noncanonical | Malformed | Overlong form still rejected, with different typed error |
| `02 61 62` into char | Noncanonical | Malformed | Upstream Postcard takes the first char; the prototype instead rejects both-char input directly |
| `[01]` into u8 with two nodes | Limit | Value 1 | The additional serialization-admission pass is removed |

Normal same-type encodings in the measured corpus retain their bytes and values.
Applications retaining original bytes for retries remain able to do so. The
breaking point is relying on decode success to imply that re-encoding is identical,
or that one decoded value has one accepted byte encoding. Hashing is not removed;
raw-byte deduplication can now distinguish equivalent decoded requests. Consumers
would have to own collection/visitor normalization and any semantic signing or
idempotency rules. This proposal cannot claim to preserve that guarantee.

The maintainer allowed hard-cut investigation but requested concrete breakage
examples before choosing this semantic change. Adoption remains undecided; the
prototype is not enabled. If adopted, this is a breaking contract release, not a
patch. Parser ownership, error mapping, consumer contracts and TypeScript parity
would need qualification. Merely removing comparison from upstream Postcard is
insufficient: the varint and char checks above are necessary even for the narrower
contract. The retained Serialize API bounds are not redesigned by this prototype.

## Measurements

IC instructions per operation, opt-3; all three samples agree. These savings
compare different acceptance contracts and must not be described as a free
implementation optimization.

| Decode case | Current baseline | Prototype | Change | Candid control |
| --- | ---: | ---: | ---: | ---: |
| Empty blob | 636.5 | 402.5 | -36.76% | see CSV |
| 64-byte blob | 1,203.5 | 772.5 | -35.81% | see CSV |
| 1 MiB blob | 2,524,535 | 1,049,510 | -58.43% | 1,065,023 |
| 1 MiB mixed blob/packed/string/optional record | 2,550,066 | 1,061,694 | -58.37% | 1,479,525 |
| 16,384 wide i16 values | 4,510,280.5 | 2,052,426.5 | -54.49% | see CSV |

At opt-z the large blob drops 2,574,293 to 1,049,847 (-59.22%). All primary
encoding counters are unchanged and no successful-decode counter regresses in
this corpus. Rejection is not uniformly cheaper: truncated wide-u16 decoding
at opt-3 increases 1,610,009.75 to 1,872,155.75 (+16.28%); wide-i16 truncation
increases 13.60%. The extra minimality checks run before those truncation errors.
The prototype does not address the ordinary-vector encoding gaps.

Total / code-section uncompressed Wasm bytes:

| Cohort / profile | Baseline | Prototype | Total reduction |
| --- | ---: | ---: | ---: |
| 87 hot spots / opt-z | 88,554 / 77,598 | 83,369 / 72,425 | 5,185 (5.86%) |
| 87 hot spots / opt-3 | 113,275 / 102,926 | 104,456 / 94,114 | 8,819 (7.79%) |
| 100 ordinary records / opt-z | 324,256 / 305,505 | 310,589 / 291,865 | 13,667 (4.21%) |
| 100 ordinary records / opt-3 | 424,179 / 389,650 | 398,716 / 364,218 | 25,463 (6.00%) |

These are whole fixture artifacts, not installed application savings. No native
codec-time or gzip comparison is claimed. Mixed-case frame sizes and hashes are
unchanged across baseline/prototype; Candid keeps its own representation.

## Validation and reproduction

Rust 1.99.1 / LLVM 23.1.1, wasm32-unknown-unknown; opt-z evaluation and opt-3
benchmark-speed profiles use LTO, one codegen unit, abort panics and stripping.
Serde 1.0.229, Postcard 1.1.3, Candid 0.10.39, Platform 0.6.14, Host/Testkit
0.67.0. Testkit owns explicitly prepared PocketIC 16.1.0. No dependency update
other than the identical scratch path override on both sides. Upstream Postcard
source and license remain in the source archives; production Cargo files are unchanged.

The 87-case IC suite has 1,904 matched rows, three equal samples each. Four
repetitions through 64 KiB, one above. Setup/transport excluded; destruction and
loop overhead included. Input/frame hashes, lengths and repetitions match;
Candid controls are identical. Artifacts were rebuilt after touching local source
when changing scratch trees. The strict char fix was included in a final rebuild
before any retained IC run. The 100-record probes measure Wasm only, not IC costs.

Native acceptance probes cover all 65,536 u16 values and overlong counterparts,
u128 boundaries, invalid booleans, trailing input, normalization cases and changed
node admission. Both pass their explicitly different expected contracts. The
research example has a missing-crate-documentation warning; this is not a release
qualification. Production native tests (18) and strict Clippy pass, including the
new char/set/custom-Serde regression. Production formatting and document links pass.
No full matrix, native timing, fuzzing, TypeScript or installed-consumer claim.

[Paired counters](comparison.csv), [Wasm sizes](wasm.json) and
[raw evidence](raw.tar.gz) retain frozen sources, licenses, hashes, samples,
acceptance logs and commands. Extract and run `bash replay.sh /absolute/fresh`
with prepared `POCKET_IC_BIN`, Rust targets and cached dependencies. The consolidated
replay is syntax-checked; its constituent builds/runs were executed, but the full
replay was not executed again. Do not deploy the archived research decoder.
