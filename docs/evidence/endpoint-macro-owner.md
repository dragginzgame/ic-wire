# Endpoint-owned proc macros

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

The unpublished working-tree split moves the existing query/update implementation
from `ic-wire-derive` to `ic-wire-endpoint-macros`, versioned in the Endpoint group.
Public `ic_wire_endpoint::{query, update}` imports, defaults, options and generated
contracts stay unchanged. Core's derive retains only Wire metadata; its endpoint
feature/direct attributes are retired without a compatibility lane. Pending
Core 0.17 / Endpoint 0.7 notes describe the coordinated API/identity cut. Actual
existing package versions remain unchanged; the new member uses the existing
Endpoint group's 0.6.1 seed until ordinary group preparation.

Publication is Endpoint macros then Endpoint; Core derive/Core stay in their own
group. The catalog uses the same compatible intra-group requirement policy as
Core's derive. Manifest or shared-input changes still select the complete gate;
subsequent macro-only source/docs changes qualify for Endpoint's mapped gate.
Platform still owns exports/execution, with one runtime guard and one codec.

## Exact generated output and uncompressed Wasm

Base source is `e1c15a4` plus the preserved incoming Build 0.74.3 lock selection.
Both frozen sources select identical registry dependencies, Rust/Cargo 1.99.1 /
LLVM 23.1.1, Runtime/macros/guard 0.19.1, Types 0.18, facade 0.21, Testkit 0.84.1,
Build/Host 0.74.3 and metadata editor 0.74.0. No supplier patches. Each source has
its own fresh Cargo target directory. Fresh compiler-artifact records, profiles,
actual section sizes and complete module hashes are retained.

| Whole Endpoint fixture | Before total / code | After total / code | Delta |
| --- | ---: | ---: | ---: |
| opt-z | 100,413 / 86,476 | 100,413 / 86,476 | 0 / 0 |
| opt-3 | 93,997 / 81,012 | 93,997 / 81,012 | 0 / 0 |

The **entire Wasm bytes are identical** per profile, not just their sizes:
opt-z SHA-256 `3d55d753902fe09bd5fe2c74628768b74cc198f3029bb8c66e0c72b9e262d981`;
opt-3 `0966619eb23968458488ce156b7dce8ff8e6e8450cfba1978e0fb39d8efbbab0`.
Both profiles use LTO, one codegen unit, abort and stripped output, differing only
by optimization level. Retired raw output (see retention notice) retain all four
builds. No size saving is claimed. A new IC instruction/cycle experiment was not
run: the measured before/after inputs produce the exact same executable module.
That is an artifact-identity proof of unchanged code, not an asymptotic savings
claim. Existing installed Runtime 0.19.1 evidence remains applicable to these
identical modules. Compression is archive storage only.

The maintained expansion probe also asserts exact token-string equality for
unit/default, scalar/default, explicit limits/contract, async update, borrowed
query/update, renamed composite and local-identifier/cfg cases: eight paired
shapes in **two runs**, eleven alternating-lane samples of 1,000 expansions after
warmups. Both implementations share the same token inputs and dependency rlibs.
The Rust harness uses `rustc -O`; dependency rlibs come from the dev build.
This times host expansion including output traversal, excluding Platform's nested
macro, Rust const evaluation and whole-project compilation.

All host medians and Retired raw output (see retention notice) retain improvements
and regressions. Run one drifted strongly (unit before/after 224,057 / 129,743 ns,
versus 64,555 / 67,892 in run two). Scalar defaults are 76,557 / 76,471 ns and
73,049 / 73,431; async update 112,636 / 113,022 and 112,125 / 111,762. Renamed
composite regresses +2.31% / +3.94%; borrowed query +7.70% in run one and -2.17%
in run two. These noisy data do not establish a host speed improvement or a
whole-project compile-time result. The cleanup's benefit is release ownership.

## Focused proofs

- Core metadata derive unit tests, 62 native/schema tests, Endpoint native/schema
  tests and strict derive/Endpoint-macro/Endpoint lint pass.
- The focused Endpoint check covers 11 expected diagnostics plus a passing
  control, native consumers, renamed macro imports, Wasm and dependency isolation.
- Core's normal/build graph excludes Endpoint and its generator, SDK/client/
  Candid packages; Core alone does not select `syn/full`.
- A simulated future Endpoint **and macro package**, with future crypto facade,
  keeps Core/derive/Types/runtime/guard IDs fixed. Retired raw output (see retention notice)
  show changed Endpoint/generator IDs and unchanged canonical identities.
- Consumer Make routing/publication selectors include both Endpoint packages,
  supplier-first, with cross-group refusal. Plan fixtures admit macro-only paths
  to the same qualified 11-stage selection; other hosts/configurations keep full.
- Both real group graphs, a verified local proc-macro package archive,
  dependency boundaries/pins, documentation and formatting pass.

The changed Make/plan fixtures run only disposable Git effects. No real staging,
commits, own-version preparation, release, upload or sibling checkout edits.
Full CI, hosted/native macOS, Candid codec matrix and installed endpoint reruns
are not claimed or needed to establish a byte-identical ownership move.
