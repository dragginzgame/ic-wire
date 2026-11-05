# Host schema cleanup qualification

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Consolidate host metadata's work/depth ceilings and the named-definition loop
used by standalone schema validation and TypeScript generation. Each traversal
keeps its independent counter and original boundary comparison; the inclusive
renderer limit and projection/lint limits are deliberately not made identical.
Name/shape/alias checks retain their order and share the same remaining allowance.
No public API, schema acceptance, diagnostics, generated clients, runtime codec,
frame bytes, budgets or retained retry/hash inputs change.

Baseline is the working tree at `fcfcf5b795213cb3a0cb5857781fae7eb48f33a3`
including the pre-existing Testkit 0.80.1 / Runtime, macros and guard 0.18.1
lock selection. That selection is preserved; before/after manifests and lockfiles
are identical. Retired raw output (see retention notice) bind separate frozen source
copies, each with its own initially empty probe and Wasm target directories.
Retired raw output (see retention notice): Rust/Cargo 1.99.1, x86_64 Linux. Host probes use opt-3,
LTO and one codegen unit; guest profiles retain the root evaluation settings.

Host generation/validation medians over **11 alternating paired batches**:

| Operation / named definitions | Before ns | After ns | Change |
| --- | ---: | ---: | ---: |
| Frame generation / 0 | 1,320 | 1,273 | −3.58% |
| Frame generation / 1 | 3,920 | 3,739 | −4.63% |
| Frame generation / 100 | 106,021 | 107,697 | +1.58% |
| Payload generation / 100 | 104,365 | 99,857 | −4.32% |
| Standalone validation / 100 | 70,524 | 66,781 | −5.31% |
| Native collection / 100 | 149,388 | 147,174 | −1.48% |

All 15 controls, sample ranges, iterations and absolute results are in
Retired raw output (see retention notice) and Retired raw output (see retention notice). The largest median increase
is +1.87% for empty validation (about 22 ns); the 100-definition frame case
increases about 1.68 µs. Other medians fall or stay level. This small single-host
screen supports a behavior-preserving cleanup, not a general speedup claim.
No native codec or IC instruction savings are claimed for host-only changes.
All generated frame/payload modules and collected registries are byte-identical.

Fresh guest builds target `wasm32-unknown-unknown`; Cargo reports `fresh=false`:

| Profile | Total uncompressed Wasm, before → after | Code section, before → after |
| --- | ---: | ---: |
| evaluation (opt-z) | 100,637 → 100,637 | 86,698 → 86,698 |
| benchmark-speed (opt-3) | 93,997 → 93,997 | 81,012 → 81,012 |

Both complete modules are byte-identical: **0 bytes / 0% change**. These are this
matched library fixture and graph, not historical cost-example sizes or projected
application savings. Retired raw output (see retention notice) retain the binding;
compressed size is not a Wasm metric.

Focused checks pass: 60 native/schema tests including shared total-work and
depth boundaries, alias cycles, unused definitions, collection poisoning and
diagnostics; strict host-library lint; generated TypeScript compilation;
176 Rust/browser vectors and 38,556 differential mutations; payload, admission,
large-byte-buffer and simulated big-endian conformance. The current lock also
passes the installed Endpoint fixture and benchmark instruction-driver compilation.
This is not a before/after qualification of Runtime 0.18.0 → 0.18.1. No full repository gate
or new deployment is inferred.

Retired raw output (see retention notice) include frozen sources, locks,
compiler/artifact messages, complete samples and probes. Build the archived
`before/probe` and `after/probe` with separate `CARGO_TARGET_DIR` values using
Rust 1.99.1 and `--locked --offline --release`, after explicitly preparing the
locked cache. Retired raw output (see retention notice) invokes both binaries alternately,
warms each operation eight times, compares outputs and records results. Small
controls use 800 iterations per batch; 100-definition controls use 40.
Retired raw output (see retention notice) reads Cargo's reported artifact paths;
reconcile those paths when moving retained inputs. All original artifacts remain
under `target/schema-cleanup-0151-20261102/`. Retired raw output (see retention notice) binds the record.
