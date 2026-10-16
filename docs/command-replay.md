# Protected command and replay trial

This evaluation-only Postcard fixture qualifies one local state-changing command
under direct IC caller authorization. It is not a Canic adapter, delegated-proof
implementation or production replay API. The `replay` feature is excluded from
the matched codec measurement builds; historical size/performance evidence is
unchanged. The production `ic-wire` library still has no runtime codec API.

## Contract and ownership

The explicit request tuple is `(contract: u32, direction: u8, subject: bytes,
audience: bytes, expiry_ns: u64, request_id: u64, amount: u64)`. Contract is
`0x5250`, direction is zero, subject must equal the actual IC caller, audience
must equal the receiving canister and expiry must be strictly after IC time.
Only controllers may invoke the fixture. Subject/audience are borrowed slices
during decode; the 128-byte argument cap is checked before copying IC arguments.
Exact consumption and re-encoding reject trailing and noncanonical input.

The response tuple is `(contract: u32, direction: u8, request_id: u64,
result_tag: u8, balance: u64)`, bounded to 32 bytes. Direction is one; explicit
result tag zero means applied, one means arithmetic overflow. Application failure
is a normal encoded response; admission/codec failures reject the call. These
tags are deliberately specified here, not derived from Rust enum order.

The fixture owns a local counter and eight stable receipts. The key is actual
caller plus request ID. An identical request returns the retained response;
the same key with different bytes rejects. The counter and receipt are written
within one synchronous IC update with no await, so they commit or roll back
together. Arithmetic overflow leaves the counter unchanged and retains its
failure response. No receipt is evicted: new identities reject at capacity,
while existing identities remain replayable. Authorization and expiry are checked
before replay, so an expired request rejects even if a receipt exists.

Storage is a private fixed-size fixture layout: a 16-byte balance/count header
and eight 162-byte slots containing request/reply lengths and retained bytes.
It occupies one stable-memory page; only the first 1,312 bytes are used. No
upgrade hook or migration is involved: the trial upgrades the identical Wasm
and reads the same contract's stable bytes. This does not propose a shared
storage format or a general durable queue for consumers.

## Evidence

The [saved report](evidence/command-replay.json) binds the checked source/lockfile,
Wasm and server hashes, compiler, build flags and exact request/reply vector.
Rust 1.99.0, `wasm32-unknown-unknown`, the `canister-evaluation` profile and the
`replay` feature were used with Postcard 1.1.3 and PocketIC server 16.0.0/client
16.1.0 on an application subnet.

Installed assertions cover:

- Discarding a successful response, retrying the exact bytes and observing only
  one applied effect and one receipt. This models client uncertainty; it does
  not inject actual network reply loss.
- 36 admission/truncation rejections with unchanged stable memory, including
  wrong subject, wrong audience, expiry, conflicting bytes and a non-controller.
- A trap after stable write rolling back both effect and receipt; retry then
  applies the command once.
- Identical Wasm upgrade followed by byte-identical replay of both earlier
  responses, with no stable-state change.
- Retained overflow failure, full-table rejection and successful old-request
  replay at capacity. Final balance is 17 with eight receipts.

Two native replay tests additionally cover nonminimal varints, trailing bytes,
all request truncations, caller-scoped identities and capacity without mutation.
The combined peer/replay library suite passes seven tests. Clippy passes with
all features; Rust 1.91 host and Wasm checks pass for the replay feature.
The first sandboxed PocketIC attempt could not bind a loopback port; the local
run with loopback permission passed. No sibling or broad repository suite ran.

Reproduce with a prepared server and dependencies:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic make evaluate-replay
```

The report and Wasm are retained under `target/replay-evaluation-evidence/`.
This is correctness evidence, not a new Wasm-size or instruction comparison.
Delegated proofs, browser command bindings, external-effect reconciliation,
general generated nested Result layouts and consumer rollout remain unqualified.
