# Release receipts

This directory preserves the small registry/source identity records, release
plans and validation/preparation receipts moved from the former per-release
documentation directories. All 131 moved records retain their original bytes.
They are provenance observations for the named release, not input to the release
runner and not a second version catalog; manifests and group ledgers own versions.

The latest recorded delivery is Endpoint/macros **0.8.1**:
[frozen delivery record](published-endpoint081/delivery.json). Tag
`endpoint-v0.8.1` selects `5bb495a`; both registry checksums match uploaded and
final all-feature archives. All 20 Linux gate stages passed. Core stays **0.17.13**.
The concurrent Testkit 0.93.1 update is deferred; the frozen release used 0.93.0.

Core/derive **0.17.13** retains its
[frozen delivery record](published-core01713/delivery.json). Tag `core-v0.17.13`
selects `c719ece`; both registry checksums match the uploaded and final
all-feature archives. All 20 Linux gate stages passed.

The prior Core **0.17.12** and Endpoint/macros **0.8.0** retain their
[frozen delivery record](published-core01712-endpoint080/delivery.json).
Tags `core-v0.17.12` and `endpoint-v0.8.0` resolve to `6082752` and
`a967a9c`. All four checksums match independent registry downloads; the record
includes both full gates, final archives and installed Endpoint qualification.

The [older 0.17.2/0.7.1 record](published-core0172-endpoint071/delivery.json)
retains its original identities. Earlier gate and installed-consumer passes are
historical observations; retired bulk logs, copied consumers, guests and source
archives cannot be verified from a summary/hash alone.

Some original publication identities predate the maintainer-authorized fresh
Git history. Old commit/tag identities are historical records and may no longer
resolve in this repository. Crates.io archives and versions are unchanged;
do not re-upload them or retag the new root as an old release. New releases use
the documented workflow from the current main. The
[research retention policy](../../docs/evidence/README.md) governs experiments
separately.
