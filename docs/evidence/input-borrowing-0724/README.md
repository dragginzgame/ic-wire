# Borrowed requests through complete IC updates

The existing borrowed decoder is useful at an endpoint boundary: compared with
an owned DTO, it saves 41.2% of complete update instructions for a 1 MiB prefix
read or business rejection, and 14.7–21.4% for a full byte sum. Both profiles also
produce smaller uncompressed Wasm. This is a minimal qualification endpoint,
not the concurrently developed endpoint macro or an installed Canic/RSX adapter.
There is no runtime change, new codec, lazy validation or protocol change.

## Matched measurements

Mean complete guest instructions, including input copy/zero initialization,
decode, handler, destruction, reply encoding, IC APIs and heap-page charges.
Scheduler overhead and host work are separate; these are not wall-clock timings.
Eight physical updates per cohort, varied nonces/payload bytes, counterbalanced
lane order, one excluded startup call per canister/profile. All 384 measured
updates validated complete logical replies. Counts are observations, not confidence
intervals. Per-sample values, ranges and cycle debits are in the JSON evidence.

| Input/work | opt-z owned | opt-z borrowed | Change | opt-3 owned | opt-3 borrowed | Change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Empty/full | 484,540 | 484,470 | −0.014% | 483,820 | 483,805 | −0.003% |
| 32 B/full | 485,529 | 485,021 | −0.105% | 484,613 | 484,145 | −0.097% |
| 32 KiB/full | 1,235,689.875 | 1,042,311 | −15.65% | 997,112.875 | 803,780 | −19.39% |
| 1 MiB/full | 24,500,303.125 | 20,891,106.875 | −14.73% | 16,897,118 | 13,287,967.875 | −21.36% |
| 1 MiB/prefix 256 B | 8,755,139 | 5,146,012 | −41.22% | 8,752,380 | 5,143,299 | −41.24% |
| 1 MiB/business reject | 8,750,826 | 5,141,701 | −41.24% | 8,750,093 | 5,141,012 | −41.25% |

No measured instruction regression occurs in the 12 cohorts under either
profile. This does not establish all input distributions, SDKs or heap layouts.
At 1 MiB prefix/reject, included heap-page charges fall from 5,600,000 to
3,040,000 instructions; do not add those charges again. Allocation/address
history remains relevant: the full-read cohorts include a higher page-charge
sample, retained in the means and ranges rather than discarded.

| Profile | Owned total Wasm | Borrowed total | Change | Owned Code | Borrowed Code | Change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| evaluation (opt-z) | 18,040 | 17,814 | −226 (−1.25%) | 15,784 | 15,561 | −223 (−1.41%) |
| benchmark-speed (opt-3) | 20,161 | 19,796 | −365 (−1.81%) | 18,137 | 17,775 | −362 (−2.00%) |

Separate images contain one identical update export and no schema metadata.
These sizes qualify this fixture, not a large application's total savings.

## Contract and ownership

Both lanes receive the same native frame (method ID 77), containing mode, nonce
and bounded bytes. The owned lane uses `Bytes<1048576>`, the borrowed lane uses
`BytesRef<'_, 1048576>` with Serde borrowing. Both enforce the same byte/node/
depth/allocation allowances and complete-input validation. Owned decoding drops
the raw argument Vec before running business logic. Borrowed decoding retains
that Vec until the synchronous handler returns, then drops it before reply
encoding. Both use caller storage for the same framed response.

Full consumption sums all bytes; prefix mode sums at most 256 bytes; business
rejection consumes none. All modes still decode/validate the complete request.
Borrowing removes the second blob allocation/copy, not the IC argument copy.
Native tests check byte equality, original-input pointers, and matching truncated/
trailing refusals. Allocation counts were not independently instrumented here.
The host computes its reply oracle separately and verifies identical request and
reply hashes across lanes. Trace audits check 96 committed, successful physical
updates per canister, unique sequence IDs, installed module hashes and complete
instruction/page detail with no dropped executions or pagination remainder.

## Reproduction and limits

Source baseline: released 0.7.23, `e90aed4f7dc0cd7ee31a9bcfb211044220ab0cec`.
The runner freezes HEAD's Cargo/workspace/runtime source, then overlays only this
benchmark's manifest and fixture. Run from that revision with the study files
present to reproduce this baseline. It uses a separate target and source copy,
so concurrent endpoint implementation work is excluded. Rust 1.99.1, wasm32,
LTO, one codegen unit, abort panic and stripped symbols; the two workspace
profiles differ in optimization level. Exact lock/source, server and artifact
hashes are in [receipt.json](receipt.json). No Candid comparison is made here.

Select the prepared instrumented PocketIC 16.1.0 binary matching the receipt;
its memory viewer endpoints are required. The ordinary server does not provide
these measurements. IC Testkit owns the private server and its cleanup; curl
reads the existing profiler API. No profiler or SDK runtime dependency is added.

```sh
POCKET_IC_BIN=/absolute/instrumented/pocket-ic-server \
  bash scripts/ci/evaluate-input-borrowing.sh /absolute/fresh-z evaluation
POCKET_IC_BIN=/absolute/instrumented/pocket-ic-server \
  bash scripts/ci/evaluate-input-borrowing.sh /absolute/fresh-speed benchmark-speed
```

The runner retains source, guest images, traces and failed-check logs. Local runs
required sandbox escalation for loopback sockets. The initial denied startup is
retained; it produced no measured calls. Host API wiring was corrected before
measurement; rebuilding opt-z from the final source produced identical guest
bytes. Native tests and Wasm Clippy pass. New MSRV qualification and full CI were
not run. The speed run reused the same exclusively owned dependency build cache.

Committed `*-samples.jsonl` retains each original execution record joined to its
call recipe/hashes; `*.json` retains all statistics and samples. Raw module
metadata, binaries, logs and full source snapshots remain at the receipt's local
evidence paths. No installed SDK, async suspension, retained retries, rollback,
real network ingress authentication or wasm64 equivalence is claimed.

The next product step is a scoped synchronous borrowed-handler integration with
lifetime/refusal tests, measured again through its actual generated entrypoint.
Preserve the owned route for callers that require ownership; those are ownership
choices within the same codec, not alternate production formats.
