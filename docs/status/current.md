# Current handoff

## Released baseline and pending batch

0.6.11 is released at `445aa56`. Pending 0.6.12 work strengthens packed-field
qualification and records two rejected optimization screens. Production runtime,
wire bytes, admission rules, metadata and exact retries are unchanged.
The incoming Cargo.lock changes are preserved; no package versions were changed.
Staging, commits, publication, pushes and releases remain maintainer-owned.

## Packed qualification

[The current study](../packed-0612.md) records 4,144 offset/type/length checks,
canonical-prefix and truncation failures, and exact allocation/byte/node limits.
Seven packed variants now participate in runtime request/response fuzzing.
The runner now regenerates 112 deterministic seeds without archive restoration;
they match the previously qualified corpus exactly. TypeScript's normal check
also covers 48 simulated big-endian codec cases, and Rust's append-helper test
passes. Neither substitutes for a real big-endian target/engine run.
Default and TypeScript-feature native suites pass 17 tests. Seeded fuzzing passes
100,000 inputs with AddressSanitizer. LeakSanitizer cannot run under this host's
ptrace environment; the successful rerun disables leak checks only.

Native opt-z large-u16 decode remains about 7.5 µs. Forced reader inlining does
not fix it and leaves Wasm byte-identical. Aligning native functions restores
about 4.8 µs but grows native machine code by 9.2%; this is diagnostic evidence
of code-layout sensitivity, not an adopted compiler setting or portable fix.
The tiny TypeScript candidate saves 66–73% for single 16/32-bit words, but control
regressions prevent adoption. Production TypeScript stays unchanged. Raw sources,
inputs, samples, rejected candidates and reproduction steps are in the study.
The full TypeScript target passes 157 vectors and 32,156 differential mutations,
plus Clippy/MSRV checks and the simulated-endian checks. Seeded fuzzing passes
another 100,000 inputs with address checking (leak checking disabled on this host).
No fresh IC counter/full Candid matrix, broad CI, real big-endian or consumer run.
Only test configuration exposes the Rust endian helpers on little-endian hosts;
production code and generator cost are unchanged, so new timing is not applicable.

## Production contract

Keep one production Serde/Postcard codec through ic_wire::native. Consumers use
Serialize/Deserialize, explicit method IDs and runtime Limits. Wire derives
host-only metadata; it does not generate serializers. Native metadata has no
field IDs and follows Serde declaration order. Coordinate layout changes and
preserve same-release retry bytes. No legacy lane or migration shim is maintained.

Packed<T> is an explicit byte-buffer field for six integer widths and 32-byte
hashes. Existing PackedI16<N> retains its tighter bound and identical bytes.
The [packed guide](../packed-fields.md) owns field selection and migration;
[the runtime contract](../runtime.md) owns admission and serializer restrictions.
Small packed integers can expand payloads; ordinary Vec fields are not optimized
implicitly. The retired finite Wire API and alternate codec strategies are not
production options.

## Comparison and adoption boundary

Use the [current comparison index](../candid-comparison.md) for measured claims.
The [promotion qualification](../packed-promotion-0611.md) preserves prototype
IC counts, allocations, frames and Wasm sizes. The full matrix documents remaining
ordinary-vector and owned-blob costs; these are distinct from packed fields.
The native timing regression and TypeScript tiny-array cost remain qualified
limitations, not claims that packed IC measurements regressed.

[Wire-side Blob qualification](../packed-consolidation-0611.md) covers manifest
hashes, headers, owned roundtrips and limits. The maintainer selected Wire-side
qualification only: siblings remain read-only, with no installed-endpoint claim.
Application bounds/root validation, storage and adoption remain consumer-owned.
Actual consumer workload measurements should guide further optimization.
Canic owns endpoint conversion, IDs, budgets and installed acceptance; external
principals remain on the Candid side. No SDK or platform dependency enters runtime.

## Tooling

Shared Tooling is 0.26.7 at `3e8d7cd`. Current incoming dependency selection includes
Platform 0.6.12 and Testkit 0.66.2. Matched experiments retain their own exact pins;
do not combine measurements from different locks. Candid/PocketIC remain benchmark
dev dependencies. Use focused checks and check for shared-target builds first.
