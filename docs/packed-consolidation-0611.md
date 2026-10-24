# Packed consolidation and consumer-boundary review

This follow-up retains a Blob-shaped boundary test and reorganizes comparison
navigation. The production runtime and protocol are unchanged. Three attempted
internal consolidations remain rejected experiments, not supported modes.

## Internal sharing screen

Both public field APIs remain: Packed<T> uses runtime limits, while PackedI16<N>
also enforces its own element count before allocation. The experiment first shared
serialization and a generic visitor carrying a runtime limit, then retained the
existing bounded visitor and shared only serialization, with/without an inline hint.
Neither variant drops the bounded field check or changes the intended bytes.

| 100-record cohort, opt-z | Baseline total / code bytes | Shared visitor | Shared serializer | Inline shared serializer |
| --- | ---: | ---: | ---: | ---: |
| Generic packed fields | 460,922 / 441,443 | unchanged | unchanged | unchanged |
| Varied bounded i16 fields | 288,122 / 268,601 | 288,151 / 268,630 | 288,126 / 268,605 | 288,126 / 268,605 |

The shared visitor adds 29 total/code bytes (+0.0101% total); serialization-only
adds four (+0.0014%). Generic-field artifacts are byte-identical to baseline.
These are tiny regressions, not correctness defects. They establish no Wasm saving,
and introduce another shared internal interface. The existing implementations
are retained rather than broadening qualification for a cleanup with no measured
artifact benefit. This is not evidence that all future internal sharing is futile.

The baseline and each candidate pass all 15 existing native admission tests,
including bounded/unbounded i16 equivalence, malformed widths and field limits.
This was an early Wasm screen, not a full optimization qualification: native timing,
IC instruction counts and opt-3 were not measured, and no speed improvement is
claimed. Production code is byte-for-byte unchanged by this follow-up, so there
is no new production runtime cost requiring a before/after timing claim.

## Reproduction and identity

Baseline is released `31bcef2` plus the incoming 0.6.11 promotion working tree.
Both sides use the same Cargo.lock, Rust 1.99.1, wasm32-unknown-unknown and the
`evaluation` profile: opt-z, LTO, one codegen unit, panic abort, stripped symbols.
The existing bulk-scale and packed-scale fixtures retain identical inputs and
codec settings; no Candid or gzip size result is substituted for Wire size.

[Wasm CSV](evidence/packed-consolidation-0611/wasm.csv) records absolute sizes and
hashes. The [archive](evidence/packed-consolidation-0611/raw.json.gz) retains baseline
source, candidate source overlays, compiler identity, correctness logs and runner.
Restore baseline files into separate before/candidate trees, apply one overlay,
then run the archived script with distinct target directories. The code-section
reader is the existing codec-report example; build it in the isolated tree if
the runner's recorded local binary path is unavailable. Earlier candidate overlays
are reconstructed from the tested transformations; formatting may differ.

## Wire-side consumer qualification

Read-only inspection of Blob Storage `dcbb8811` identifies its upload manifest as
a relevant hash-list boundary. The new native test mirrors chunks followed by
exact header names/values. It checks 0/1/4/1,024 hashes, raw suffix equality,
owned roundtrip, frame-limit rejection and host metadata for packed hashes plus
header records. The empty/small cases are representation controls, not claims of
valid application manifests. Application domain validation is deliberately not
reimplemented in Wire.

The [consumer guide](packed-fields.md#consumer-boundary-example-blob-storage)
records request/response coordination, retained intent handling, chunk/header
bounds and preservation of the distinct Candid storage record. Wire-side tests
are not installed endpoint acceptance, and synthetic fixture Wasm sizes are not
an installed consumer measurement. No sibling checkout was modified or built.

The inspected declaration contains hashes and metadata, not whole content bytes.
There is therefore no evidence from this boundary that optimizing the owned-1-MiB
blob decoder is the next consumer win. Canonical checks remain intact. Manifest
adoption and real application measurements are the next consumer-owned step.

## Documentation

The current comparison page is now a short index of maintained results and
tradeoffs. The previous report is preserved under docs/history, and inbound
historical section links are redirected. Evaluation clearly marks its historical
sections. No evidence or reproduction boundary was deleted.
