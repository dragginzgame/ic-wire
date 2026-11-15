# Installed IC instruction runner

This private native workspace consumes published Testkit 0.106.0. Testkit owns
prepared-server admission, startup and instance/server lifetime; the runner owns
only the selected Wire consumer experiment and its independent frame oracle.
It creates/funds/installs four disposable canisters, queries each with distinct
callers, then stops/deletes them and closes the owned instance/server.

The separate workspace is intentional: PocketIC 16.1.0 pins thiserror 2.0.18,
whereas the codec comparison retains 2.0.21. The guest is built from the parent
qualification workspace, not this host lock. No native simulator, Candid or SDK
is added to product Wire. The optional guest SDK is Platform Runtime 0.22.5;
its safe macros provide the IC export while this repository forbids unsafe code.

From the repository root, explicitly select a prepared server:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic make measure-consumer-ic
```

The command never downloads a server. It uses offline locked Cargo dependencies,
creates fresh per-supplier/profile guest targets and retains instructions.json,
compiler identity and the exact measured artifacts under qualification/target.
A violated numerical criterion returns failure after saving the report. That is a
measurement finding, not a failed artifact installation or missing results.
Counter intervals include result destruction and loop overhead; setup and SDK
reply wrapping are outside them. Loop control is reported, not subtracted.

The [frozen contract and results](../../docs/design/consumer-instructions.md)
record thresholds, source hashes, remaining regressions and evidence limits.
Use `cargo clippy --manifest-path qualification/ic-instructions/Cargo.toml --locked --offline -- -D warnings`
for the native driver. Root dependency commands include this workspace by default;
formatting also includes it. The measurement target verifies its Toko Tooling pin.
The inherited package source/license markers are development-only; all packages
remain unpublished. Ordinary consumer tests need neither this runner nor a server.
