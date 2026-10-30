# Synchronous borrowing through generated endpoints

`#[ic_wire::endpoint::update(borrow, ...)]` reduces measured 1 MiB prefix-read and
business-rejection costs by 46.8% relative to the equivalent owned endpoint.
Full scans improve 15.4–22.8%. Both profiles produce slightly smaller Wasm.
Platform still copies ingress once; borrowing avoids the subsequent blob copy.
The codec, frame bytes, complete-input validation and limits are unchanged.

This follows the [initial raw-export probe](../input-borrowing-0724/README.md).
Here both artifacts use the actual pending Wire/Platform endpoint generator and
bounded owned response encoder. The borrowed wrapper retains the raw argument
vector through reply encoding; the owned wrapper releases it after decoding.
These are standalone fixtures, not an installed Canic application.

## Measured results

Mean complete guest instructions (not native time or wall-clock latency), including
input copying, decode, handler, response encoding, destruction, IC API and heap-page
charges. Eight physical calls per cohort, counterbalanced lane order and varying
nonce/body bytes; empty, 32 B, 32 KiB and 1 MiB controls. One startup call per lane
is excluded. All 384 measured updates across the two profiles passed independent
reply oracles and matching request/reply hash checks. JSON retains each sample,
min/max, page charges and observed cycle debit; these ranges are not confidence
intervals. Host work and scheduler overhead are not included.

| Input/work | opt-z owned | opt-z borrowed | Change | opt-3 owned | opt-3 borrowed | Change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 B / full | 487,480.000 | 487,350.000 | -0.027% | 484,905.000 | 484,971.000 | +0.014% |
| 32 B / full | 488,556.000 | 487,869.000 | -0.141% | 485,630.000 | 485,279.000 | -0.072% |
| 32768 B / full | 1,206,012.875 | 1,012,542.000 | -16.042% | 965,296.875 | 772,152.000 | -20.009% |
| 1048576 B / full | 23,454,818.125 | 19,845,529.875 | -15.388% | 15,849,494.000 | 12,240,531.875 | -22.770% |
| 1048576 B / prefix256 | 7,709,654.000 | 4,100,435.000 | -46.814% | 7,704,754.000 | 4,095,861.000 | -46.840% |
| 1048576 B / reject | 7,705,307.000 | 4,096,090.000 | -46.841% | 7,702,576.000 | 4,093,687.000 | -46.853% |

Empty opt-3 inputs regress by 66 instructions (+0.014%) in each work mode; all
other measured cohorts improve. This fixture does not establish every workload.
Included 1 MiB prefix/rejection page charges fall from 5,600,000 to 3,040,000;
do not add them again. Full scans include a higher page-charge sample, preserved
in means and ranges. Business rejection is an ordinary successful reply that
consumes none of the body; it is distinct from malformed-input trapping.

| Profile | Owned total Wasm | Borrowed total | Change | Owned code | Borrowed code | Change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| evaluation (opt-z) | 31,344 | 31,000 | −344 (−1.10%) | 27,589 | 27,247 | −342 (−1.24%) |
| benchmark-speed (opt-3) | 33,550 | 33,349 | −201 (−0.60%) | 29,772 | 29,571 | −201 (−0.68%) |

Sizes are total uncompressed Wasm and code-section bytes, one export per image.
The owned probe rebuilt against the pre-integration snapshot is byte-identical
to the owned probe after integration in both profiles; hashes are in the receipt.
This establishes no code/instruction regression for that owned fixture, not all
applications. Allocation counts and host macro-generation/native timings were
not measured. No Candid comparison or gzip objective is claimed.

## Reproduce and audit

The [receipt](receipt.json) identifies compiler, lock, source files, original owned
artifact hashes, instrumented server and raw evidence. Rust 1.99.1 / wasm32,
Platform 0.8.2, Testkit 0.70.2; existing profiles use LTO, one codegen unit, abort
panics and stripped symbols. The runner freezes the working tree before building
both ownership choices with identical inputs and dependencies. It adds no runtime
profiler dependency and delegates private server lifecycle to Testkit.

```sh
POCKET_IC_BIN=/absolute/prepared/instrumented/pocket-ic-server \
  bash scripts/ci/evaluate-input-borrowing.sh /absolute/fresh-z evaluation endpoint
POCKET_IC_BIN=/absolute/prepared/instrumented/pocket-ic-server \
  bash scripts/ci/evaluate-input-borrowing.sh /absolute/fresh-speed benchmark-speed endpoint
```

Use the instrumented PocketIC 16.1.0 server hash in the receipt; the standard
server lacks its memory-viewer API. Loopback sockets required local sandbox
approval. The runner keeps source/images/raw traces and validates module hashes,
unique physical sequence IDs, committed replies and complete instruction/page
records with no truncation, dropped executions or unconsumed pagination. Re-run
`node scripts/ci/summarize-input-borrowing.mjs RAW_DIRECTORY` to audit the retained
raw calls/traces. Compact per-call receipts and cohort summaries are retained here.

Separately, `make check-endpoint` passes native storage-pointer/schema checks,
valid borrowed direct/DTO signatures, lifetime-escape and async/unit borrowing
refusals, Clippy, Wasm, Rust 1.91 and dependency-boundary checks. The installed
fixture verifies borrowed replies, malformed/truncated/trailing/wrong-ID/oversized
refusal before mutation and synchronous rollback on reply overflow. Existing
owned, unit, async-await and guarded-composition cases still pass. Async borrowing
is deliberately unsupported; no new executor or codec is introduced.
