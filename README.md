# IC Wire

A bounded Postcard binary protocol for controlled Rust endpoints on the Internet
Computer. IC Wire owns encoding, explicit layouts, resource admission and typed
codec errors. Consumers own transport, authentication, endpoint integration,
application semantics and retry policy.

The maintained runtime supplies explicit-layout derives, bounded containers,
finite float wrappers and canonical request/reply envelopes. The native Serde
API admits ordinary containers and recursive values under explicit resource
limits. Optional host-side generation emits matching TypeScript codecs without
importing a transport or SDK.

Start with the [runtime guide](docs/runtime.md), [design](docs/design.md),
[evaluation scope](docs/evaluation.md) and [current handoff](docs/status/current.md).
The runtime and derive crates were first published at 0.3.0; the private
`ic-wire-qualification` package remains unpublished.

## Development

The virtual root workspace owns package metadata, dependency selections and the
lockfile. See [local setup](docs/development.md) and [contributor instructions](AGENTS.md).
With the documented prerequisites prepared:

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

Use focused targets for changed behavior. `make ci` is the complete scaffold gate
and requires an explicit local request. Release commands deliver Git versions
and tags; registry publication is a separate maintainer action.

## License

Licensed under the [MIT License](LICENSE).
