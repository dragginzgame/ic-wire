# Historical release receipts

This directory preserves the small registry/source identity records, release
plans and validation/preparation receipts moved from the former per-release
documentation directories. All 131 moved records retain their original bytes.
They are provenance observations for the named release, not input to the release
runner and not a second version catalog; manifests and group ledgers own versions.

The latest delivered pair remains Core/derive **0.17.2**, Endpoint/macros **0.7.1**:
[compact delivery record](published-core0172-endpoint071/delivery.json).
Its recorded Git releases are `46d7d08 / core-v0.17.2` and
`da5ed1e / endpoint-v0.7.1`. Checksums can be compared with registry downloads
and tagged source. Original gate and installed-consumer passes are historical
observations; bulk logs, copied consumers, guests and source archives are no
longer retained in this tree and cannot be verified from a summary/hash alone.

These original publication identities predate the maintainer-authorized fresh
Git history. Old commit/tag identities are historical records and may no longer
resolve in this repository. Crates.io archives and versions are unchanged;
do not re-upload them or retag the new root as an old release. New releases use
the documented workflow from the current main. The
[research retention policy](../../docs/evidence/README.md) governs experiments
separately.
