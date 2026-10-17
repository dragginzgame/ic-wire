# Generated runtime browser integration

Current browser checks use a bounded volatile reply cache. Durable receipt layout,
rollback and upgrade qualification now belong to the Relay receipt probe. The
saved JSON retains its historical source identity; fresh browser reports qualify
transport, generated codecs and same-instance exact-byte responses only.

The browser now uses the same explicit `Wire` and `Method` declarations as the
installed Rust endpoints. The current fixture replaces its old private Message
layout and hand-written preflight guards with the optional IC Wire generator.
The [saved runtime report](evidence/runtime-browser.json) records source/artifact
hashes, exact Rust vectors and compiler, SDK, browser and emulator identities.
The [runtime guide](runtime.md#typescript-generation) defines the generated API.

[client.ts](../examples/browser/src/client.ts) is a small transport adapter:

```typescript
const client = createClient(agent, canisterId);
const snapshot = await client.inspect({
  labels: ['é🦀'], correlation: { some: 42n },
});
const receipt = await client.apply({
  subject: identity.getPrincipal().toUint8Array(),
  audience: Principal.fromText(canisterId).toUint8Array(),
  expiry, id: 1n, amount: 7n,
});
```

The generated modules own field order, variant IDs, bounds and canonical envelope
encoding. The application supplies its authenticated SDK agent; signatures,
query verification, update polling and certificate verification stay with the SDK.
There is no Candid actor or Candid selection in this client. The evaluation crate
still uses Candid/CDK for existing fixture infrastructure; the production runtime
dependency graph remains independent of them.

## Reproduce

Prepare Rust 1.99.0 with `wasm32-unknown-unknown`, Node 24.21.0, npm 12.2.0 and
PocketIC server 16.0.0. Install browser dependencies explicitly from the repo root:

```sh
npm --prefix examples/browser ci --ignore-scripts
npm --prefix examples/browser run prepare-browser
make check-typescript
make check-browser
POCKET_IC_BIN=/absolute/path/to/pocket-ic make evaluate-browser
```

The command builds the canister with feature `replay`, target
`wasm32-unknown-unknown` and profile `canister-evaluation`. A host-only `bindings`
feature enables `ic-wire/typescript`; it is not enabled in that Wasm build.
The host fixture generates `apply.ts`, `inspect.ts`, `wire-runtime.ts` and exact
Rust vectors. TypeScript checks include expected failures for number/u64,
array/byte-buffer, option and enum-tag mismatches. Esbuild bundles the browser
entrypoint; Playwright launches Chromium against disposable local canisters.
No dependencies or browser binaries are installed by the check. Generated files,
node_modules, bundles, connections, screenshots and reports remain ignored under
`examples/browser/`. The harness closes the browser and local fixture afterward.

## Qualified behavior

The Chromium 156.0.8078.4 / SDK 6.1.1 / PocketIC 16.0.0 run passed:

- Nested query request/reply bytes match Rust, preserving UTF-8, a leading BOM,
  bounded lists and an optional `u64::MAX` correlation value.
- Signed protected updates use principal bytes, bigint and explicit Result/enum
  projections. Success and Overflow replies match Rust vectors. Exact request
  retries return retained success and failure replies without duplicate effects.
- Wrong subject, audience, expiry, conflicting bytes and a noncontroller caller
  reject. Oversized/malformed input rejects; querying state confirms no mutation.
- A signed browser call to `runtime_forward` produces two peer deliveries and one
  effect. The receiver validates the forwarder's actual principal. Invalid,
  oversized and unauthorized forwarding and an unavailable recipient reject.
- All 68 truncated reply prefixes reject with typed codec errors. Trailing bytes,
  wrong method identity, excess text and invalid outgoing UTF-16 reject too.
- An agent retaining the mainnet root key rejects the local certificate. This
  negative test replays an existing command, so submission cannot add an effect.
  Only the local harness explicitly fetches its local root key; the adapter does
  not change root keys or disable verification.

The fixture uses a public deterministic test seed and explicitly grants that
principal controller access. This qualifies controlled direct-caller auth, not
Internet Identity, delegated proofs, session expiry or production access policy.
The SDK owns raw transport buffers before codec validation; codec bounds do not
retroactively limit that allocation or measure JavaScript engine heap use.

The bundle contains SDK and test code without production optimization. Its byte
count is recorded for reproducibility, not as codec size or deployment savings.
No native timing, IC instruction or Wasm savings comparison is made by this run.
Canic macros, composite queries, external-effect recovery and coordinated cached
frontend rollout still require their own integration work. Siblings are unchanged.

## Historical evidence

The original [single-canister report](evidence/browser-interop.json) and
[two-canister report](evidence/peer-interop.json) remain unchanged snapshots of the
private Message layout using postcard-bindgen 0.9.1. Their source hashes identify
that earlier implementation. Current source retires those browser guards and the
postcard-bindgen dependency; there is no compatibility decoder for that layout.
The [historical peer description](peer-interop.md) and independent codec size
comparisons retain their original scope. Current reproduction runs the runtime
fixture described above, not those historical artifacts.
