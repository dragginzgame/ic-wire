# Historical browser and inter-canister Postcard trial

This document describes the retained 0.1.3 evidence. Current browser source uses
the [generated runtime integration](browser-interop.md); the examples below are
historical and are not the current client API.

The same scalar/blob contract worked on both a direct browser call and a
browser → forwarding canister → recipient canister path in local PocketIC.
This is an evaluation fixture, not an implemented Canic adapter or public codec API.
The [saved report](evidence/peer-interop.json) binds the run to source and artifact
hashes, compiler, SDK, generator, browser and emulator versions.

## One payload contract across both paths

Rust generates the existing Message bindings and golden vectors. The browser's
bounded client encodes the request once. The forwarding endpoint validates it,
retains those exact bytes and passes them through the CDK's raw-argument call to
`evaluate` on the configured recipient. It explicitly supplies argument bytes;
it does not rely on the transport's default empty Candid arguments.

The forwarder performs two sequential calls for each input, using the retained
request buffer across awaits. Each reply must be bounded, canonical, have the
expected contract/direction, echo the account/payload and increment revision with
wrapping arithmetic. Both replies must also match byte-for-byte. The first reply
is returned unchanged to the browser for the same generated decoder to consume.

The historical client exposed:

```ts
const direct = createClient(agent, recipientId);
const viaPeer = createClient(agent, forwardingCanisterId);
const request = { account: 7n, revision: 42n, payload: new Uint8Array([1, 2]) };
const a = await direct.evaluate(request);
const b = await viaPeer.evaluateViaPeer(request);
// Both replies carry revision 43n and the same account/payload.
```

The [peer module](https://github.com/dragginzgame/ic-wire/blob/226a11fce6d5cdd59a77d9a006c91f5a46c58b42/crates/ic-wire-canister-evaluation/src/peer/mod.rs) is behind
the evaluation-only `peer` feature. The historical browser command enabled it; matched
size/instruction experiments do not. Fixture configuration requires the actual IC
caller to be a controller. The recipient exposes a bounded observation containing
its call count, whether its two latest requests match, and its last actual caller.
This is test instrumentation, not a user identity propagation protocol.

## Evidence

The final Chromium run passed:

- Three direct and three forwarded signed browser updates: empty, 128-byte and
  4,096-byte payloads, including u64::MAX and values above 2^53.
- Six actual inter-canister calls, with three identical-request repetitions and
  identical response bytes. For each input the recipient reported the forwarder's
  principal as its caller and the expected increment in its received-call count.
- Oversized and malformed sender arguments rejected before peer dispatch; the
  recipient's call count did not increase.
- An unconfigured peer and a call to a canister without code rejected with distinct
  internal failure categories. A signed browser identity could not configure the
  peer because it was not a controller.
- The existing 4,264 truncated-response checks and wrong-root certificate rejection
  still passed. Native reply checks reject oversize, malformed/trailing bytes,
  wrong contract/direction, and incorrect account/revision/payload under both codecs.

`make check-canister-evaluation` runs five native tests for each codec selection
and checks host/Wasm compilation on Rust 1.99 and 1.91. Browser execution used
Node 24.21.0, SDK 6.1.1, Chromium 156.0.8078.4 and PocketIC server 16.0.0.
No runtime timing or linked-size claim is made for this transport trial.

## Bounds and remaining work

Ingress length is checked before argument copying. Peer reply length is checked
before application deserialization/re-encoding; the CDK already owns the received
transport buffer at that point. This does not prove a 5,120-byte cap on allocations
inside the transport. The fixture observes at most one retained previous request,
not an unbounded history.

The repeat is deliberate and safe only because this handler is stateless. It is
not a production retry policy, durable replay receipt, exactly-once guarantee or
interruption-recovery proof. The original browser caller is not automatically the
recipient's caller: the receiving canister sees the forwarder. A protected command
must bind the intended caller/proof explicitly and preserve Canic's auth ownership.

The contract still uses its reviewed experimental declaration order. Nested
Result/error schemas, durable field/variant identity, general generated bounds,
authorization and committed-effect replay remain unfinished. Transport failures
and codec failures are distinguished internally and surfaced as IC rejection text;
a typed public application-error contract has not been established.

The current command below runs the replacement runtime fixture; reproducing this
historical report requires its recorded source snapshot. See the [browser setup](browser-interop.md):

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic make evaluate-browser
```

The prior browser and nested-authority evidence remain historical snapshots. No
sibling repository, production API, dependency selection or package version changes
as part of this trial. The next integration proof is a protected command with
bounded proof extraction and retained-response replay, kept outside codec policy.
