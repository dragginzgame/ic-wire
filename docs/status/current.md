# Current handoff

## Released 0.7.11; pending 0.7.12

Baseline is `a99a99cc3fadbde004e69d9ee4888c99711b3bee`. The preceding uncommitted
adversarial test, report and changelog work is preserved. The follow-up adds
private measurement/qualification fixtures and rejects another parser candidate.
Production runtime, protocol bytes, identities, admission rules and generated
TypeScript are unchanged. Staging, commits, own versions and releases remain
maintainer-owned.

## Issue #26: unsigned evidence

Documented minimal unsigned big-endian magnitudes over existing `Bytes<N>`:
zero is empty bytes, and consumers reject redundant leading zeroes before
constructing their arithmetic type. No new Wire numeric wrapper or metadata kind
is needed. [The guide](../unsigned-magnitude.md) includes lossless external Nat
conversion and browser bigint helpers, with explicit field/admission boundaries.

The private Nat test checks None, zero, u128 maximum, 2^128, 2^256 and the admitted
512-bit maximum, exact magnitude bytes, over-limit refusal and malformed forms.
Generated metadata/browser vectors check the same option DTO in both directions.
Candid/BigUint remain unpublished test dependencies; published runtime and
generator code are unchanged. No new performance benchmark is applicable to this
documentation/qualification change. Canic must adopt the documented projection
and preserve its overflow classification, status/error union and external Candid.
Focused Nat conversion tests pass on Rust 1.99.1 and 1.91; Clippy passes.
`check-typescript` passes 165 exact-frame vectors and 35,228 differential
mutations, including native/Wasm feature isolation. Issue closure/publication
and installed consumer qualification remain outstanding.

## Warning integration

A disposable Cargo build-script consumer now checks default warning visibility,
independent per-field suppression, complete acknowledgement and stale paths.
It passes on Rust 1.99.1 and 1.91.0, and runs inside `check-typescript`.
Run it alone with `bash scripts/ci/check-bulk-advice.sh`; logs remain under
`target/bulk-advice/`. No extra warning feature or canister logging was added.

Canic adoption remains separate: the prior read-only inspection found six old
three-argument generator calls despite a 0.7.11 lock selection. No sibling was
edited or built. See the [caller inventory](../evidence/followup-0712/README.md)
and [migration examples](../packed-fields.md#host-side-bulk-advice-and-ci).
The fixture qualifies the integration pattern, not installed Canic behavior.

## Rejection budgets

The new `admission-wasm`/`admission-instructions` examples measure four explicit
budget policies on ordinary u8/u64 vectors and byte buffers. Across 144 valid and
truncated configurations, three repeated IC samples agree and host/Wasm outcomes
match. All valid 0/1/64-element controls pass every policy.

In this dedicated fixture, a truncated million-element ordinary byte vector costs
69,211,213 instructions with generous limits, 36,030 with 1,024 nodes, and
272,998 with 64 KiB allocation headroom. Both tighter settings also refuse the
valid large ordinary vector. The node policy still accepts a valid 1 MiB byte
buffer. A 64 KiB frame cap refuses oversized input in 226 instructions.
These are illustrative tradeoffs, not automatic defaults or Candid-equivalent
quotas. Do not compare this fixture's absolute counters against the broad suite
as a runtime regression. See [selection guidance](../packed-fields.md#budget-selection)
and the [study and reproduction](../evidence/tail-and-budgets-0712/README.md).

The preceding adversarial test verifies nested budgets and forged-length prefix
work. It found no admission bypass. Generic remaining-byte checks cannot reject
all large collection counts because unit elements require no encoded bytes.

## Parser experiments rejected

Sharing entire varint readers saved 10,916 bytes in one fixture but regressed
small-u16-list decode by 84.4%. Sharing only multi-byte tails saves 4,966 total
bytes in standard opt-3, but grows the other three fixture/profile combinations.
Small/wide u16-list decode still regresses 24.1%/38.3%; some small controls regress
as well. Neither candidate is installed in the production library.

The tail candidate has fresh counters for 298 standard cases / 7,006 operation
rows; identical baseline Wasm reuses the preceding measured counters. Input/frame
hashes match and Candid control counters are unchanged. The 100-type experiment
is size-only. The broader [full comparison](../full-comparison-0710.md) remains
historical qualification; this is not a new full matrix or native-time study.

## Checks and boundaries

Focused warning checks pass on both toolchains. New measurement examples pass
strict Clippy, native Rust 1.91 and Wasm Rust 1.91 checks. Formatting and local
links pass; full CI, browser conformance and broad sibling suites were not rerun
for this fixture-only follow-up. The preceding runtime/adversarial checks remain
recorded in their report. One Serde/Postcard codec remains; consumers retain
normalization, transport, identity coordination and exact retry/signature bytes.
See the [runtime contract](../runtime.md).
