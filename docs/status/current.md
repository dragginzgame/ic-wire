# Current handoff

## Current batch

Pending 0.5.20 removes the redundant finite input comparison against method bytes:
schema admission already guarantees the schema frame ceiling is no larger.
TypeScript generation now shares one safe-integer ceiling for finite metadata,
registry bounds and method allowances, including exact f64 conversion.
Protocol bytes, validation order, typed errors, owned/borrowed behavior and public
APIs are unchanged. Package versions, staging and Git refs are untouched.

Wire derives now choose generated generic and visitor identifiers absent from the
contract source. Previously, a field type named `D` or enum payload named `A` was
shadowed by a generated Serde parameter and failed compilation. Regression cases
also cover underscore-suffixed names and a visitor-name collision, asserting exact
Postcard bytes. Generated result constructors are also fully qualified: consumer
value names `Ok`/`Err` previously broke record/enum decoding compilation. The
regression checks unit/payload variants and unknown-tag rejection. No field/variant
identity or decoding rule changes. All generated library references and calls now
use absolute paths or fully qualified associated types, including the primitive
enum tag. A no-implicit-prelude regression shadows `u32` with a consumer byte alias
and checks tags above 255; consumer field types remain as declared.
Generated bindings use private prefixed names; the experimental function-import
shielding and its wrapper are removed. Record decoding projects tuple fields
instead of binding consumer field names. Enum payload types are inferred from
constructors, preserving consumer `Self` references; unit payloads remain `()`.
Regressions cover ordinary constant names, array-length constants, associated
payload types, exact bytes and malformed enums. Deliberate collisions with private
generated names are not guaranteed. Wire identity and acceptance are unchanged.

An unpublished `ic-wire-benchmarks` package now compares 114 deterministic Candid
and Wire workloads. Candid/PocketIC remain dev-only through IC Platform; IC Testkit
owns server admission/lifecycle. The former blanket simulator restriction is
narrowed to permit codec benchmarks while keeping application integration outside
this repository. Native time, allocation counts, peak heap, actual IC instructions
and linked/gzip Wasm sizes are measured separately under opt-z and opt-3.
See [methodology and results](../candid-comparison.md). PocketIC pins the shared
lockfile's `thiserror` to 2.0.18; production dependency-boundary and runtime checks
pass. Published crates gain no Candid or simulator dependency.

Wasm canonical comparison now uses 32-byte blocks and word-sized tails. The
matched corpus measures 14.0% fewer 1 MiB owned-decode instructions at opt-z
(+129 Wasm / +48 gzip bytes), and 2.38% fewer at opt-3 (+25 Wasm / unchanged gzip).
Seven small opt-3 decode cases add three instructions; protocol bytes are unchanged.
See the [retained comparison and reproduction](../candid-comparison.md#wasm-comparison-optimization--pending-0520).

Benchmark artifact discovery now follows Cargo JSON paths, including configured
target directories. Reports reject missing comparison partners, duplicate rows
and empty runs before creating CSV output. The full rerun at
`target/codec-comparison-2026-10-22-rerun/` passes: 2,352 matched comparisons and
2,838 IC result rows, all matching retained optimized counts and frame hashes.
Both profiles retain identical allocation/heap results and Wire artifact sizes.

## Evidence and qualification

`make test` and `make check-typescript` pass, including 146 vectors and 29,560
differential mutations. The latter includes strict Clippy and the generator's
Rust 1.91 library check. Formatting, Wasm compilation and documentation checks pass.
The derive follow-up passes `make test`, compile-fail diagnostics, strict Clippy
and Rust 1.91 native/Wasm checks. The separate reproducer failed before the fix
and compiles afterward; logs are in `/tmp/ic-wire-derive-hygiene/` and
`/tmp/ic-wire-result-hygiene/`. The full-path follow-up also passes these gates;
logs are in `/tmp/ic-wire-qualified-paths/`. The conventional-binding simplification
passes the same gates; `/tmp/ic-wire-conventional-derive/` retains its failing
`Self` regression and passing checks. Runtime costs
are not benchmarked for this name-resolution fix: emitted codec operations are
unchanged, and previously failing contracts had no runnable baseline. Macro
expansion now inspects the source token text; compile-time cost was not measured.

The benchmark corpus, lint, Rust 1.91 native/Wasm, formatting, local Make adapter,
document links and dependency boundaries pass. The completed run is retained at
`target/codec-comparison-2026-10-22-v1/` with source snapshots and raw samples.
All 2,838 IC result rows have agreeing repeated counts and native-frame equality.
PocketIC required localhost access outside the sandbox; its failed startup log
and successful rerun are both retained. Linux was measured; macOS was not run.
Earlier benchmark directories are pilots, not the reported evidence.

The protocol probe compared baseline `83f90a4` with the decoder change twice,
including unchanged-frame and nonminimal-input checks. Timings are mixed and the
host was not isolated; no speedup or causal regression is established. Raw samples,
absolute timings and controls are in the
[protocol study](../protocol-study.md#redundant-frame-check-cleanup-0520).
The constant refactor replaces identical compile-time values; it changes no
algorithm and has no separate generator performance claim. Full CI was not run.

Qualification remains in `ic-wire-qualification`, with `check-qualification` as its
CI gate. Older alternative-codec suites remain retired; the new Candid suite is separate.
Allocation/borrowing fixtures, compiler diagnostics, protocol and generator probes
remain maintained. Historical reports require their recorded source revisions.

## Retained decisions

Production remains bounded Postcard. Native owned content uses slice equality;
Wasm retains its word comparison. Both skip exact same-input-region comparisons
for borrowed values after length/allowance checks. Existing borrowing and the
released native shortcut remain documented in the
[protocol/layout study](../protocol-study.md).

Released 0.5.17 rejected word-reduction and early-exit native comparisons after
owned-blob regressions; see [native comparison results](../native-comparison-study.md).
Neither [bulk-reservation policy](../bulk-reservation-study.md) replaces current
growth: capacity savings came with multi-chunk or timing regressions.
[Native borrowed-view qualification](../native-views-study.md) compares existing
owned/borrowed APIs, not a new optimization. It finds 21–25% lower 1 MiB decode
plus byte-scan time and 45–46% lower sixteen-text-pair time. Blob-storage already
borrows download descriptor text; its chunk transport remains provider-specific.

Hypothetical layout cuts remain experiments. Generic single-pass canonical
decoding is unimplemented; normalized maps and arbitrary Serde semantics require
more than varint checks. Exact retries, canonical hashing and typed errors remain
required. Metadata and scalar validators are trusted declarations. Earlier
runtime evidence is in [evaluation](../evaluation.md); generator scaling evidence
is in [performance](../performance.md#typescript-generator-comparisons).
Shared Tooling is 0.25.9; IC Build's release editor is 0.63.0 with `cli`.
Ordinary agent work does not stage, commit or release.
