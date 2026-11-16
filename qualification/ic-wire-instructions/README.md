# Installed IC instruction runner

This private native package belongs to the single qualification workspace and
consumes published Testkit 0.107.3 for original PocketIC client/type identities.
The Rust driver selects Testkit’s managed-server capability, verifies a prepared
PocketIC 16.1.0 executable, and supplies explicit startup, lifetime and output
budgets. It retains process/cleanup evidence and bounded server logs, explicitly
closes its instance, independently observes deletion, then stops the server.
It creates/funds/installs two disposable canisters, queries each with distinct
callers, then stops/deletes them and closes the owned instance/server.

The driver and guest share qualification/Cargo.toml and Cargo.lock. PocketIC 16.1.0 requires thiserror 2.0.18 in this graph; Platform Types
0.21.2 uses a compatible requirement. Generic-array 0.14.7 remains selected. No native simulator,
Candid or SDK is added to product Wire. The guest uses Platform Runtime 0.23.14;
its dependency-free `system-raw` feature supplies the IC operations. The private
guest owns one fixed query export with a narrowly expected Rust 2024 unsafe
attribute; product Wire continues to forbid unsafe code.

From the repository root, explicitly select a prepared server:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic make measure-consumer-ic
```

The command requires curl and never downloads a server. It uses offline locked Cargo dependencies,
creates fresh per-profile guest targets and retains instructions.json,
compiler identity and the exact measured artifacts under qualification/target.
The report contains current measurements without a historical comparison gate.
Counter intervals include result destruction and loop overhead; setup and raw
reply wrapping are outside them. Loop control is reported, not subtracted.

The [measurement contract](../../docs/design/performance.md) records operations
and evidence limits. Focused checks for the native driver, without starting a server:

```sh
cargo metadata --manifest-path qualification/Cargo.toml --format-version 1 --locked --offline > /dev/null
cargo clippy --manifest-path qualification/Cargo.toml -p ic-wire-instructions --locked --offline -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path qualification/Cargo.toml -p ic-wire-instructions --no-deps --locked --offline
```

All common Cargo, Rust, Rustdoc and Clippy lints are inherited by the driver.
Ordinary library requirements allow compatible updates; the lockfile fixes the
reviewed graph and the Toko Tooling source selection remains exact.
Root dependency commands and formatting include this package through the
qualification workspace. The measurement target admits the qualification Toko Tooling selection.
The inherited package source/license markers are development-only; all packages
remain unpublished. Ordinary consumer tests need neither this runner nor a server.
