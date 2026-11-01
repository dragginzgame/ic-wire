# Platform 0.16 adoption

Require published **ic-platform 0.16.0**, with macros and the singleton runtime
guard at 0.16.0. Keep canonical **ic-platform-types 0.15.1** and the independent
Core codec/schema identity. [Suppliers](suppliers.json), [manifest](Cargo.toml.snapshot)
and [lock](Cargo.lock.snapshot) retain exact registry identities and checksums.

This is pending **Endpoint 0.3.0**, because application runtime suppliers,
including timers, must align on Platform 0.16 rather than mixing executor owners
from 0.15 and 0.16. Core's compatible pending version stays **0.13.3**. Actual
member versions stay the published Core 0.13.2 / Endpoint 0.2.1; no release,
staging, commit, push, publication or sibling checkout change was performed.

Testkit 0.75.2 still requires native-only Platform 0.15.3. That host facade can
coexist: the production Wasm endpoint graph has **one 0.16 runtime/guard**, and
schema-only consumers select no CDK/executor. Both facades reexport the same
canonical Types 0.15.1 and external PocketIC 16.1.0 identities. No wrapper,
downstream Cargo patch, second schema trait or second codec was added.

The incoming lock already selected Build/Host **0.73.4** and Metrics **0.24.4**;
those edits are preserved. Build's requirement and installed release editor
remain 0.73.3. [Preservation checks](preservation.json) verify that non-Platform
package selections remain intact. Earlier benchmark and adoption reports retain
their original frozen dependencies; this update does not reinterpret them.

## Changes and qualification

The selected registry CDK, executor, IC0, Candid, Principal and Account source
trees contain **43 files, all byte-identical** to 0.15.3.
[Source review](source-review.json) retains their per-file hashes. Crypto ownership,
Dalek and K256 changes are outside the selected production feature graph. The
entrypoint generator uses Syn 3 and scopes native symbols by the consuming Cargo
crate, retaining exact Wasm export names. Runtime guard source is unchanged.

Our direct-rustc diagnostic recipe now explicitly supplies
`CARGO_CRATE_NAME=ui_contract`, matching its existing `--crate-name ui_contract`.
Cargo normally supplies this variable; no application helper is needed. The
initial endpoint compile gate caught the missing variable, and qualification
resumed at that failed diagnostic step after the recipe was fixed. The
[initial log](checks/check-endpoint.log) and
[successful continuation](checks/check-endpoint-resume.log) remain separate.

Focused checks pass on Linux x86_64, Rust 1.99.1 and the Rust 1.91.0 floor:

- Three endpoint fixture tests: borrowed custody, actual Contract projection,
  defaults and overrides. Existing automatic identity/frame vectors pass.
- Eleven expected endpoint diagnostics plus passing control; nine ordinary
  codec diagnostics plus passing control. Strict Clippy and diagnostic-runner
  minimum-version compilation pass.
- Endpoint Wasm build, Rust 1.91 Wasm check and production graph isolation.
- Standalone Principal/Account derives, schema-only and combined endpoint
  native/Wasm graphs, Rust 1.91 tests, Clippy and generated TypeScript byte and
  malformed-input agreement. Absent/explicit-zero subaccounts remain distinct.
- Current and simulated-next runtime consumer checks preserve exact
  Core/derive/Types identities; the simulation is not an unpublished API promise.
- The installed endpoint fixture passes actual admission, malformed and wrong
  identity refusals, request/reply boundaries, borrowed replies, async/composite
  calls, framework hooks, synchronous rollback and async transaction boundaries.
- All 333 standard matched Wire/Candid cases roundtrip/refuse damaged frames;
  Rust 1.91 benchmark all-target compilation with wire,candid,ic passes.
- Dependency boundaries, declared pins, formatting and documentation links.

[Focused logs](checks/), [type graphs](type-graphs/),
[identity graphs](identity-graphs/), [core graph](core-graph.txt),
[endpoint graph](endpoint-graph.txt) and [compiler](compiler.txt) retain evidence.
The installed check uses Testkit-owned lifecycle with the explicitly prepared
PocketIC server 16.0.0, SHA-256
`69e324bdb68d32d878b7a9504b1379f08f8d1921272bacb065b0fabb3d0f3792`.
This is local simulator qualification, not live IC, macOS or consumer fleet tests.

## Protocol and reproduction

Wire bytes, field order, budgets, primitive/malformed acceptance, Wasm method
names, transaction rules and retained retry/hash inputs are unchanged. Principal
remains bounded bytes max 29. Account absent `01 04 00` and explicit zero
`01 04 01` plus 32 zero bytes remain distinct. Native symbol scoping changes host
link names, not the frame contract. No application DTO regeneration is needed
for the Platform adoption; runtime suppliers still require coordinated alignment.

No runtime codec cost change is introduced: the selected runtime/Candid bodies
and Wire generator implementation are unchanged. A fresh full instruction/Wasm
performance matrix was not run and no performance or size saving is claimed.
No full repository CI was run for this focused adoption.

[Source capsule](source.tar.gz) contains actual Rust inputs, including existing
untracked ByteArray/corpus additions, manifests, lock and qualification scripts.
After preparing the recorded compilers, Cargo cache, Wasm target and the existing
Node/TypeScript compiler, reproduce from that source in an isolated directory:

```sh
bash scripts/ci/check-endpoint.sh
bash scripts/ci/check-diagnostics.sh
bash scripts/ci/check-platform-types.sh /absolute/path/to/prepared/typescript/bin/tsc
bash scripts/ci/check-stable-identities.sh
POCKET_IC_BIN=/absolute/path/to/matching/pocket-ic bash scripts/ci/check-endpoint-ic.sh
cargo test --locked --offline -p ic-wire-benchmarks --test roundtrip --features wire,candid
cargo +1.91.0 check --locked --offline -p ic-wire-benchmarks --all-targets --features wire,candid,ic
cargo +1.91.0 check --locked --offline -p ic-wire-qualification --example check-diagnostics
```

The isolated type fixtures reconcile only their disposable locks; later checks
use locked/offline resolution. Complete temporary outputs remain under
target/platform-016-adoption, target/endpoint-check.TmutW5,
target/platform-types.ndKytd and target/stable-identities.lBciHq. Normal Cargo
cleanup may remove them. SHA256SUMS binds permanent evidence and the source capsule.
