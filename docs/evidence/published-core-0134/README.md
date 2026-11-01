# Verified Core 0.13.4 delivery

Delivered 2026-11-01 following the maintainer's deployment request. Source commit
`25a4cb9a27c53c79498039eb2eedd253af46ca7f` passed the standard complete local CI
gate in **607 seconds**. The runner then prepared only Core's metadata, committed
`f83755b30f76704105164e163b9334310e106e44`, created annotated tag `core-v0.13.4`
(object `dcb4dc156989f8f40e472f3dc58d809d92f27aac`) and atomically pushed
origin/main and that tag. [Remote readback](remote-delivery.txt), exact receipts,
release plan and [full logs](checks/) retain the delivery bindings. No gate was
skipped. Endpoint remains **0.3.0**.

Published and independently verified:

- [ic-wire-derive 0.13.4](https://crates.io/crates/ic-wire-derive/0.13.4).
- [ic-wire 0.13.4](https://crates.io/crates/ic-wire/0.13.4).

The publication adapter first completed a two-package dry run and archive
verification. Before upload, both archives' clean VCS identity matched the release
commit and package directories. Additional provenance checking used Shared's
canonical helper from the isolated **exact adopted 1211262** checkout; it fetched
that commit into a fresh object store. This was a one-off supplementary check,
not a new installed consumer gate or a dependency on a moving sibling. Its
[request/readback](provenance/) remains retained; the full object store stays
under `target/deploy-0134/provenance/source.git`.

One separately authorized publication attempt uploaded both packages. Independent
sparse-index and static archive downloads verify unyanked entries, registry
SHA-256, bytes identical to both the prepared and uploaded archives, and exactly
one clean `.cargo_vcs_info.json` with the release commit and correct directory.
Core's actual source contains the new admission checks. Published packages exclude
root tests/drivers/evidence; source-local unit tests remain within the existing
src allowlist. Per-package `*-registry.json`, `*-vcs.json`, `*-verified.json` and
[prepared identities](prepared-archive-identities.json) retain these checks.

The [registry-only consumer](registry-consumer/) resolves one Core/derive 0.13.4,
Endpoint 0.3.0, Platform/runtime guard 0.16.0 and canonical Types 0.15.1 identity,
with no Cargo patches or workspace paths. All **six Rust 1.91 tests** pass:
ByteArray framing/metadata, endpoint Contract projection, Principal/Account
schemas and malformed inputs, including absent versus explicit-zero subaccounts.
Rust 1.91 Wasm compilation passes; its normal/build graph excludes Candid,
PocketIC, serde_json, Testkit and the external executor.

Reproduce by copying the consumer manifest/lock and source snapshots to a fresh
workspace, preparing their locked registry cache, then running:

```sh
cargo +1.91.0 test --locked --offline --lib
cargo +1.91.0 check --locked --offline --target wasm32-unknown-unknown
```

The retained [registry verifier](verify-registry.py) consumes downloaded
`NAME-index-after.jsonl` and `upload-archives/NAME-0.13.4.crate` beside itself and
compares the registry checksum/provenance with Cargo's local upload archives.
All raw artifacts, failed/successful histories and publication attempt evidence
remain in `target/deploy-0134`, `target/publication` and `.git/release-state`.
Nothing was cleaned. [SHA256SUMS](SHA256SUMS) binds this evidence.

The delivered batch selects Shared 0.27.24, Build/Host 0.73.5 and Testkit 0.76.0.
It retains one Serde/Postcard codec and preserves protocol bytes, budgets,
acceptance and exact retry/hash inputs. The small admission cleanup's measured
tradeoffs are in the [matched study](../cost-experiments-0134/README.md); historical
performance reports retain their frozen inputs. This delivery does not claim a
fresh full Candid performance matrix, hosted/macOS qualification or new Wasm
savings. Further writer experiments are separate, unreleased work. No sibling
checkout was modified or new Endpoint release performed.
