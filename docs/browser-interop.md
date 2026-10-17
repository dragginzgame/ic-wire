# Generated runtime browser integration

The maintained browser fixture checks generated codecs and SDK transport against
one disposable canister. It has no receipt cache, expiry/audience policy or peer
retry path. Relay and consumers own delivery policy; historical reports below
retain their original scope and source identities.

The browser and Rust endpoints share explicit `Wire` and `Method` declarations.
[client.ts](../examples/browser/src/client.ts) supplies the small transport adapter:

```typescript
const client = createClient(agent, canisterId);
const snapshot = await client.inspect({
  labels: ['é🦀'], correlation: { some: 42n },
});
const receipt = await client.apply({ id: 1n, amount: 7n });
```

IDs here are echoed correlation values, not deduplication keys. Each admitted
update runs the handler again. The fixture uses a fresh method identity for its
simplified request; only fresh disposable installations use this contract.
The generator owns field order, variant IDs, bounds and canonical envelopes.
The SDK owns signatures, query verification, update polling and certificates.

## Reproduce

Prepare the tool versions and dependencies in [development](development.md),
including the selected Testkit launcher, Node/npm and Chromium:

```sh
make check-typescript
make check-browser
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic make evaluate-browser
```

The shared build driver uses feature `browser-fixture`, target
`wasm32-unknown-unknown` and profile `canister-evaluation`. Host-only `bindings`
enables TypeScript generation. The host emits `apply.ts`, `inspect.ts`,
`wire-runtime.ts` and five exact Rust request/reply vectors. Playwright runs the
bundle in Chromium and retains `examples/browser/artifacts/report.json` with
source/artifact hashes, compiler, SDK, browser and emulator identities.

A native recursive echo query additionally compares browser metadata with an
independent Rust Serde tree, wide integers and canonical frame bytes. Its host
probe rejects excessive recursive depth. This tests the generic native API, not
IcyDB endpoint contracts.

## Maintained checks

- Nested queries preserve Unicode text, bounded lists and optional u64 values.
- Signed updates and typed success/Overflow replies match Rust vectors.
- Noncontrollers and malformed/oversized arguments fail; state checks detect
  unintended mutation. Reply truncation, trailing bytes and wrong identity fail.
- Outgoing text bounds and invalid UTF-16 fail; TypeScript rejects incorrect
  integer, option and enum projections.
- A wrong root key rejects the local update certificate. This uses a zero
  increment, so submitting the certificate probe changes no counter value.

The deterministic public identity is for disposable tests only. This checks
controlled direct-caller authentication, not login, delegation or production
permissions. SDK reply allocation precedes codec admission. No JavaScript heap,
IC instruction or deployed size savings are inferred from this correctness run.

## Historical evidence

The [saved runtime browser report](evidence/runtime-browser.json) covers the old
receipt/replay and peer fixture. The [original browser report](evidence/browser-interop.json)
and [peer report](evidence/peer-interop.json) cover the earlier private Message
layout and postcard-bindgen. These files remain unchanged. Their recorded source
identities, rather than current source, reproduce those retired experiments.

## Fixture lifecycle owner

The browser runner uses the private Node helper from
[IC Testkit PR #121](https://github.com/dragginzgame/ic-testkit/pull/121), selected
at `26fc0b062aebf208cc71e0e55dcd9f5880bbf663` in the npm manifest and lockfile.
The project npm configuration allows only root-declared Git dependencies.
Normal `npm ci` prepares that selection; the helper has no install scripts.
Testkit owns JSON readiness, append-only fixture logs and bounded stdin/TERM/KILL
cleanup. IC Wire retains the Rust fixture, browser/HTTP lifecycle, contract vectors
and report assertions. The build driver hashes the selected helper source and lock under report
`provenance.sources`, alongside retained Wasm receipts.
This is an unreleased upstream contribution, separate from the Rust Testkit pin.
