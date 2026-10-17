# IC Wire CDK endpoint companion

An unpublished companion for raw, manually replied IC endpoints. The `ic-wire`
codec remains independent of CDK and Candid; this crate intentionally depends on
CDK, which can retain Candid transitively. Exports, permission policy, handlers,
lifecycle, retry and external-await recovery remain consumer-owned.

In a raw `#[ic_cdk::update(manual_reply = true, hidden = true)]` or query endpoint,
call `ic_wire_cdk::endpoint::read::<YourMethod>(authorize)` before the handler and
`ic_wire_cdk::endpoint::reply::<YourMethod>(&response)` afterwards. Export the
name selected by `YourMethod::NAME`; this helper does not generate exports.
`authorize` is a synchronous closure that traps unless the caller is admitted.
For installation, the lifecycle owner explicitly selects its authorization closure.

The reader runs authorization before reading the raw argument size, checks that
method's byte limit before copying, and traps on malformed input. The writer
traps on encoding failure so current-message writes roll back. It does not undo
writes before an await or previous messages. Encoded application errors remain
ordinary replies. Raw replies received through CDK calls still allocate before
codec admission; this crate does not own callback allocation or execution.

Focused checks from the repository root:

```sh
cargo test --locked -p ic-wire-cdk
cargo clippy --locked -p ic-wire-cdk --all-targets -- -D warnings
cargo +1.91.0 check --locked -p ic-wire-cdk --target wasm32-unknown-unknown
cargo build --locked -p ic-wire-canister-evaluation --lib --release --features cdk-fixture --target wasm32-unknown-unknown
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic cargo run --locked -p ic-wire-canister-evaluation --example cdk-fixture --features cdk-fixture -- target/wasm32-unknown-unknown/release/ic_wire_canister_evaluation.wasm
```

The installed fixture checks typed query replies, malformed and over-limit input,
authorization before the size refusal, and rollback of stable growth/writes on a
response codec failure. This qualifies synchronous raw endpoint mechanics, not
Canic macros, async recovery, composite queries or production authorization policy.
