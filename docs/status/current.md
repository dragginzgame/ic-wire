# Current handoff

## Current batch

Released 0.5.16 (`24c1250`) is the baseline. Pending 0.5.17 records two rejected
native canonical-comparison experiments, two bulk-reservation experiments, and
native borrowed-view qualification and benchmark tooling improvements.
No production Rust code changes.
Protocol bytes, identities, limits and malformed-input acceptance are unchanged.
Package versions, dependencies, staging and Git refs remain untouched.

The word-reduction candidate enables the existing Wasm/test implementation on
native hosts. It regresses owned 4 KiB decode about 137% and 1 MiB decode 74–84%.
The early-exit chunk candidate regresses those workloads 77–78% and 33–37%.
Both keep the borrowed same-input-region shortcut. Neither is selected.

Exact bulk reservation saves capacity for single blobs but increases four-chunk
capacity 50%. Adding 32 bytes of headroom avoids that counterexample and saves
75%/50% capacity for 256/512 KiB blobs under a 1 MiB bound. However, 512 KiB
encoding regresses 5–11%; neither allocation policy replaces the default.

## Evidence and qualification

[Native comparison results](../native-comparison-study.md) contain paired absolute
timings, small/integer/borrowed controls, all samples and reproducible patches.
Both candidates were built in disposable snapshots and run twice, with reversed
baseline/candidate order on the second run. The same lockfile was used in each
lane; CPU affinity was not fixed. The probe's canonical roundtrip, nonminimal
integer and baseline/candidate frame comparisons passed. Timing gains are not
claimed; no IC, Wasm-size, browser or macOS measurement was made.

Raw builds and binaries are in `/tmp/ic-wire-native-word/` and
`/tmp/ic-wire-native-early/`, candidate sources in `/tmp/ic-wire-native-compare/`.
The runner's optional-file branch was exercised by both complete experiments.

[Bulk-reservation results](../bulk-reservation-study.md) include encode timings,
returned Vec capacities, all samples and candidate patches. The extended probe
adds threshold-adjacent blobs, four/sixteen-chunk lists and `.capacity` outputs.
Both candidates passed the probe and unchanged-frame checks in two paired runs;
capacity is not allocator footprint or peak memory. Raw evidence is in
`/tmp/ic-wire-bulk-results/` and `/tmp/ic-wire-bulk-slack/`.
Shell syntax, documentation links and whitespace checks pass. Full runtime suites,
full CI and hosted macOS were not rerun for this tooling/evidence-only batch.
Rejected candidates did not receive complete malformed-input qualification.

[Native borrowed-view qualification](../native-views-study.md) measures the public
native API with identical runtime code in both lanes. Borrowing cuts 1 MiB
decode-plus-byte-scan time 21–25% and sixteen-text-pair time 45–46%. Pure decode
savings are substantially larger and do not represent end-to-end work. All four
process runs passed content, re-encoding, pointer and frame-identity checks.
Blob-storage already borrows download descriptor text; its owned chunk transport
is provider-specific. No sibling edits or protocol cut were made. Raw runs are
in `/tmp/ic-wire-native-views-qualified/`.

## Retained decisions

Production remains bounded Postcard. Native owned content uses slice equality;
Wasm retains its word comparison. Both skip exact same-input-region comparisons
for borrowed values after length/allowance checks. The released borrowed-blob
improvement and limits of its measurements remain in the
[protocol/layout study](../protocol-study.md). Consumers still pay to read/hash
returned data; those operations are not included in pure decode timings.

The hypothetical fixed-key, compact-tag and mandatory-field layouts remain
experiments, not adopted schemas. Generic single-pass canonical decoding remains
unimplemented; normalized maps and arbitrary Serde semantics require more than
varint checks. Earlier generator scaling evidence is in
[performance](../performance.md#typescript-generator-comparisons).

AGENTS.md requires measured cost comparisons and inline protocol-change notes.
Exact retries, canonical hashing and typed errors remain required. Metadata and
scalar validators are trusted declarations. Other rejected runtime experiments
remain in [evaluation](../evaluation.md).
Shared Tooling is 0.25.9; IC Build's release editor is 0.63.0 with `cli`.
Ordinary agent work does not stage, commit or release.
