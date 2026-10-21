# Current handoff

## Current batch

Released 0.5.9 (`007b3f4`) is the integration baseline. PR #16 adopts Shared Tooling 0.25.9
release retry protection at reviewed revision
`85aded191d85ac4a8c0e5deab207f6be18895862`. A committed but undelivered release
preparation refuses a second increment and preserves its saved plan and evidence.
Package versions and dependency selections remain unchanged. The PR is refreshed
against current main; merging and release delivery remain maintainer-owned.

## Qualification

The refreshed PR passes snapshot integrity (104 files, exact 0.25.9 revision),
documentation links, disposable consumer Make/release routing and the disposable
release integration fixture. These checks execute release effects only inside
fixture repositories. Existing prepared tool binaries were reused read-only.
No full local CI, actual release, package publication or consumer suite ran.
Hosted checks must qualify the refreshed PR commit independently of earlier runs.

## Retained decisions

0.5.7 adopted IC Tooling 0.63.0, with the release editor selecting `cli` separately
from the diagnostic library. It also bounds registry enumeration to once per
operation and validates optional collection lengths. TypeScript qualification
runs in CI/release gates; see [development](../development.md) for local Node/npm
preparation before offline release validation.

Production remains bounded Postcard; JSON is optional generation/tooling metadata.
Finite scratch reuse shipped in 0.5.5. Fresh encoding remains separate because
sharing its mutable-buffer path regressed small-message instruction cost.
The [scratch-buffer study](../evaluation.md#finite-scratch-buffer-follow-up)
retains byte parity, capacity/retry tradeoffs and measured evidence.
Caller-owned scratch capacity may outlive large messages; retain immutable retry
bytes independently. Rejected experiments remain in [evaluation](../evaluation.md).

Shared Tooling is 0.25.9 in this PR. IC Build's release editor remains 0.63.0 with `cli`.
Shared snapshots, dependency selection and Rust runtime encoding are unchanged.
Staging, commits, versions, releases and publication remain maintainer-owned.
