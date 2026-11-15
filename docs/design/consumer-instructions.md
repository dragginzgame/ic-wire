# Installed consumer instruction comparison

Frozen before the first execution: measure the existing sample upload/receipt,
unchanged field representations, IDs and budgets, at both wire-size (opt-z) and
wire-speed (opt-3). Seven separate warmed operations: request encode/decode,
response encode/decode, bad request header, truncated request, and a loop control.
Each query runs one warmup then eight repetitions per operation. Three queries
per artifact use distinct caller identities to avoid simulator query-cache reuse.
Setup, schema generation and reply transport are outside counter intervals;
result destruction, operation dispatch, counter-read and loop overhead remain.
The loop control is reported and never subtracted. Report median batch counts and
all samples from ic0.performance_counter(0), not native timing estimates.

Both suppliers must return the independent 34-byte request and 14-byte reply
literals alongside counts. Before accepting this experiment, all six codec
operation medians must be <= the reference at each profile (zero accepted
instruction regression); loop control is diagnostic only. Failures are recorded,
not averaged away. This is one workload, not complete workload qualification.

A private optional consumer example uses published Platform Runtime 0.22.5 for
safe IC calls and entrypoint macros. No SDK reaches Wire or the default standalone
consumer. Both guests use the same SDK and locked dependency cohort. The native
runner uses published Testkit 0.106.0 to own the explicitly prepared PocketIC
server and instance. Its separate workspace/lock prevents PocketIC's exact
thiserror 2.0.18 requirement from altering the guest's 2.0.21 selection. Candid is
transitive native simulator machinery, not a product codec or research comparator.
This does not recreate a Testkit/Wire adapter or new public executor.

The runner requires POCKET_IC_BIN, refuses a borrowed server URL, and records the
server binary hash/version, artifact hashes, guest/host lock hashes and sources.
Instance/server close is requested after returned errors as well as success;
upstream panic behavior is not a guaranteed cleanup contract. Successful runs
explicitly stop/delete each installed canister. Installation wrappers and SDK
bytes make these artifacts unsuitable for replacing the separate SDK-free size
comparison. Thresholds and selected cases must not be changed after seeing results
without a new documented experiment.

## First installed result — 2026-11-15

[Machine-readable results](consumer-instructions.json) retain all samples,
comparisons and identities. The full `make measure-consumer-ic` command completed
installation/query/cleanup for all artifacts and deliberately returned failure
for the unmet instruction criterion. An earlier direct-driver execution produced
the same counters; all three samples per artifact/operation agree exactly.
PocketIC was the explicitly selected prepared 16.1.0 server; Rust 1.100.0 and both
fixed profiles match the size experiment. All 12 queries returned the exact
independent request/response literals. Returned errors and successful execution
are distinguished from the failed performance criterion.

| Operation | opt-z candidate | opt-z reference | opt-3 candidate | opt-3 reference |
| --- | ---: | ---: | ---: | ---: |
| Request encode | 5224 | 4961 | 2674 | 2443 |
| Request decode | 5390 | 5634 | 3016 | 3294 |
| Response encode | 1725 | 1744 | 696 | 690 |
| Response decode | 590 | 712 | 389 | 425 |
| Bad request header | 189 | 197 | 106 | 132 |
| Truncated request | 4523 | 4779 | 2483 | 2740 |

Values are median eight-operation batch counts divided by eight; counter-read and
loop overhead are included. Request encoding regresses by 263 instructions at
opt-z (5.30%) and 231 at opt-3 (9.46%). Opt-3 response encoding regresses by six
instructions (0.87%). The other nine comparisons are below reference. No averaging
or faster decode result accepts those three regressions.

Next investigation: the generic serialization/admission path and its Wasm inline
boundaries. The reference also contains primitive-specific writers; those are
not automatically imported to obtain a passing number. Any change must preserve
Postcard ownership, bytes, budgets and malformed-input handling, and recheck both
instruction and total Wasm-size criteria. The reference/candidate source reduction
is not by itself permission for an encoding regression.
