# Runtime 0.19.1 / Host 0.74.3 release qualification

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Core 0.16.2 preparation selects published Runtime/macros/exact guard **0.19.1**
and native Host artifacts/fs/process **0.74.3**. Facade 0.21, Testkit 0.84.1 and
canonical Types 0.18 stay selected. Compatible manifest floors are unchanged;
Endpoint's existing published ^0.19.0 range admits the Runtime patch, so Endpoint
remains 0.6.1. There are no supplier patches or codec changes.

Rust/Cargo 1.99.1, x86_64 Linux; a fresh wasm32 evaluation guest (opt-z, LTO,
one codegen unit, panic abort, stripped) passes the installed
`typed_calls_limits_and_transactions` test on the explicitly prepared standard
PocketIC 16.1.0 server through Testkit. It covers typed native requests/replies,
wrong/malformed/over-budget requests, owned/borrowed paths, async/guards,
rejections and synchronous rollback. Native lint/format admission also passes.
The fresh guest is **100,637 total / 86,698 code-section uncompressed bytes**,
SHA-256 `b824d9cd743c9ea5ccfef4ab79eba0c2e82c2fa8fe1a1561ec7438b7ac8c0e13`. This is a footprint observation, not a paired savings or
instruction/cycle measurement. The prior codec matrices retain their original
Runtime 0.19.0 / Host 0.74.2 source and are not relabelled as this supplier graph.

The Retired raw output (see retention notice) preserves exact root manifest/lock/compiler
pin, Endpoint source and installed test, runner/check scripts, logs and actual
guest. Extract to a separate inspection directory; reproduce from the preserved
source revision plus approved lock selection with the recorded compiler and a
separate target directory. Run `make check-endpoint-ic` with explicit
`POCKET_IC_BIN` and `IC_TESTKIT_POCKET_IC_URL` unset. The complete source-bound
release gate still must pass before metadata preparation and delivery.

All production Wire frame bytes, schema/identity rules, admission and original
retry/signature/hash inputs remain unchanged. Candid stays in unpublished
qualification. No independent Endpoint release is implied.
