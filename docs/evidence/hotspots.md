# Account and fragmented-blob hotspots

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

The [rebased Core 0.15.1 / Types 0.17 preparation](account-batch.md)
supersedes this page's historical pending-version patches. Measurements below
remain bound to their original sources.

Prepared on 2 November 2026. The maintainer approved the coordinated Account
contract cut. The patches below are source-qualified preparation, not activated
registry adoption: normal Wire still uses published Types 0.16's array schema.
No sibling working checkout, package version, Git release or registry changed.
Compiler adoption is qualified [separately](README.md).

## Selected Account change

Use the standard `serde_bytes` adapter on Platform Types' existing
`Option<[u8; 32]>` subaccount. Wire describes that field as optional bounded bytes
with minimum and maximum 32, using the existing schema trait. Principal remains
bytes(max 29); owner/subaccount order, Rust field types, account equality/hash/text
and absent versus explicit zero are preserved. External Candid DTOs are unchanged.

This is a binary contract break. The present field changes from `01` followed by
32 bytes to `01 20` followed by those bytes; absence stays `00`. Coordinate the
supplying Types release, Wire schema, client generation and contracts. Requalify
budgets, and retain original retry/signature/hash inputs rather than re-encoding
them. There is no legacy reader or migration lane. Other Serde formats must
qualify the adapter too; omitted optional map fields still deserialize as None.

Reviewable patches:

- Retired raw output (see retention notice),
  based on committed Platform `3cae1e552e17d7533a99bc5fd2a1c2b7e08021ec`.
- Retired raw output (see retention notice),
  against the normal current schema/fixture. Apply only with the supplying Types.

Types 0.17 is the preparation's pending ledger, not a claim that it is published.
The Retired raw output (see retention notice)
still lists 0.16.0 as the newest release at this qualification's final check.
Activation requires the published adapter and a matching single canonical Types
identity through Runtime/Endpoint. Then rerun installed-consumer and endpoint
runtime-supplier qualification on that registry graph. Temporary Cargo patches
exist only inside isolated qualification sources; none is required downstream or
added to normal Wire. No upstream PR was opened by this implementation request.

Subsequent supplier adoption selects published Types 0.17, which retains the
original array encoding and does not contain this adapter. The source patch and
its pending-0.17 ledger are historical preparation inputs; rebase them onto a
later maintainer-selected Types release before adoption. The recorded prototype
and actual-supplier measurements remain bound to their frozen sources.

## Actual supplier measurements

The paired supplier study uses real Platform Accounts, with no stand-in DTO on
the Wire side. Both sides select identical dependency declarations, including
normal `serde_bytes`; the baseline retains the released tuple representation.
Measured source/supplier hashes, fresh compiler artifacts, input/frame identities
and all Candid instruction controls are checked by the analyzer. All Candid
counters and frames match exactly in this actual-supplier study.

There are 23 timed/counted Account, Principal-map and Account-set cases, including
empty/singleton/16/128 controls. The complete 58-case guest remains reachable for
Wasm measurement. Wire wins 85/92 primary IC comparisons, including every decode.
Seven encode comparisons still favor Candid; no blanket parity claim is made.

Opt-3 instructions per operation and complete frame bytes:

| Workload | Before Wire | Prepared Wire | Candid | Change |
| --- | ---: | ---: | ---: | ---: |
| 128 explicit-zero Accounts, encode | 482,927.25 | 122,726.25 | 104,255.5 | −74.59% |
| Same, decode | 569,479.5 | 104,711.5 | 1,636,131.5 | −81.61% |
| Account set, 128 entries, encode | 359,897.25 | 116,308.25 | 107,161.25 | −67.68% |
| Same, decode | 575,686.5 | 263,697.5 | 1,563,713.5 | −54.19% |
| Absent subaccount scalar, encode | 964.25 | 976.25 | 35,595.25 | +1.24% (+12) |
| Absent subaccount scalar, decode | 949.5 | 871.5 | 48,105.5 | −8.21% |
| 128 explicit-zero Accounts, bytes | 8,077 | 8,205 | 8,349 | +128 bytes |

