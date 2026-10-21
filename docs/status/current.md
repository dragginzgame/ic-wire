# Current handoff

## Current batch

Released 0.5.8 (`1894d0a`) is the baseline. The bounded-growth TypeScript writer
experiment is complete and rejected. Production source is unchanged; the pending
0.5.9 changelog records evaluation evidence only. Package versions, staging and
Git refs remain untouched.

The prototype allocated at most 64 bytes initially, grew geometrically within
method limits and kept the final independent copy. Small frames under a 2 MiB
allowance benefited substantially, but tight 4 KiB blobs slowed 48–59% and tight
1,024-float arrays slowed 20–67% across two matched native Node runs. The latter
also requested roughly twice as many buffer bytes. Do not adopt this version.
See the [experiment](../evaluation.md#typescript-bounded-growth-experiment) for
inputs, environment, methodology, source hashes, limitations and retained data.

## Qualification

Both variants compiled with TypeScript 7.0.2 strict checks. Twenty timed cases
matched exact frames and decoded values. Another 450 baseline/candidate boundary
comparisons passed across both directions, Unicode, growth thresholds, floats,
and insufficient/exact frame allowances. These checks exercised only the isolated
prototype; the maintained runtime was not edited.

Raw runs, source variants, compiled JavaScript and the boundary probe are under
`target/typescript-buffer-growth/`, also retained at `/tmp/ic-wire-ts-growth/`.
The compact native result table is maintained under
`docs/evidence/typescript-buffer-growth/native.csv`.
No browser, IC instruction, deployment-size or peak-memory claim follows from
this native Node experiment. Full CI and consumer suites were not run.

## Retained decisions

0.5.8 enforces TypeScript numeric metadata bounds and removes per-float scratch
buffers/views. Its released qualification and implementation are in `1894d0a`.
Production remains bounded Postcard; JSON is optional generation/tooling metadata.
TypeScript qualification runs in CI/release gates; see
[development](../development.md) for local Node/npm preparation.

Finite scratch reuse shipped in 0.5.5. Fresh Rust encoding stays separate because
sharing the mutable-buffer path regressed small-message instruction cost.
The [scratch-buffer study](../evaluation.md#finite-scratch-buffer-follow-up)
retains byte parity, capacity/retry tradeoffs and measured evidence.
Caller-owned scratch capacity may outlive large messages; retain immutable retry
bytes independently. Rejected experiments remain in [evaluation](../evaluation.md).

Shared Tooling remains 0.25.8. IC Build's release editor remains 0.63.0 with `cli`.
Shared snapshots, dependency selection and Rust runtime encoding are unchanged.
Staging, commits, versions, releases and publication remain maintainer-owned.
