# Verified Core 0.13.5 delivery

Delivered on 2026-11-01 after the maintainer requested push/deploy. Source
`422b3f554b2fec9d2b4794cb6c7bea456498d404` passed the complete standard local
release CI gate in **567 seconds**. The standard runner prepared only Core's
metadata, created release commit `a8195d42683f90810a9afd615e4f27e3e5a3c744`
and annotated `core-v0.13.5` tag (object
`6e6aff91a74f8436a1541eebac112b4aa9a13247`), then atomically pushed origin/main
and the exact tag. [Remote observation](remote-delivery.txt), [plan](core-0.13.5.plan),
source-bound receipts and [complete check logs](checks.tar.gz) retain the bindings.
Endpoint remains **0.3.0**. No release gate was skipped or artifact cleaned.

Published and independently verified:

- [ic-wire-derive 0.13.5](https://crates.io/crates/ic-wire-derive/0.13.5).
- [ic-wire 0.13.5](https://crates.io/crates/ic-wire/0.13.5).

Both publication dry runs and archive verification passed. Before one authorized
two-package upload, exact registry versions were confirmed absent. Prepared
archives have clean VCS provenance for the release commit and package directories;
the canonical supplementary helper at adopted Shared commit
`1211262f32ea193cc4edcd60d2750baf1f1d6425` independently fetched the source commit.
[Provenance receipts](provenance/source-verified) retain that check. This does not
add an installed gate or a dependency on a moving sibling checkout.

Independent sparse-index/static archive downloads match registry checksums,
prepared bytes and Cargo's actual upload archives. Both entries are unyanked.
Exactly one clean `.cargo_vcs_info.json` identifies each release source/path.
The downloaded Core archive's output-writer source equals the released file and
contains the selected Wasm-only inlining hint. Published dependencies exclude
Candid, host tooling and the proposed independent Runtime. Per-package registry,
VCS and verified records plus [prepared identities](prepared-archive-identities.json)
retain the observations.

The fresh [registry-only consumer](registry-consumer/Cargo.toml.snapshot) passes
all **six Rust 1.91 tests** and Wasm compilation: ByteArray framing/metadata,
Endpoint Contract projection, Principal/Account exact representations and malformed
input, including absent versus explicit-zero subaccounts. Its [identities](consumer-identities.json)
resolve one Core/derive 0.13.5, Endpoint 0.3.0, Platform/macros/guard 0.16.0 and
canonical Types 0.15.1. The [normal/build Wasm graph](consumer-wasm-graph.txt)
contains no Candid, simulator, serde_json, host tooling or external executor.
There are no workspace-path dependencies or Cargo patches in that consumer.

Reproduce in a fresh directory by restoring the consumer's manifest, lock and
source snapshots, preparing its locked registry cache, then running:

```sh
cargo +1.91.0 test --locked --offline --lib
cargo +1.91.0 check --locked --offline --target wasm32-unknown-unknown
```

[verify-registry.py](verify-registry.py) accepts package, version and release
commit and reads downloaded `NAME-index-after.jsonl`, `upload-archives/` and
`prepared-archives/` beside it. Run from the release checkout's Wire root with Cargo's retained
`target/package` archives to repeat the checksum/source/body checks.
[verify-consumer.mjs](verify-consumer.mjs) audits retained consumer metadata and
graphs under `target/deploy-0135`. Raw archives, metadata, publication selections,
source object stores and all attempts remain there, in `target/publication` and
`.git/release-state`. [SHA256SUMS](SHA256SUMS) binds the checked-in evidence.

This release keeps one Serde/Postcard codec, preserving bytes, budgets, acceptance
and exact retry/hash inputs. The [writer study](../writer-inlining-0135/README.md)
records the 2,509 total / 2,505 code-byte opt-3 saving in the 100-type fixture,
unchanged opt-z size, maximum 1.83% IC instruction increase and native variability.
Those are frozen matched measurements, not a new post-publication full Candid
matrix or hosted/macOS acceptance claim.

[PR #41's Runtime adoption](../runtime-owner-017/README.md) is retained source
research and is **not integrated or deployed**. The [final registry observation](runtime-publication-gate.json)
still finds Runtime absent and macros/guard at 0.16.0. Its eventual release must
qualify actual registry suppliers and canonical Types, then coordinate consumers.
No sibling checkout was modified and no Endpoint release was performed.
