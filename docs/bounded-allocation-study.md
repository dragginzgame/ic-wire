# Bounded allocation and borrowing study

Historical 0.4.7/0.4.8 experiments. The borrowed finite contract API is now
implemented in the 0.5.0 release; see the
[migration guide](../crates/ic-wire/README.md#borrowed-bounded-contracts-and-the-05-api-cut).

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

## Follow-up: where a hard cut would help

A second investigation starts at released 0.4.8, commit
`a5f3cd66f2afb9650b7028944e4d88ebab04c967`. A concrete API hard cut would remove
the owned-deserialization requirement from `Wire`, place lifetime-aware decode
bounds on the consuming functions, and add bounded borrowed byte/text types with
lifetime-aware derives. Generic callers relying on the old implied bound must
change. This would require a pre-1.0 minor release (0.5.0 from this baseline),
even if explicit identities and encoded bytes stay unchanged. This is still an
experimental API direction; native dynamic admission remains useful.

The measured saving instead comes from a compatible canonical-comparison change.
A borrowed value can serialize the exact slice being compared. After checking
allowance and matching slice length, identical pointers prove byte equality
without scanning that region again. A different address still takes the existing
exact comparison path. Previous mismatches remain failures; trailing bytes,
length limits and noncanonical scalar/length representations remain checked.
No canonical-validation bypass or new decoder mode is introduced.

The selected shortcut applies only on Wasm (and native unit tests for coverage).
An initial all-target candidate produced substantial native timing regressions,
including a 1 MiB borrowed case from 260,839 to 584,545 ns. That candidate is
rejected. With the shortcut restricted to Wasm, the native benchmark executable's
entire `.text` section is byte-identical to baseline. A repeat native run puts
large byte cases within about 3%; no native speedup is claimed.

| Borrowed decode + full scan | opt-z before | opt-z selected | opt-3 before | opt-3 selected |
| --- | ---: | ---: | ---: | ---: |
| 32 KiB packed manifest | 581,074 | 511,251 | 309,764 | 256,397 |
| 4 KiB opaque bytes | 71,323 | 62,549 | 39,231 | 32,518 |
| 1 MiB opaque bytes | 17,958,169 | 15,729,875 | 9,831,299 | 8,127,306 |

Units are IC instructions per operation. Savings are about 12% at opt-z and
17% at opt-3. Owned decoding pays a small branch cost: empty finite bytes increase
from 399 to 403 instructions at opt-3 (1%); large owned byte cases increase by
5–12 instructions. The combined opt-z probe grows from 108,170 to 108,172 linked
Wasm bytes, while gzip-9 stays 33,308. Opt-3 shrinks from 119,762 to 119,746 linked
bytes and 37,580 to 37,578 gzip bytes. These are whole-probe measurements, not a
consumer deployment qualification or individual symbol-size attribution.

For the potential hard cut, a separate manually implemented bounded borrowed
byte type uses the finite decoder plus the selected shortcut. Its 1 MiB opt-z
case costs 15,729,727.5 instructions, only 147 fewer than native borrowing with
the shortcut (0.00094%). Empty bytes fall from 907.5 to 760.5 instructions,
16.2% or 147 instructions. This supports investigating more expressive bounded
borrowed DTOs, but does not justify a trait redesign for bulk speed alone. It
avoids an owned payload copy, as existing native borrowing already does. Borrowed
values must still keep their backing frame alive.

All 44 frame hashes agree across the original, selected and experimental finite
builds. The finite prototype passes three fixture tests, including field bounds,
input pointer identity, method identity, trailing and noncanonical lengths. It
has no production derive or public borrowed type implementation. Its measured
source differs from the tested source only by added test-only code.

Measurements reuse the compiler, profiles, native calibration and external IC
protocol described above. Both IC profiles were rerun after restricting the
shortcut to Wasm, with exactly identical instruction samples. Raw results,
source/lock hashes, original rejected candidate, Wasms and source archives are
retained under `target/borrow-savings-2026-10-19/`. Only the compact
[summary](evidence/borrow-savings-2026-10-19/summary.csv) is checked in. This batch
changes no workspace dependencies, framing, public APIs or canonical rules.

Runtime tests, strict Clippy, Rust 1.91 host/Wasm and 90 TypeScript vectors plus
17,524 differential mutations pass. Unit checks cover equal separate buffers,
identical slices, every differing byte, exhausted allowance, shorter input,
trailing input and an already-failed comparison. Full CI and native macOS were
not run. A breaking API release is not selected by this investigation.
