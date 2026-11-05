# Borrowed requests through complete IC updates

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

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
