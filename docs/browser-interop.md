# Postcard browser trial

One typed update endpoint works end-to-end in Chromium against a local PocketIC
canister. Rust generates JavaScript bindings, TypeScript declarations, limits and
golden vectors from the evaluation message. The frontend imports a small wrapper:

```ts
import { createClient } from './client';

const client = createClient(agent, canisterId);
const reply = await client.evaluate({
  account: 7n,
  revision: 42n,
  payload: new Uint8Array([1, 2, 3]),
});
// reply.revision === 43n
```

This is compiling evaluation code in [client.ts](../examples/browser/src/client.ts),
not an exported IC Wire library API. The agent is supplied by the application;
its identity, signatures and certificate verification remain SDK responsibilities.
The wrapper uses raw `HttpAgent.update` bytes instead of a Candid actor for this
endpoint. Other Candid endpoints can retain their existing actors.

## Reproduce

Prepare Rust 1.99.0 with `wasm32-unknown-unknown`, Node 24.21.0, npm 12.2.0 and
PocketIC server 16.0.0. Install browser dependencies explicitly from the repo root:

```sh
npm --prefix examples/browser ci --ignore-scripts
npm --prefix examples/browser run prepare-browser
make check-browser
POCKET_IC_BIN=/absolute/path/to/pocket-ic make evaluate-browser
```

The evaluation command builds the Postcard canister with the root
`canister-evaluation` profile and a native fixture server, regenerates bindings,
checks TypeScript, bundles the browser entrypoint and launches Chromium. It closes
its browser and fixture afterward. It never installs tools automatically.
Generated packages are private; bindings, node_modules, bundles, connection data,
screenshots and reports stay ignored under `examples/browser/`.

## Evidence

The [saved report](evidence/browser-interop.json) records compiler, browser, SDK,
generator and source/artifact hashes. The run used SDK 6.1.1, postcard-bindgen
0.9.1 and Chromium 156.0.8078.4 (Playwright 1.64.1). The npm lockfile pins the
browser tooling; Cargo.lock pins Postcard and binding generation.

- Three signed updates matched Rust request bytes and response values, using empty, 128-byte and
  4,096-byte payloads. Account/revision values include integers above JavaScript's
  safe range, `u64::MAX`, and wrapping revision output zero.
- All 4,264 truncated response prefixes rejected. Local checks also rejected
  oversized payloads, trailing bytes, wrong contract/direction, nonminimal varints
  and out-of-range outbound integers.
- A real oversized call failed at the installed canister's wire limit.
- A separate agent retaining the mainnet root key rejected the local certificate.
  The positive local fixture explicitly fetches its local root key. The reusable
  wrapper does not fetch or replace root keys.
- TypeScript rejection fixtures enforce bigint and Uint8Array caller fields.

The 341,942-byte bundle includes the SDK and browser test code, without production
bundle optimization. It is not a codec size or deployment saving measurement.
Earlier [canister size measurements](canister-comparison.md) retain their original
source/lock hashes; those sizes were not remeasured after adding browser generation.

## Remaining limits

The generated decoder allocates arrays from encoded lengths, so this wrapper
first validates the complete fixed schema without allocating from those lengths.
It enforces 5,120 outer bytes, 4,096 payload bytes, canonical bounded integers,
contract/direction identity and exact input consumption. Re-encoding checks the
canonical bytes after decoding. Generated u64 decoding returns numbers for small
values despite bigint declarations; the wrapper normalizes both cases to bigint.
These guards are handwritten for this one fixture and must be generated or
otherwise maintained with the schema before general use.

Only u32/u8/u64 fields and one bounded byte vector are qualified here. Strings,
options, enums, nested collections and general generator correctness are not.
The fixture still derives its wire order from its Rust declarations; this trial
does not resolve stable schema identity or general declaration-order independence.
No query endpoint, Internet Identity/delegation flow, production deployment or
cached-frontend rollout is tested. The signed test identity is ephemeral. Storage
formats, sibling repositories and external Candid contracts are unchanged.

The practical frontend direction is generated bindings plus a typed bounded
client, while preserving the application's authenticated agent. The selected
Postcard engine is viable for this endpoint; a general frontend contract still
needs those unresolved checks before adoption.
