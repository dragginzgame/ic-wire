# Current handoff

## Scope

The maintainer selected a codec-only repository on 2026-10-19. Endpoint SDKs,
simulators, installed canister drivers, browser transport, the private adapter
crate and format-comparison evidence are retired. Native codec controls,
compiler diagnostics and standalone Rust/TypeScript byte conformance remain.
Published runtime APIs, wire bytes, limits and package versions are unchanged.
A subsequent maintainer request authorizes the bounded
[MessagePack/Postcard experiment](../messagepack-comparison.md); only its pure
codec fixture/native driver are maintained here. External IC probes stay outside
the workspace. Production encoding remains Postcard.
The maintainer has released 0.4.8 (`a5f3cd6`). Pending compatible notes are
0.4.9; this task executes no release or publication.

Source starts at `a3e6577` with the pending license-metadata repair preserved.
Shared Tooling remains at reviewed 0.24.2; its generic scripts/catalogs and guides
are intact. Product setup no longer selects its platform executable bundle.
The full pre-change source, retired reports, browser dependencies and previous
build artifacts are preserved outside this checkout at
`/tmp/ic-wire-format-removal.hbmMHv/`.

## Maintained qualification

The Blob adoption hard cut is delivered independently in
[PR #90](https://github.com/dragginzgame/ic-blob-storage/pull/90), commit `2967298`,
from `/tmp/ic-blob-wire-cut`. All owned methods/lifecycle use Wire; installed
client/browser and recovery checks pass against local substitutes. Wire gains 1 MiB finite/native
Rust and generated TypeScript conformance cases, with exact frame equality,
truncation, identity/direction, trailing bytes, noncanonical lengths and allocation
refusal. These focused cases pass; the consumer uses published Wire 0.4.4 without
new runtime functionality. Provider transports and persistent records remain
consumer contracts.

See [local development](../development.md), [evaluation scope](../evaluation.md),
[runtime contract](../runtime.md) and [native timing controls](../performance.md).
The evaluation package stays unpublished. IC Build retains ownership of compiler
diagnostic execution. Private raw Postcard/bitcode/wincode controls retain their
different guarantees; they are not production modes or installed measurements.

Focused Linux checks pass: 22 runtime/native tests and seven doctests, native
raw-codec roundtrips, strict evaluation Clippy and Rust 1.91 host checking,
twelve compiler-diagnostic subjects/control, ninety TypeScript vectors and
17,524 differential mutations, runtime Wasm/floor checks, formatting, pins,
dependency boundaries, snapshot integrity and documentation links. Disposable
Make/release fixtures and common-tool admission pass without platform executables.
All-feature/all-target dependency inspection retains 79 original package
identities and prunes 234 packages. Both offline archives retain the exact MIT
LICENSE; archive preparation used `--no-verify` and uploaded nothing.

The native timing driver runs all 63 case/lane pairs and refuses existing output
without overwriting it. Its final report is `target/native-performance-verified/report.json`;
lock/case/driver hashes match the checked source. Earlier draft failures and
focused logs remain under `/tmp/ic-wire-removal-*`. No full local CI, installed
endpoint, native macOS, compressed size or IC instruction qualification is claimed.

## Preserved contributor work

All packages inherit only the MIT SPDX license. Publishable crates include the
root LICENSE as identical package-local regular files. The formatting hook refuses
tracked symlinks; the local fixture now preserves file types so it exercises that
admission rule. Earlier locked metadata and offline
archive checks passed without duplicate-license warnings; both archives contain
the exact root text. Existing packaging warnings concern excluded tests/examples.

The local Make fixture selects authored crate inputs through Git, including
untracked modules and excluding ignored output. Shared Tooling owns the formatter
and hook implementation. The fixture confines all Git/index/release-routing
effects to disposable repositories.

## Blob contract qualification follow-up

[Blob PR #91](https://github.com/dragginzgame/ic-blob-storage/pull/91), commit
`61df9b6`, extends PR #90 with
frozen DTO/method vectors, application DTO trait separation, preserved local
historical/record schemas and native/installed codec measurements. Integration
and evidence stay in the consumer repository. Canic adoption remains separate.

Wire's only runtime change is exact XOR/OR accumulation over canonical comparison
words on Wasm (and its owning unit tests). At Rust 1.99.1, release Wasm, the shared
1 MiB opaque-byte control falls from 4,195,809.5 to 2,885,112.5 decode instructions
per iteration. Manifest decode costs increase by 202–226 instructions (at most
0.46%); frames and admission charges are identical. This does not measure Blob
HTTP transfer, end-to-end latency, linked size or compressed deployment size.
Native code retains its existing comparison. Raw consumer reports and the
isolated candidate source remain under `/tmp/blob-codec-comparison` and
`/tmp/blob-*-baseline*.json`, `/tmp/blob-*-candidate.json`.

Concurrent maintainer changes to IC Build selection and Cargo.lock are preserved
and excluded from this task's source metrics.

Focused Wire checks pass: 24 unit/integration cases, seven doctests, runtime
Clippy, Wasm and Rust 1.91 host/Wasm compilation, 90 TypeScript vectors and
17,524 differential mutations, including the 1 MiB conformance cases. Dependency
boundaries and documentation links pass. Blob independently passes its DTO,
installed storage/standalone, frozen-record/image and Chromium checks; detailed
evidence is retained in PR #91. No full local CI or native macOS run is claimed.

## MessagePack experiment

The maintainer-requested [comparison](../messagepack-comparison.md) is complete:
27 cases, raw Postcard/MessagePack arrays/maps and bounded Wire, two optimization
profiles, native samples, external PocketIC instruction counts and independently
linked/gzipped Wasm probes. All native/IC/profile input hashes agree. Keep Postcard
for the current Blob-shaped contract; wide random integer vectors are a measured
MessagePack exception. Binary hashes are explicitly a separate schema experiment.
No production format, version or runtime implementation changed in this batch.

The native rerun is `make evaluate-messagepack`; `check-evaluation` owns the
roundtrip and representation probes. Focused tests, strict Clippy, Rust 1.91 host,
formatting, dependency boundaries and documentation links pass. No full CI or
native macOS benchmark was run. Raw reports/CSV/identities are checked in under
`docs/evidence/messagepack-2026-10-19/`; exact local sources, external harnesses,
Wasms and logs remain under `target/messagepack-2026-10-19/`. Preserve the external
harness archive for IC reproduction; it is deliberately outside workspace deps.
Incoming runtime optimization and ic-build dependency work remain preserved.

## Wasm optimization follow-up

[Measured Wasm optimizations](../runtime-optimization.md) retain targeted budget
inlining and wider exact canonical comparisons for bulk writes. The current
1,024-array-hash manifest uses 39%/36% fewer encode/decode instructions at opt-z
and 30%/7.5% fewer at opt-3; opt-z 1 MiB decode falls 26%. Probe Wasm grows about
2.2–2.4 KB. Speed-profile binary-hash encode and some short-buffer cases regress;
full results are retained. Native inlining attempts were rejected, so final hints
apply only to wasm32. Bytes, APIs, admission arithmetic and versions are unchanged.

24 runtime tests, seven doctests, runtime Clippy, Rust 1.91 host/Wasm checks and
90 TypeScript vectors plus 17,524 differential mutations pass. Baseline and
candidate sources, Wasms and logs remain at `target/runtime-optimization-2026-10-19/`;
raw IC results and identities are in `docs/evidence/runtime-optimization-2026-10-19/`.
No full CI, native macOS or whole-consumer deployment check was run.

## Borrowing and allocation qualification

[Measured tradeoffs](../allocation-comparison.md) cover 44 cases under opt-z/opt-3
on native and external IC probes. Native decode now borrows input; reusable output
APIs retain canonical checks and admission. Finite Wire remains owned. Packed
consumer layouts have distinct identities and a tested borrowed array projection;
no Blob schema/deployment or production encoding changes are made here. Borrowing
saves about 10% of large-byte IC decode-plus-scan; packed hashes show much larger
consumer-schema gains. Reuse retains high-water capacity and has no substantial
combined encode/decode win. Finite maxima can waste memory on small messages.

Focused runtime/evaluation tests, strict Clippy, Rust 1.91 host/Wasm and the
90 TypeScript vectors/17,524 differential mutations pass. Raw evidence is under
`docs/evidence/allocation-2026-10-19/`; source/harness/lock/measurement archives
are under `target/allocation-2026-10-19/`. Native macOS and full CI were not run.
Concurrent IC Build 0.21 dependency edits are preserved and excluded from this
task's source metrics. No staging, commit, release or publication was performed.

## Bounded allocation follow-up

The [focused study](../bounded-allocation-study.md) selects capped geometric growth
for hidden-length lists: 4,096 u32 elements drop from 4,095 reallocations to 12
and native decode improves 36.7%. Existing bytes and public APIs are unchanged.
Exact-size finite encoding is rejected as a blanket default: empty-byte IC
encoding improves 96.4%, but full array manifests regress 37.1% at opt-z.
An isolated borrowed finite contract compiles using the existing Method trait;
production derive/lifetime qualification remains outside the shipped change.

Runtime tests, Clippy, Rust 1.91 host/Wasm and Rust/TypeScript conformance pass.
Compact measurements are in `docs/evidence/hardcut-2026-10-19/`; full source, raw
reports and external probe artifacts remain in `target/hardcut-2026-10-19/`.
No full CI, native macOS, staging, commit or release was performed.

## Borrowed canonical comparison

The [follow-up study](../bounded-allocation-study.md#follow-up-where-a-hard-cut-would-help)
selects a Wasm-only same-slice comparison shortcut: large borrowed decode-plus-scan
uses 12% fewer instructions at opt-z and 17% fewer at opt-3. Native executable
code remains identical; the initial all-target candidate was rejected after
native regressions. Owned short-message overhead is small but measured.
The borrowed finite-contract hard cut saves only 147 further instructions on the
1 MiB case and remains a prototype. No public API or wire-format cut is selected.

Runtime tests, Clippy, Rust 1.91 host/Wasm and TypeScript conformance pass. Compact
results are in `docs/evidence/borrow-savings-2026-10-19/`; full raw/source/probe
artifacts remain in `target/borrow-savings-2026-10-19/`. Full CI/macOS were not run.
