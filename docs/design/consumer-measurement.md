# Standalone consumer Wasm comparison

Frozen before the first measurement: the existing sample-upload workload in
qualification/consumer runs request encode/decode and response encode/decode, with
black_box and value assertions retained identically for both suppliers. It uses
the public contract, fixed field representations, method ID and budgets already
qualified by literal vectors. There is no SDK, canister transport or Candid lane.
The executable includes std startup, allocator, assertions and consumer glue;
total uncompressed module bytes are the primary metric, not codec-only estimates.

Candidate 0.23.7 and published reference 0.22.4 use the same Rust 1.100.0 compiler,
locked dependencies and wasm32-unknown-unknown target. Separate fresh build
folders prevent feature/profile mixing. wire-size uses opt-z, full LTO, one codegen
unit, panic=abort and symbol stripping; wire-speed changes only optimization to 3.
These match the pinned reference's evaluation/benchmark-speed profile settings.
No Binaryen pass or compression is applied. Host schema/derive features are off.

Acceptance for this bounded comparison: exact contract bytes/acceptance remain
unchanged; all four artifacts must validate and execute without host imports;
candidate total bytes must not exceed reference for either profile (zero accepted
size regression). Report code-section payload and other bytes separately. A pass
qualifies only this consumer/build comparison. It is not evidence for all workloads
or IC execution cost; IC instructions and allocation remain separate metrics.

`make measure-consumer-wasm` builds and executes all four artifacts, records
compiler/Node versions, base commit, selected source and lock hashes, artifact
hashes and section sizes. Environment flag overrides are refused. Reports and
artifacts are retained in a uniquely named directory under qualification/target.
Timing of Node execution is deliberately not reported as IC instruction evidence.

## Measured result — 2026-11-15

All four modules validated and executed their retained roundtrip assertions under
Node 24.21.0, with zero host imports. Rust was 1.100.0/LLVM 23.1.5 on Linux x86_64.
The selected source/lock and artifact hashes are in the [final report](consumer-measurement.json)
and [source manifest](consumer-measurement-inputs.json). The [baseline report](consumer-measurement-baseline.json)
and [baseline source manifest](consumer-measurement-baseline-inputs.json) preserve
the original regression. Reference artifact hashes were identical in every run.

| Profile | Candidate total | Reference total | Difference | Candidate code payload | Reference code payload |
| --- | ---: | ---: | ---: | ---: | ---: |
| opt-z | 45,279 | 46,138 | -859 (-1.86%) | 38,320 | 39,180 |
| opt-3 | 46,119 | 47,026 | -907 (-1.93%) | 39,517 | 40,426 |

All sizes are uncompressed bytes. Other bytes include data, tables, module/section
headers and any retained custom sections; their exact counts are in the report.
Payload frames remain the independently qualified 34-byte request and 14-byte reply.

The original candidate measured 44,127 bytes at opt-z and 47,770 at opt-3, failing
the latter criterion by 744 bytes. Combining meter checks/commits reduced those to
44,117/47,549. Using the reference's Wasm-only inline policy for the three small
meter operations produced the final 45,279/46,119 above. This gives back 1,152 bytes
of the original opt-z advantage to eliminate the opt-3 loss. No threshold, workload,
byte representation or budget changed between runs. There is no claim that the
final choice minimizes every profile or improves IC instruction counts.

The meter now reads both balances, checks refusal/depth/node/heap conditions, then
commits both debits. Failed visits still debit neither allowance and make refusal
sticky; direct buffer charges retain their existing behavior after refusal. This
is a local optimization of the existing admission owner, with no new codec or
per-type specialization. The inline policy is narrowly reused from pinned
reference native/mod.rs (the design reference revision and retained license apply).
Eight native differential tests (including allowance sweeps/mutated frames), the
sticky-refusal tests and eight allocation groups pass; Clippy also passes.

This closes the size criterion for this one standalone consumer at both profiles.
IC instructions, broader workload coverage, browser-engine execution and installed
package proof remain outstanding. Host schema single-root timing regressions are
independent and remain recorded in the host measurement contract.
