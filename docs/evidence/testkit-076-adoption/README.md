# Published Testkit 0.76 adoption

Qualified 2026-11-01 on Linux. Adopt `ic-testkit` **0.76.0** with defaults
disabled in Wire's workspace dependency, used only by the private codec benchmark
and endpoint fixture. Both now share the existing **Platform 0.16.0** supplier.
The lock removes Platform 0.15.3 and upgrades only Testkit; all other package
versions, including Types 0.15.1 and Build/Host 0.73.4, remain unchanged.
No own release version, published runtime dependency, codec byte, resource budget,
acceptance rule or saved retry/hash input changes. No publication was performed.

## Supplier review

The [authoritative sparse-index entry](registry-release.json) selects the latest
stable, unyanked published release observed during adoption, with Rust 1.91 and
archive SHA-256
`5c9e84475c2c4a85466848d36694882cbeb3d1732819871acc3900d93862dde7`.
The downloaded Cargo archive matches that checksum. Its VCS metadata identifies
commit `2973946c6f7dc13552af78f887d5e2a8dbf37a61`, path `crates/ic-testkit`.
The upstream [release notes](https://github.com/dragginzgame/ic-tooling/blob/2973946c6f7dc13552af78f887d5e2a8dbf37a61/releases/integration/CHANGELOG.md)
describe Platform 0.16/Certificate identity alignment and arithmetic-only Metrics
0.24.4 with defaults disabled.

All **28 Testkit Rust source files are byte-identical** to 0.75.2. Per-file hashes
are retained in [supplier-source-hashes.json](supplier-source-hashes.json), with
archive provenance in [supplier-review.json](supplier-review.json). The dependency
change still requires checking real callers because Platform owns their native
PocketIC/DTO identities. No wrappers, Cargo patches or new dependencies are added.
The supplying sibling checkout was inspected read-only; its unpublished work was
neither adopted nor modified. The crates.io metadata API returned HTTP 403;
selection used the successful sparse-index query and Cargo's verified archive.

## Focused qualification

All commands use the recorded lockfile and offline prepared caches:

- Strict Clippy for both private packages' targets: existing codec instruction,
  admission, input-borrowing/caller-storage and endpoint installed/cost callers.
- Rust **1.91.0** compilation of the same targets and selected features.
- Core normal/build graph excludes Platform, Testkit, PocketIC and Candid.
- Endpoint fixture's actual Wasm normal/build graph excludes Testkit, PocketIC,
  Candid, serde_json and the external CDK executor.
- Native qualification graph selects Testkit 0.76.0 and Platform 0.16.0, with no
  remaining Platform 0.15 copy.
- The existing installed endpoint test passes against the explicitly prepared
  PocketIC server: exact replies, malformed/over-budget admission, async calls
  and reply/transaction rollback. Its Wasm is built by the owning script.

See [clippy.log](clippy.log), [msrv.log](msrv.log), [installed.log](installed.log)
and the three retained graph files. An initial all-target command omitted the
fixture's `schema` feature and failed on schema-only assertions. The corrected
command follows its supported setup; the initial failure logs remain retained.
No production source change was made to silence that qualification mistake.

The installed server is PocketIC **16.0.0**, SHA-256
`69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.
The development compiler is Rust **1.99.1**, LLVM **23.1.1**. These local checks
do not qualify other native hosts or constitute a full CI/performance matrix.
No native-time, IC-instruction, cycle or Wasm-footprint improvement is claimed:
this is qualification-dependency/type alignment, with unchanged production
codec/generator source and runtime graph. Earlier cost reports retain their
frozen 0.75.2 locks; do not pair their results with a new graph as a regression
comparison. Raw uncompressed Wasm/code-section metrics remain the size objective.

## Reproduction and evidence

The exact [qualification runner](qualify.sh) is recorded for execution from
`target/testkit-076-adoption/qualify.sh` in the repository. Copy it there, inspect
the prepared server path and adapt that local path on another host. Native
features are `wire,candid,ic,cost-smoke` on the benchmark package and `ic,schema`
on the endpoint fixture, with package-qualified Cargo arguments in the runner.
Wasm graph/build steps run separately without
host schema features. The runner starts only the explicitly prepared local server
through Testkit, with no implicit download. `scripts/ci/check-endpoint-ic.sh`
owns the installed test. Global Cargo configuration/build inputs must also remain
fixed; this evidence is not a hermetic build claim.

[Cargo.toml.snapshot](Cargo.toml.snapshot), [Cargo.lock.snapshot](Cargo.lock.snapshot)
and the toolchain snapshot bind the adopted inputs. The `.before` files retain
the earlier graph, [lock-review.json](lock-review.json) records the exact package
version delta, and [SHA256SUMS](SHA256SUMS) covers the evidence directory.
Own versions remain Core 0.13.3 / Endpoint 0.3.0; this pending batch is uncommitted.
