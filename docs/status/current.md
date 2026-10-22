# Current handoff

## Current batch

Pending 0.5.22 preserves one Postcard encoding with no size/speed selector.
Existing finite-derived, ordinary-Serde and borrowed APIs remain separately
qualified admission/ownership paths. Opt-z and opt-3 are compiler profiles of
that same implementation. Optimize code-section and total uncompressed Wasm
size, measuring instruction tradeoffs; gzip is not an acceptance criterion.
Package versions, staging, commits and release remain maintainer-owned.

Earlier retained runtime changes are Wasm native serialization/compound inlining
and shared fresh finite output allocation/header setup. Protocol bytes, bounds,
canonical validation, typed errors and public APIs are unchanged. The inlining
experiment reduces opt-3 array-manifest encode/decode instructions by 20.0%/18.8%
against 0.5.21. Shared output setup reduces the 100-type opt-3 code section by
9,300 bytes (2.49%), trading +0.49–0.68% encode and +4.23–5.66% decode instructions
against its immediately preceding baseline. Byte-level sharing was rejected after
an 82% generic-vector decode regression; rejected hints and tuple helpers are absent.
See [runtime experiments](../candid-comparison.md) and
[setup evidence](../candid-consumer-comparison.md#one-default-shared-finite-output-setup).

The small-savings follow-up keeps out-of-line Wasm header validation and
`strip = "symbols"` in benchmark profiles for both codecs. Most file savings are
metadata removal, not code headroom; consumers select stripping in their own
workspace and lose diagnostic names. Header sharing has a small instruction cost
and is not a code-size win in every profile. See the consumer report's small-savings
section and `docs/evidence/small-savings-0522/` for before/after evidence.
Bulk-comparison sharing remains rejected. The earlier full rerun includes those
changes; the guarded verification follow-up is summarized below.

## Latest Candid comparison

Guarded finite verification is now retained: direct `Bytes<N>` verifies its length
without rescanning the unchanged copied payload. Generated records delegate only
when every field preserves roundtrip boundaries. Manual Wire implementations must
keep the hidden defaults. Any custom child forces whole-record exact comparison;
ordinary native Serde, enums and container verification remain unchanged. This is
one Postcard encoding, with no caller-selectable mode or protocol change.

The unguarded prototype was rejected after a custom-field boundary-shift regression.
It could accept bytes that re-encoded differently even though each local comparison
looked equal. The whole-type guard rejects that input. The earlier bulk-loop
alternatives remain rejected; only the guarded field verification is retained.

Opt-3 1 MiB owned finite blob decode falls from 2,524,473 to 1,049,786 instructions
(−58.42%), versus Candid's 1,065,406. Frame sizes remain 1,048,590/1,048,588 bytes.
Ordinary native Serde owned blob decode remains 2,524,535; borrowing remains 824
with different ownership. One-byte finite blob decode costs 17 extra instructions
(+1.96%). The optimization does not eliminate all Candid wins.

Opt-3 Wire/Candid sizes, in bytes:

| Fixture | Code section | Total uncompressed Wasm |
| --- | ---: | ---: |
| Standard corpus | 264,914 / 788,971 | 314,728 / 860,566 |
| 100 derived types | 355,288 / 2,010,763 | 389,823 / 2,079,944 |

The 100-type code section is 8,403 bytes smaller than the preceding baseline;
standard code grows 315 bytes. The 100-type cohort has sixteen repeated structural
width patterns; whole-fixture sizes do not isolate derive code or predict app savings.
Toko and 1/10-type cohorts retain their earlier measurement boundary and were not
rerun for guarded verification. The prior full five-cohort comparison remains
historical evidence under `docs/evidence/stripped-full-comparison-0522/`.

The [latest report](../candid-comparison.md#guarded-finite-verification--pending-0522)
owns results, counterexample, scope and reproduction. Runtime tests, lint,
native/Wasm MSRV and 16 compile-fail diagnostics plus their passing control pass.
Two native runs per profile, allocation/heap passes and all eight artifact hashes
are verified under `docs/evidence/canonical-fields-0522/`. All 4,838 IC rows agree
across three samples, with unchanged input/frame hashes. All 1,844 fresh
encode/decode allocation and peak-heap summaries match baseline. Native opt-3
finite blob decode improves from 33.53–33.69 to 17.34–17.42 µs; Candid is
18.20–18.38 µs in the rerun. These are unpinned Linux observations.
Full CI and macOS were not run. Ordinary work does not stage or release.

## Retained decisions and tooling

Runtime remains Wasm-safe and independent of Candid, simulators and platform SDKs.
Candid/PocketIC are unpublished benchmark dev dependencies through IC Platform;
IC Testkit owns server admission/lifecycle. Consumers own transport, endpoints,
authentication, retry policy and database encodings. No automatic migration or
parallel legacy protocol exists. Borrowing preserves framing and canonical bytes.

Shared Tooling is 0.26.6 at `4cb9bc9`; IC Build/Testkit and transitive Host select
0.64.0. The release editor is 0.64.0 with `cli`. Their earlier adoption checks and
PocketIC driver qualification passed on Linux; see the retained evaluation reports
and `target/tooling-064-adoption/`. Historical optimization studies remain linked
from [evaluation](../evaluation.md) and [consumer comparisons](../candid-consumer-comparison.md).
Ordinary agent work does not stage, commit, push or release.
