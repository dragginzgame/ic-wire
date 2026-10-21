# Current handoff

## Current batch

Local main and remote main match at `3d83549`. The local annotated `v0.5.10`
tag points to `86aa88c`, which includes the small-frame optimization from PR #15
but predates the tooling adoption from PR #16. The subsequent merge brought that
tooling change onto main. The remote has no `v0.5.10` tag as of this inspection;
Git delivery of the release tag remains unconfirmed. Package publication was not
checked. Do not move the existing tag or infer publication from the branch push.

Pending 0.5.11 notes now contain the tooling adoption. The dated 0.5.10 entry
matches its local tag's changelog. Package versions remain 0.5.10. This correction
and handoff are uncommitted; staging, commits, tag delivery and releases remain
maintainer-owned. Main CI for `3d83549` is pending at this inspection.

Shared Tooling 0.25.9 is present at reviewed revision
`85aded191d85ac4a8c0e5deab207f6be18895862`. It refuses a second release increment
past an undelivered committed preparation and preserves the saved evidence.
Reconcile the existing release delivery before attempting another increment.

## Qualification

The refreshed PR passes snapshot integrity (104 files, exact 0.25.9 revision),
documentation links, disposable consumer Make/release routing and the disposable
release integration fixture. These checks execute release effects only inside
fixture repositories. Existing prepared tool binaries were reused read-only.
No full local CI, actual release, package publication or consumer suite ran.
All three hosted checks passed on PR #16 head `257a161` before its merge at
`73ef717`; that result is separate from the current main run.

The merged small-frame optimization reserves fresh ceilings up to 128 bytes;
larger ceilings and scratch reuse retain their existing policy. Its 37 focused
tests and three-platform PR CI passed before merge. The measured 18–26% IC
instruction reduction applies to affected small fresh frames; exact frames are
unchanged. See [evaluation](../evaluation.md#small-finite-schema-reservation)
for native/IC results, size and retained-capacity tradeoffs. External raw probes
remain under `/tmp/ic-wire-pr15-probe/`. The TypeScript growth experiment was
rejected separately and remains documented in the same evaluation guide.

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
Only the reviewed release-tooling snapshot changes in PR #16; dependency
selection and Rust runtime encoding remain unchanged.
Staging, commits, versions, releases and publication remain maintainer-owned.
