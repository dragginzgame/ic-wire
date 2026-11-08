# toko-tooling

A source distribution of the Dragginzgame shared development tooling. The crate
contains the scripts, Make interfaces, CI actions, tool catalogs, engineering
rules, licenses and guides used by consumers. Its Rust library is dependency-free
and `no_std`; it has no build script and never installs anything during a build.

Cargo downloads the tooling code from crates.io. Cargo.lock selects the exact
release and archive checksum; installation remains an explicit command.

Declare the registry dependency in a private tooling package:

```toml
[dev-dependencies]
toko-tooling = "0.33"
```

Run `make shared-tooling-update` to update this package and install its selected
code, or `make shared-tooling-install` after a separate Cargo update. The installer
checks the downloaded archive against Cargo.lock before executing its exporter.
It preserves consumer edits and recorded entrypoints, retires unchanged obsolete
files, and prepares/checks the declared tools. `make shared-tooling-check` verifies
the installed snapshot and its registry selection offline.

See the [snapshot adoption guide](https://github.com/dragginzgame/toko-tooling/blob/main/docs/consuming-snapshots.md#cargo-selection-and-installation)
for initial setup and migration. Releases before 0.33 contain only a marker and
cannot supply the tooling payload.
