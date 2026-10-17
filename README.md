# IC Wire

A small, bounded binary protocol for Rust
applications on the Internet Computer. The goal is Candid-like developer ease
with substantially less Wasm code and serialization work for communication whose
producers and consumers we control.

**Status: 0.3.0 published on crates.io.**
The library supplies explicit-layout derives, bounded containers, typed method
bindings, finite float wrappers and canonical request/reply envelopes. Its normal dependency graph has
no Candid, CDK or host tooling. Start with the [runtime guide](docs/runtime.md)
and [installed adoption proof](docs/runtime-adoption.md). Canic macro integration
remains consumer work; a [local macro fixture](docs/macro-adoption.md) tests the
expansion pattern here. An optional host generator now emits bounded TypeScript
codecs from those same wire declarations. Installed Chromium tests qualify
controlled queries, protected updates, replay and peer calls. Production browser
login/delegation and Canic integration still need consumer qualification.
The library and derive packages are configured for crates.io; the evaluation
package remains unpublished. See [publication preparation](docs/development.md#first-publication)
for the separate Git release and registry steps. Both
[`ic-wire`](https://crates.io/crates/ic-wire/0.3.0) and
[`ic-wire-derive`](https://crates.io/crates/ic-wire-derive/0.3.0) are published.

Start with [the handoff](docs/status/current.md), [design](docs/design.md),
[proposed developer experience](docs/developer-experience.md) and
[evaluation plan](docs/evaluation.md). The [initial evidence](docs/evidence.md)
explains why this investigation is worthwhile without claiming measured savings.
The [shared-codec assessment](docs/shared-codec-evaluation.md) evaluates optional
reuse with IcyDB and identifies the database contracts that remain separate.
The [payload screen](docs/codec-screen.md) records reproducible sizes and the
remaining obstacles to selecting a shared encoding.
The historical [matched canister experiment](docs/canister-comparison.md) adds measured
Wasm and IC instruction evidence, including retained Candid management traffic.
The [production runtime comparison](docs/runtime-comparison.md) measures the
current envelopes, bounds and finite floats against strict Candid.
The historical [nested authority extension](docs/nested-authority.md) adds bounded delegate
and grant collections with new installed measurements.
The [browser trial](docs/browser-interop.md) demonstrates a typed TypeScript client
using generated Postcard bindings in real Chromium against local PocketIC.
The [peer extension](docs/peer-interop.md) exercises those same bytes through a
forwarding canister and verifies the actual recipient caller.
The historical [protected-command trial](docs/command-replay.md) qualified direct-caller
authorization, atomic local effects and stable exact-byte replay across upgrades.
Durable receipt-policy qualification now lives in [Relay’s receipt probe PR](https://github.com/dragginzgame/ic-relay/pull/2);
the current browser fixture retains only volatile replay state.
The [consumer adoption assessment](docs/consumer-adoption.md) maps Canic and sibling
integration work and the remaining prerequisites.

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
release targets validate, version, tag and push to origin/main; they do not
publish packages. See the [release workflow](docs/development.md#snapshot-maintenance-and-release-boundary).

## License

[Dragginz Source Available License — Verifiable ICP Burn, Version 1.0](LICENSE),
using the same license text as IC Relay with this project's copyright notice.
This is source-available, not an OSI-approved open-source license.
