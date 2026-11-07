# Release receipts

This directory preserves the small registry/source identity records, release
plans and validation/preparation receipts moved from the former per-release
documentation directories. All 131 moved records retain their original bytes.
They are provenance observations for the named release, not input to the release
runner and not a second version catalog; manifests and group ledgers own versions.

The latest recorded delivery is Core/derive **0.17.11**, with Endpoint/macros
unchanged at **0.7.3**: [frozen delivery record](published-core01711/delivery.json).
Core's tag `core-v0.17.11` resolves to `104558b`. Checksums can be compared with
registry downloads and tagged source. The receipt distinguishes current delivery
qualification from earlier benchmark graphs.

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
