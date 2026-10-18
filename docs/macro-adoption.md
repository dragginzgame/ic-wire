# Local Canic-shaped macro proof

This fixture tests query/update attributes in IC Wire's own workspace while Canic
is changing independently. No sibling source is read as implementation authority
or modified. The attributes are evaluation-only, in the unpublished
`ic-wire-adapter-fixture` proc-macro crate. They are not re-exported by `ic-wire` or enabled in
its default dependency graph. Canic remains the production macro owner.

The following syntax compiles in
[the fixture](../crates/ic-wire-canister-evaluation/src/macro_fixture/mod.rs):

```rust,ignore
#[ic_wire_adapter_fixture::query(method = Read, guard = guard)]
fn macro_read(correlation: u64) -> u64 { /* fixture handler */ }

#[ic_wire_adapter_fixture::update(method = Add, guard = guard)]
async fn macro_add(amount: u64) -> Result<u64, u8> { /* fixture handler */ }
```

The bodies above are abbreviated; the linked source contains the tested bodies.
These attributes demonstrate a possible Canic expansion, not implemented
`canic::query` or `canic::update` syntax. No Candid argument/mode is supported.
The fixture also contains a synchronous unit-request/unit-reply update.

## What the expansion owns

The macro binds the handler input and synchronous return or future output to one
Method. A compile-time assertion ties Method::NAME to the exported handler name.
Explicit IC query/update exports are inspected in the built Wasm. Contract IDs
remain consumer-assigned; there is no cross-module ID registry in this fixture.

The synthetic adapter stages are startup check → raw argument length → actual
caller check → argument copy → strict request decode → request guard → handler →
response encoding → raw reply. Native instrumentation tests that order and proves
that startup, size, caller, decode and guard failures prevent handler execution.
This order is local test policy, not a claim about Canic's current lifecycle.
Canic must preserve its own required ordering when integrating the codec.

Application Result errors remain encoded replies. An encoding failure after a
handler runs traps, allowing the IC to roll back local state changes in that
transaction. A normal rejection there could commit a mutation without a usable
reply. The installed fault-injection endpoint deliberately has an inadmissible
response budget, writes state and verifies that the resulting trap rolls it back.
This does not recover effects committed before an external await. The async
handler awaits an immediately ready future; external-call recovery is unqualified.
The installed trial also injects that failure before the first successful update,
verifying that both zero-page growth and its initial write roll back.

The attributes emit `ic_platform::cdk` entrypoints for this experiment. They do not implement a
replacement executor or platform APIs. Native tests call the same expansion with
a mock transport; installed tests use real IC APIs and Platform's qualified CDK
runtime/executor. Checked caller/controller and allocating argument helpers use
`ic_platform::cdk::api`. The rollback fixture's stable read/write/grow use
Platform 0.1.12's `ic_platform::cdk::stable` exports; the direct CDK dependency
is removed ([Platform #20](https://github.com/dragginzgame/ic-platform/issues/20)).
The generated entry has no Rust arguments and uses `manual_reply = true` plus
`hidden = true`. This suppresses Candid service metadata while preserving callable
Wasm exports and the original executor contexts. There is no direct Candid crate;
CDK still contains Candid transitively. The full export roster is checked on Wasm.
Controller checks are the fixture's auth policy, not Canic authorization/delegation.
No automatic replay policy is inserted. The Add command has no receipts, so
repeating it produces another effect. The separate receipt-backed runtime trial owns
its explicitly tested retry policy.

## Checks and evidence

With locked dependencies and tools prepared:

```sh
cargo test --locked -p ic-wire-canister-evaluation --lib --features macro-fixture macro_fixture
make check-macro-diagnostics
POCKET_IC_BIN=/absolute/path/to/pocket-ic make evaluate-macros
```

The diagnostic command builds a valid isolated workspace first, then requires
the expected failures from the actual attribute expansions: wrong request type, wrong
sync/async response, export-name mismatch, a Candid selector, missing guard and
async query. Async/composite queries are deliberately outside this fixture.
Failures retain logs; all compilations use locked dependencies offline.

The [installed report](evidence/macro-adoption.json) identifies source hashes,
Wasm hash, Rust compiler, PocketIC binary, feature, profile, target and exact
request/reply bytes. It verifies query, sync and async update exports, encoded
application errors, 16 input/auth/guard rejections without state mutation, reply
failure rollback, unit envelopes and repeated-command effects. It is correctness
evidence, not a size, timing or instruction-saving measurement.

The update endpoint rejects query invocation. PocketIC also accepts a replicated
call to the query endpoint through its update API; the test records the correct
reply and unchanged state instead of assuming that direction must reject.

Canic can now port this small expansion pattern into its own macro owner, replace
the synthetic hooks with its actual lifecycle/auth stages and run equivalent
macro-owned compile-fail and installed tests. Browser codecs, raw transport limits,
request hashes, durable replay and external-effect policy retain their existing
owners. The existing browser proof tests shared runtime contracts, not these
new fixture-specific endpoint IDs.