The large Account encode ratio falls from 4.63× to 1.18× Candid. Opt-z encode
falls 737,914.5 → 285,178.5 (−61.35%), versus Candid's 283,819 (1.005×).

| Profile | Wire total Wasm before → after | Wire code before → after | Candid total / code before → after |
| --- | ---: | ---: | ---: |
| opt-z | 226,963 → 224,729 (−2,234) | 193,213 → 190,983 (−2,230) | 688,456 / 629,007 → 688,212 / 628,762 |
| opt-3 | 297,363 → 294,984 (−2,379) | 265,780 → 263,405 (−2,375) | 996,088 / 938,528 → 996,854 / 939,293 |

These are total uncompressed modules and independently parsed section-10 payloads
for the complete benchmark guest, including setup/reporting. Candid artifacts can
change because supplier setup remains reachable; its counted codec work is
unchanged. These are not application linker savings. Gzip size is not a metric.

Native opt-3 Account-list encode medians, two runs: 10,581.96 → 2,399.30 ns
(−77.33%) and 10,494.53 → 2,274.64 ns (−78.33%). Decode: 9,655.61 → 3,692.74 ns
(−61.76%) and 9,647.91 → 3,552.93 ns (−63.17%). Encode allocations remain 1/0
allocations/reallocations, decode 1/5. Encode peak live bytes rise 8,077 → 8,205;
decode stays 8,064. All cases, small controls and regressions are retained in
Retired raw output (see retention notice), Retired raw output (see retention notice),
Retired raw output (see retention notice),
Retired raw output (see retention notice) and Retired raw output (see retention notice).

## Other experiments and decisions

The independent 58-case study retains Retired raw output (see retention notice),
Retired raw output (see retention notice), Retired raw output (see retention notice),
Retired raw output (see retention notice) and Retired raw output (see retention notice).

| Experiment | Total / code Wasm delta, opt-z; opt-3 | Decision |
| --- | --- | --- |
| Scalar writer inlining | 0 / 0; 0 / 0 | Reject: no improvement |
| Fuse depth checks | −172 / −172; −38 / −38 | Reject: Account batch opt-z encode +4.68% for tiny savings |
| Pass depth validity into budget check | −31 / −31; +49 / +49 | Reject at size screen; no timing claim |
| Account byte DTO prototype | −1,993 / −1,960; −2,960 / −2,930 | Research only; actual supplier study above supersedes it |
| Borrow blob views, copy after validation | +506 / +453; −203 / −251 | Consumer pattern only; keep owned default |

The depth candidate passes the existing 5% + 64 instruction screen, but that is
insufficient reason to retain its hotspot regression. No generic encoder change
is kept. Prototype Account measurements must not be substituted for the actual
supplier results: two opt-z Candid set-decode counters drift in that research run,
recorded explicitly in the summary.

### Follow-up: sharing fixed helpers across derived types

Four further independent Wasm-only outlining hints were screened against current
unchanged production codec source. None passes the footprint objective, so none
is kept or advanced to native timing/IC instruction qualification. This screen
changes no production bytes, admission budgets, acceptance or runtime sources.

Total / code byte deltas; `cold-error` applies cold + noinline to Budget::error,
and the others apply noinline to the named existing helper:

| Helper | 20 controls opt-z | 20 controls opt-3 | 100 types opt-z | 100 types opt-3 |
| --- | ---: | ---: | ---: | ---: |
| Error handling | +159 / +158 | −78 / −79 | −36 / −37 | +8,332 / +8,331 |
| Header construction | 0 / 0 | −713 / −714 | 0 / 0 | +52 / +51 |
| Frame-limit adjustment | +1,889 / +1,887 | +1,343 / +1,339 | +3,491 / +3,490 | +11,774 / +11,773 |
| Budget initialization | +1,969 / +1,951 | +2,140 / +2,122 | +3,719 / +3,702 | +11,672 / +11,671 |
