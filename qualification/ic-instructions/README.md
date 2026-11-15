# Installed IC instruction runner

This private native workspace has one package, `ic-wire-instructions/`, and
consumes published Testkit 0.106.0. Testkit owns
prepared-server admission, startup and instance/server lifetime; the runner owns
only the selected Wire consumer experiment and its independent frame oracle.
It creates/funds/installs two disposable canisters, queries each with distinct
callers, then stops/deletes them and closes the owned instance/server.

The separate workspace has a [scoped adoption exception](../../AGENTS.md#scoped-instruction-workspace-exception).
PocketIC 16.1.0 pins thiserror 2.0.18, whereas the codec workspace selects 2.0.21.
A 15 November 2026 resolver probe merged the real driver into a scratch parent
workspace: it resolved successfully, but downgraded the codec's thiserror and
thiserror-impl to 2.0.18. Isolation keeps that native constraint out of codec
dependency maintenance. Revisit the exception when the selected Testkit/PocketIC
stack relaxes it. The guest is built from the parent qualification workspace,
not this host lock. No native simulator, Candid or SDK
is added to product Wire. The optional guest SDK is Platform Runtime 0.22.5;
its safe macros provide the IC export while this repository forbids unsafe code.

From the repository root, explicitly select a prepared server:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic make measure-consumer-ic
```

The command never downloads a server. It uses offline locked Cargo dependencies,
creates fresh per-profile guest targets and retains instructions.json,
compiler identity and the exact measured artifacts under qualification/target.
The report contains current measurements without a historical comparison gate.
Counter intervals include result destruction and loop overhead; setup and SDK
reply wrapping are outside them. Loop control is reported, not subtracted.

The [measurement contract](../../docs/design/performance.md) records operations
and evidence limits. Focused checks for the native driver, without starting a server:

```sh
cargo metadata --manifest-path qualification/ic-instructions/Cargo.toml --format-version 1 --locked --offline > /dev/null
cargo clippy --manifest-path qualification/ic-instructions/Cargo.toml -p ic-wire-instructions --locked --offline -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path qualification/ic-instructions/Cargo.toml -p ic-wire-instructions --no-deps --locked --offline
```

All common Cargo, Rust, Rustdoc and Clippy lints are inherited by the driver.
Ordinary library requirements allow compatible updates; the lockfile fixes the
reviewed graph and the Toko Tooling source selection remains exact.
Root dependency commands include this workspace by default;
formatting also includes it. The measurement target verifies its Toko Tooling pin.
The inherited package source/license markers are development-only; all packages
remain unpublished. Ordinary consumer tests need neither this runner nor a server.
