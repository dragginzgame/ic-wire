# Current handoff

## Released baseline and pending 0.7.1

0.7.0 is released at `aa7cc32acbd54a58d77720bedfb1ae530926f57e`; package versions
remain 0.7.0. This batch started clean. The open 0.7.1 changelog covers opt-in
host bulk advice and Packed<i8>. Staging, versions, commits, publication and
pushes remain maintainer-owned. No sibling was changed.

## Implemented work

`typescript::lint_bulk_fields` scans named resolved schema roots, including
nested aliases/containers and shared references. It recommends byte buffers or
Packed for ordinary primitive lists. Exact diagnostic paths plus nonempty reasons
acknowledge deliberate compact fields; stale acknowledgements remain visible.
The `bulk-lint` example prints advice and optionally fails CI with
`--deny-warnings`. Consumers must include inline method request/response schemas
as roots. Types without Wire metadata are outside coverage. No automatic rewrite
or canister runtime policy is introduced.

Packed<i8> uses existing packed machinery and Int8Array metadata. For [-128,127],
bytes remain `02 80 7f`; traversal budgets, JSON projection and generated clients
change on adoption. Preserve consumer constraints and regenerate clients. All
existing fields, frame identities, hashes and retry rules remain unchanged.

## Qualification and tradeoffs

The [matched bulk study](../evidence/bulk-advice-071/README.md) retains 109 cases,
2,180 IC row pairs, three equal samples per row, matching logical input identities
and unchanged Candid controls. It compares ordinary versus explicitly packed
fields with the same expanded fixture and Cargo.lock, not a transparent upgrade.
At opt-3, 16,384 i8 values encode in 17,298 versus 820,017 ordinary instructions;
decode costs 17,435 versus 667,758. Frame bytes are identical. Every selected
packed encode/decode case beats Candid in both profiles. This is corpus evidence,
not a universal guarantee. Single-i8 encode/decode regress 6.13%/8.47% versus
ordinary fields; small integer payloads may grow when fixed-width packing is used.

Packed cohort total Wasm falls 13,779 bytes at opt-3 (10.10%). The unchanged
100-record consumer remains byte-identical at 398,712 bytes. Optional host linting
adds about 119 microseconds to generation for 100 schema roots in the measured
probe. No native codec timing or installed-canister size claim is made.

Native/lib tests, strict Clippy, doctests, CLI exit checks, Rust 1.91 and native/
no_std/Wasm consumers pass. TypeScript passes 159 vectors, 33,560 mutations,
20 projection vectors and 54 simulated-endian cases. ASan fuzzing passes 100,000
inputs with leak detection disabled for sandbox constraints. Real big-endian
execution remains unqualified. Formatting, diff and local document links pass.

## Retained direction

One production Serde/Postcard codec remains, with one-pass strict primitive
decoding and consumer-owned normalization. Retain original frames for exact
retries and raw-byte signatures/hashes; successful decoding does not promise
identical re-encoding. No compatibility lane exists. The
[0.7 one-pass study](../evidence/one-pass-070/README.md) and
[0.6.13 full matrix](../full-comparison-0613.md) remain historical evidence.
Ordinary-vector controls and their large gaps remain visible. See the
[bulk field guide](../packed-fields.md) and [runtime contract](../runtime.md).

Shared Tooling is pinned at `4c0f198e`, recorded version 0.26.10. IC Build/Host/
Testkit are 0.67.0 and Platform 0.6.14. Siblings remain read-only. Do not infer
policy from mutable siblings or patch snapshot-owned files.
