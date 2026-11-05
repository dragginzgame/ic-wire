# Slice-only parser cleanup (0.7.7)

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

The retained change removes the private parser's unused Flavor type parameter and
PhantomData. Wire already constructed only Postcard Slice inputs. Width-specific
integer readers, budgets, public APIs, accepted bytes and error rules remain
unchanged. Removing the unused finite-float macro argument changes no expansion.
This is a maintenance simplification, not a new codec or broad performance claim.

## Matched artifact measurements

| Cohort / profile | Total uncompressed Wasm, before → after | Code section, before → after |
| --- | ---: | ---: |
| Standard / opt-z | 377,801 → 377,801 | 337,772 → 337,772 |
| Standard / opt-3 | 573,464 → 573,444 | 518,754 → 518,734 |
| 1 type / opt-z | 53,998 → 53,998 | 42,997 → 42,997 |
| 1 type / opt-3 | 74,876 → 74,871 | 48,134 → 48,129 |
| 100 types / opt-z | 310,738 → 310,738 | 291,971 → 291,971 |
| 100 types / opt-3 | 398,744 → 398,739 | 364,250 → 364,245 |

All opt-z binaries are byte-identical. Opt-3 total reductions are 0.0035%, 0.0067%
and 0.0013%, respectively. Retired raw output (see retention notice) record each
artifact; these are full linked fixtures, not symbol-attribution estimates.

## IC instructions

The 298-case standard corpus supplies 4,026 matched operation rows across both
profiles and its native/bounded/borrowed field representations. All input hashes,
frame hashes, lengths and repetition counts match. Every opt-z count is unchanged.
At opt-3, 51 decode rows improve and 93 truncation-refusal rows change; encode,
reuse, header-refusal and loop-control counts are unchanged.

| Representative operation, opt-3 | Before → after instructions | Change |
| --- | ---: | ---: |
| Scalar record decode | 1,334.5 → 1,324.5 | −0.75% |
| Empty native blob decode | 398.5 → 398.5 | 0% |
| 1 MiB native blob decode | 1,049,508 → 1,049,508 | 0% |
| Truncated i128-zero refusal (worst regression) | 280.75 → 282.75 | +0.71% |

There are no successful-decode regressions in this corpus. The rejection regression
is retained as a two-instruction tradeoff, not omitted from the result. The
Retired raw output (see retention notice) include every operation and percentage.
Scaling cohorts were size-screened only; their instruction counts were not rerun.
No new Candid, native timing, heap or installed-consumer performance claim is made.
