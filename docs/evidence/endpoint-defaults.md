# Endpoint defaults and automatic identity

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Bare `#[query]` / `#[update]` now use exported-name IDs and bounded defaults.
Both directions select 65,536 frame bytes, 8,388,608 conservative allocation
allowance, 65,536 nodes and depth 32. `id`, `request` and `response` overrides
remain independent; existing explicit settings retain their exact contract.

ID selection is the constant `native::method_id(exported_name)`: FNV-1a-64 over
UTF-8 `ic-wire:method:` plus the name. No hash computation is emitted into message
execution. Neither the codec nor framing format changes. Removing an old explicit
ID or bounds can change its contract: ID 7 / u8 42 is
`49 57 07 00 00 00 00 00 00 00 00 2a`; automatic `echo` selects
3346031142226386015 and emits `49 57 5f 5c cc 87 98 7d 6f 2e 00 2a`.
Clients must retain explicit values or coordinate ID/budget/schema adoption.
There is no old-ID fallback.

## Focused validation

`bash scripts/ci/check-endpoint.sh` passes fixed ASCII/UTF-8 identity/frame vectors,
actual contract/schema defaults and independent overrides, native compilation,
strict Clippy, Wasm/MSRV and graph isolation. The current combined source passes
11 compiler-failure subjects and a valid control, including concurrent borrowing
subjects. That work is preserved separately, not attributed to default selection.

`POCKET_IC_BIN=<prepared-server> bash scripts/ci/check-endpoint-ic.sh` passes the
existing owned endpoint cases plus bare-unit and default-ID calls, named-export
identity, wrong-ID refusal, 65,535/65,536/65,537-byte request frames, and response
boundary success/refusal with synchronous mutation rollback. Wire schemas expose
the exact generated identity and effective limits. Local logs and artifacts are
under `target/endpoint-defaults/`.

## Host generation study

This is a historical three-lane study. Compile the archived Retired raw output (see retention notice)
with its sibling Retired raw output (see retention notice) and Retired raw output (see retention notice), using the
compiler/linker setup from the [original runner](https://github.com/dragginzgame/ic-wire/blob/fcfcf5b795213cb3a0cb5857781fae7eb48f33a3/scripts/ci/evaluate-endpoint-generation.sh).
The maintained runner now compares both sides of the current API and checks token
parity; it does not reproduce this historical, one-sided omitted-settings lane.

The Retired raw output (see retention notice) is the exact mandatory-settings generator captured at
this follow-up's start. The Retired raw output (see retention notice) contains default
selection and concurrent synchronous-borrowing generator changes. The experiment
uses owned handler declarations; it cannot isolate the parser/branch cost of
those concurrent additions from defaults. No saving is attributed to defaults.

Rust 1.99.1, x86_64 Linux, harness `rustc -O`, Cargo dev-built proc-macro2/quote/Syn
libraries selected by the unchanged lock. Each of three signatures (unit, u64,
async application record/result) has eleven samples of 1,000 expansions per lane,
alternating lane order. Before/after use identical explicit arguments and handler
tokens. The additional defaults lane omits ID/budget arguments; token streams are
prepared before timing. This measures Wire's host parsing/emission and output
consumption only, excluding Platform expansion, Rust type checking and const
hash evaluation. It is not native codec time or IC instruction cost.

| Signature | Before explicit ns | After explicit ns | Change | After omitted settings ns |
| --- | ---: | ---: | ---: | ---: |
| Unit | 78,957.605 | 79,368.263 | +0.52% | 66,157.304 |
| u64 | 96,579.723 | 100,611.634 | +4.17% | 86,169.656 |
| Async record/result | 117,950.738 | 122,279.354 | +3.67% | 105,745.061 |

These are medians, including the explicit-input regressions. Host activity and the
concurrent generator work limit attribution. Retired raw output (see retention notice),
Retired raw output (see retention notice) and the exact Retired raw output (see retention notice) are retained.
The original runner snapshots both generator sources; use the archived candidate
for this historical comparison.

Default selection and identity hashing occur in constant initializers. The
message-time codec/copy/reply paths are unchanged for matched resolved settings;
no new runtime speedup or Wasm saving is claimed. Fresh native codec timings,
IC instruction comparisons and Wasm/code-section size comparisons were not run
for this selection-only change. Earlier runtime measurements remain separately
bound to the [initial extraction study](endpoints.md).
