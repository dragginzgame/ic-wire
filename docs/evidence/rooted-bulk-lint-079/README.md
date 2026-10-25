# Rooted bulk review qualification

Issue [#25](https://github.com/dragginzgame/ic-wire/issues/25), pending 0.7.9.
Baseline: published 0.7.8, `dece7fccbf98f059994e71adf3b9bcc532057522`.
Candidate: the working-tree source patch retained in [raw evidence](raw.tar.gz).

`lint_bulk_roots` takes explicit consumer-owned roots, registry definitions and
acknowledgements. It traverses references at their use sites, without synthetic
names in paths. For example `/Reply/fields/field_ids/list/u32` is independent of
`/Reply/fields/other_ids/list/u32` even when both use the same Vec definition.
The word suffix makes changing u32 to u64 require fresh review. This is not a
full schema fingerprint: field bounds and custom validators remain owner-reviewed.
Aliases add no path segments; recursive references already active on the current
path stop expansion, while sibling uses are inspected independently. A 64-level,
65,536-visit bound prevents exponential DAG expansion; exhaustion returns a typed
Complexity error, without a partial report. Metadata validation remains separate.

The existing `lint_bulk_fields` API retains its documented definition-once
semantics and paths. Both APIs use one structural walker and candidate detector.
No runtime bytes, encoding, admission budgets or generated client types change.
Rooted consumers must review new paths rather than copy a shared-definition
acknowledgement across every field. Publication and IcyDB adoption are pending.

## Focused verification

35 native tests pass, including five bulk-lint tests. Coverage includes collected
shared containers, unrelated registration order, aliases, removed/changed fields,
recursive enum/record edges, inline result roots, JSON-pointer escaping, excluded
bytes/packed fields, constrained scalar candidates, malformed refs and exponential
DAG refusal. Strict all-target Clippy, doctests and Rust 1.91 host checks pass.
The unchanged TypeScript runtime was not rerun; full CI and fresh macOS checks
were not run for this local batch.

## Host cost controls

Rust 1.99.1 / LLVM 23.1.1, Linux x86_64 Threadripper 7970X, unchanged Cargo.lock,
Cargo release profile, no CPU affinity. Separate target directories isolate source
identities. Each variant warms 100 calls, then takes 11 samples of 1,000 operations;
three runs alternate before/after. Values are medians of 33 batch means, including
allocation/destruction. Schemas contain one or 100 u32-list fields with empty
acknowledgements. No native codec, IC instruction or Wasm size claim is made.

| Host lint operation | Baseline ns | Candidate ns | Change |
| --- | ---: | ---: | ---: |
| Definition lint, one inline field | 718 | 681 | −5.15% |
| Definition lint, 100 inline fields | 49,762 | 49,048 | −1.43% |
| Rooted lint, one interned field | unavailable | 1,186 | new scope |
| Rooted lint, 100 interned fields | unavailable | 58,583 | new scope |

Existing-API per-run medians: one field 724/678/737 → 704/669/667 ns;
100 fields 50,751/49,967/48,549 → 50,076/48,501/49,070 ns. Small changes are
noise, not optimization claims. Rooted traversal costs more than inline definition
review (candidate controls: +505 ns / +9,535 ns), because it resolves every use and
checks expansion bounds. Those are different input graphs and review semantics,
not a matched before/after regression. Published definition lint cannot supply a
correct field-scoped baseline for interned containers.

Raw evidence contains source, benchmark harnesses, samples, validation logs and
replay commands. Extract into a disposable directory and run `bash replay.sh`
with prepared Rust/dependencies. Source qualification remains the native test and
host API checks; these timing controls are not consumer release gates.
