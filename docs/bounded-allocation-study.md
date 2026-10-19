# Bounded allocation and borrowing study

Investigated 2026-10-19 against released 0.4.7, commit
`48a016f44ef3c11e7f199c40bf6cdacf09f86b49`. This concerns message codecs;
durable storage and recovery remain outside Wire's scope.

## Selected change: bounded list growth

`List` already reserves an admitted length when the decoder supplies one. When
that hint is absent, it previously requested capacity for one more element at
every exhaustion. Native admission intentionally hides hints to prevent an
untrusted count from allocating before element admission.

The selected implementation doubles requested capacity, capped at the declared
maximum N, using saturating arithmetic and fallible exact reservation. It still
decodes no more than N elements, refuses excess before decoding that element,
and leaves known-length reservation unchanged. Allocation failure behavior,
wire bytes, public signatures and schema maxima are unchanged. Compared with
one-at-a-time growth, spare capacity can remain between length and N; this is
within the existing worst-case finite allocation bound. Requested capacity is
not a promise about an allocator's actual physical allocation.

A native `List<u32, 4096>` decode through the hint-hiding adapter showed:

| Elements | Previous reallocations | Selected reallocations | Previous median ns | Selected median ns |
| ---: | ---: | ---: | ---: | ---: |
| 32 | 31 | 5 | 800 | 336 |
| 1,024 | 1,023 | 10 | 18,494 | 11,712 |
| 4,096 | 4,095 | 12 | 75,619 | 47,880 |

Each nonempty case also makes one initial allocation. The large case reduces
reallocations by 99.7% and native decode time by 36.7%. This is a targeted
hidden-length result, not a general finite decoding or IC instruction claim.
Regression coverage exercises empty/one/odd bounds, overflow refusal, exact
re-encoding, zero-sized values and node admission through the native adapter.

## Rejected default: exact-size finite encoding

An isolated candidate replaces maximum-schema reservation with an exact sizing
pass before serialization. Across all 44 existing allocation cases, frame hashes
and lengths remain identical. Representative opt-z IC encode instruction results:

| Finite workload | Previous | Exact sizing | Change |
| --- | ---: | ---: | ---: |
| Empty byte payload | 41,403 | 1,477 | -96.4% |
| 4 KiB byte payload | 45,641 | 5,734 | -87.4% |
| Empty array manifest | 26,438 | 9,484 | -64.1% |
| 1,024-hash array manifest | 1,979,735 | 2,714,187 | +37.1% |
| 1,024-hash packed manifest | 39,474 | 42,845 | +8.5% |

Empty blob output capacity drops from 1,048,597 to 12 bytes. Native manifest
encode medians regress about 10–19%, with a 12% regression for the full array
manifest. Native empty-byte encoding improves about 13%; 4 KiB encoding regresses
20%. Exact sizing is therefore not selected as the universal default. No runtime
allocation strategy flag or parallel codec mode was introduced. Workload-specific
storage APIs need their own justification; this experiment does not establish an
end-to-end latency win.

## Borrowed finite contracts: compiled feasibility prototype

A private isolated prototype removes `DeserializeOwned` from the `Wire`
supertraits and adds `Deserialize<'de>` requirements at finite decode calls.
A manually implemented bounded borrowed text type and lifetime-parameterized
method marker use the existing `Method` trait; a new generic-associated-type
method framework is unnecessary for this proof.

After adding explicit owned-decode bounds to three generic test/example helpers,
the prototype passes the existing all-feature runtime tests plus a new test for
owned/borrowed frame equality, input pointer identity, canonical rejection,
trailing bytes and field-bound rejection. The borrowed type has zero owned heap
in its shape and retains the text bound in schema metadata. Canonical comparison
still runs: borrowing is not unchecked access.

This is not a shipped API or complete derive implementation. Production adoption
would require lifetime-aware record/enum derives (currently all generics are
rejected), bounded borrowed byte/text types, lifetime escape diagnostics,
TypeScript/schema parity and full tests of nested borrowed collections. Generic
callers that relied on `T: Wire` implying owned deserialization would need explicit
decode bounds; that is a public API hard cut even when bytes remain identical.
The prototype narrows the design risk but does not justify deleting native
admission, whose dynamic/recursive contracts remain distinct. No production
borrowing trait or method API changes are made in this batch.

## Measurement and evidence

Both encoding variants use Rust 1.99.1 / LLVM 23.1.1, opt-z, LTO, one codegen unit,
panic abort. Native measurements run on Linux x86_64, Threadripper 7970X, CPU 8,
without exclusive host ownership. The existing allocation driver takes eleven
rotated calibrated batches. External PocketIC 16.1.0 probes use five queries with
four operations each and `ic0.performance_counter(0)`, excluding fixture setup.
Decode measurements in that driver include its full byte scan and owned drop.
The source/driver protocol is described in the [allocation study](allocation-comparison.md).

The isolated list probe counts allocation/reallocation calls around one native
decode; input construction is excluded. Timing separately takes eleven batches
of 1,000 decode/drop operations, reporting the median. Both variants use the same
probe source and dependency versions. This is requested-allocation instrumentation,
not peak memory; timing batches include counter overhead in both variants.

The compact [measurement table](evidence/hardcut-2026-10-19/allocation.csv) retains
selected exact-size results and every list case. Full raw reports, compiler/lock
identities, baseline and candidate sources, external probes and logs remain in
`target/hardcut-2026-10-19/`. No thousands-line raw JSON dump is added to Git.
Only opt-z was measured in this study; no whole-consumer Wasm-size or deployment
claim is made. Focused runtime tests, strict Clippy, Rust 1.91 host/Wasm and
Rust/TypeScript conformance qualify the selected growth change. Full CI and
native macOS measurements were not run.
