# Verified Core 0.13.3 and Endpoint 0.3.0 delivery

Delivered on 2026-11-01 following the maintainer's deployment request. Git
releases and crates.io publication were executed separately, Core first.

| Group | Release commit | Annotated tag | Complete local CI |
| --- | --- | --- | ---: |
| Core | `33306395a833fd29f8c8a91abf9b8d38123dd0f9` | `core-v0.13.3` | 579 s |
| Endpoint | `3ddaeb834e0f8e3290f3146a119435a185b9bda9` | `endpoint-v0.3.0` | 617 s |

Core's pre-version source is `2d6558f` and Endpoint's is the Core release commit.
The standard adapters validated the complete workspace, then prepared only each
selected group's metadata, checked receipts, committed/tagged and atomically
pushed origin/main. [Remote observation](remote-delivery.txt), release plans,
validated/prepared receipts and [logs](checks/) retain exact source/delivery
bindings. No release gate was skipped or narrowed. Native macOS/hosted results
are separate from these local Linux release gates.

Published and independently verified on crates.io:

- [ic-wire-derive 0.13.3](https://crates.io/crates/ic-wire-derive/0.13.3).
- [ic-wire 0.13.3](https://crates.io/crates/ic-wire/0.13.3).
- [ic-wire-endpoint 0.3.0](https://crates.io/crates/ic-wire-endpoint/0.3.0).

Dry runs verified the prepared packages before upload. Each registry record is
unyanked; downloaded archive bytes match the local upload archive and registry
SHA-256. Exactly one clean `.cargo_vcs_info.json` per package points to the
correct release commit and package directory. Published packages exclude root
test/benchmark/evidence directories; Core's source-local unit test module remains
within its existing src allowlist. The per-crate `*-registry.json`, `*-vcs.json`
and `*-verified.json` files retain readback and provenance. This does not infer
success only from Cargo's exit status, an index entry or a tag.

A standalone registry-only consumer (no patches or workspace paths) resolves
one identity each for Core/derive 0.13.3, Endpoint 0.3.0, Platform/runtime guard
0.16.0 and canonical Types 0.15.1. All **6 Rust 1.91 tests** pass, covering exact
ByteArray framing/metadata, endpoint Contract projection, Principal/Account
schemas, malformed refusals and absent versus explicit-zero subaccounts. Rust
1.91 Wasm compilation passes. Its normal/build Wasm graph excludes Candid,
PocketIC, serde_json and an external executor. [Consumer inputs/results](registry-consumer/)
retain the exact manifest, lock, source and identities.

To reproduce the registry consumer after preparing Rust 1.91 and its Wasm target,
copy Cargo.toml.snapshot/Cargo.lock.snapshot and lib.rs.snapshot from that
directory into a fresh Cargo.toml/Cargo.lock/src/lib.rs tree. Then run:

```sh
cargo +1.91.0 test --locked --offline --lib
cargo +1.91.0 check --locked --offline --target wasm32-unknown-unknown
```

Prepare the recorded registry cache first if unavailable; these commands do not
publish. Full local release logs and publication logs remain in target/deploy-0133,
release custody in .git/release-state, and per-attempt publication evidence in
target/publication. Successful delivery does not authorize cleanup.

The delivered snapshot remains Shared 0.27.23 at `619d194`. The newly requested
Shared 0.27.24 adoption and optimization investigations follow this delivery;
they are not included in these published artifacts. Earlier frozen benchmark
reports retain their own supplier/context identities. No fresh performance matrix,
IC cycle claim, Wasm optimization claim or gzip size comparison is implied here.

Core retains one Serde/Postcard codec. Existing bytes/admission stay unchanged;
explicit ByteArray adoption changes length prefix/bulk admission as described in
the Core release notes. Endpoint 0.3 requires aligned Platform 0.16 runtime
suppliers; canonical Types and Core schema identity remain stable. No sibling
checkout was modified. SHA256SUMS binds this delivery evidence.
