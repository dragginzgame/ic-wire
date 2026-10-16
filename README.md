# IC Wire

An unpublished investigation of a small, bounded binary protocol for Rust
applications on the Internet Computer. The goal is Candid-like developer ease
with substantially less Wasm code and serialization work for communication whose
producers and consumers we control.

**Status: no_std library scaffold with an evaluation-only codec comparison.**
The library has no runtime dependencies or public API. The maintainer selected
Postcard for controlled inter-canister calls and owned user endpoints. The wire
contract and production integration remain unfinished.
The package has `publish = false`. Do not reserve the crates.io name or publish
until the maintainer explicitly approves the direction and publication.

Start with [the handoff](docs/status/current.md), [design](docs/design.md),
[proposed developer experience](docs/developer-experience.md) and
[evaluation plan](docs/evaluation.md). The [initial evidence](docs/evidence.md)
explains why this investigation is worthwhile without claiming measured savings.
The [shared-codec assessment](docs/shared-codec-evaluation.md) evaluates optional
reuse with IcyDB and identifies the database contracts that remain separate.
The [payload screen](docs/codec-screen.md) records reproducible sizes and the
remaining obstacles to selecting a shared encoding.
The [matched canister experiment](docs/canister-comparison.md) adds measured
Wasm and IC instruction evidence, including retained Candid management traffic.
The [browser trial](docs/browser-interop.md) demonstrates a typed TypeScript client
using generated Postcard bindings in real Chromium against local PocketIC.

The intended consumers are Canic, IC Relay, IcyDB and native Rust clients.
IC Wire owns encoding rules, bounds and typed codec errors. Each consumer owns
its contracts and application semantics. Relay owns delivery and retries;
Canic owns endpoint integration. Host-side tooling stays outside the codec.

Candid remains necessary at external interfaces that specify it, including IC
management, ledger and selected registry calls. Candid-free application endpoints
do not imply Candid-free Wasm when other linked code still uses the codec.

## Development

The virtual root workspace owns the catalog, package version and lockfile.
See [local setup and commands](docs/development.md) and
[contributor instructions](AGENTS.md). With the documented prerequisites prepared:

```sh
make install-tools
make tools-check
make install-hooks
make check
make fmt-check
make lint
make check-wasm
make check-msrv
```

`make` lists available commands; `make ci` is the complete scaffold gate.
The reviewed Shared Tooling snapshot owns setup, formatting and hooks; checks
never install tools implicitly. CI is configured for Linux and both macOS
architectures, including the Rust 1.91 floor and Wasm target. These checks qualify
the scaffold only and establish no performance or protocol claims. Standard
release targets are present but disabled while the investigation is unpublished.

## License

[Dragginz Source Available License — Verifiable ICP Burn, Version 1.0](LICENSE),
using the same license text as IC Relay with this project's copyright notice.
This is source-available, not an OSI-approved open-source license.
