# Current handoff

## Released 0.7.12; pending 0.7.13

Baseline is `974b6ff3e4a89c37a6c93081d7a26bee2e709f7b`; the checkout started clean.
This batch fixes duplicate numeric method identities in host registry generation.
Own versions, staging, commits, publication and releases remain maintainer-owned.
A concurrent tooling lock update to 0.68.5 is preserved outside this fix; the
benchmark uses identical isolated locks and does not qualify that update.

## Numeric method identities

Previously distinct names could reuse one ID. A request payload ff admitted as
u8=255 under ID 7 also decoded as i8=-1 under ID 7: the frame carries an identity
and direction, not a payload schema or method name. `generate_registry` now
rejects repeated parsed u64 IDs across names and query/update modes, including
spellings such as `7`/`07`/`+7`. It returns Schema without returning source or
emitting diagnostics. Conflicting consumers must assign distinct identities and
coordinate clients. Unique zero/u64::MAX identities remain valid.

The native regression demonstrates reinterpretation with the same ID and refusal
with a different ID. It fails against the old generator and passes with the fix.
The browser fixture itself contained an ID-1 collision; its second method now uses
ID 2. Added generated u8/i8 request and response checks use IDs 7/8 and reject
cross-method frames. Existing valid generated output and all runtime bytes and
admission rules remain unchanged. This tightens host metadata acceptance only.

## Qualification and cost

All 39 native tests pass on Rust 1.99.1 and 1.91. The TypeScript gate passes
165 vectors and 35,228 mutations, plus validation, diagnostics,
exact frames, browser mutations, strict Clippy and native/Wasm feature boundaries.
The matched host benchmark uses identical compiler/dependencies/settings and
three paired runs, with small controls and 33 samples per cell. For 8/128/1,024
methods, pooled medians change 9.36→9.40 / 144.11→147.34 / 1,254.78→1,252.15 µs.
Controls drift by up to 4.5%; no speedup is claimed. All valid generated sources
are byte-identical across variants. No IC-instruction or runtime Wasm-size change
is claimed. See [raw results and reproduction](../evidence/method-ids-0713/README.md).
Full CI and installed consumer suites were not run.

## Consumer boundaries and retained work

Uniqueness is enforced within one generator call, not across independently built
registries or direct codec calls. Field order/schema changes under the same ID
remain a coordinated-deployment concern. Consumers bind clients to the deployed
schema and retain original frames for signatures and exact retries; collection
normalization may still change re-encoded bytes. See the [runtime contract](../runtime.md).

Canic generator adoption, Toko's installed Wire path and reported application
Wasm headroom were not requalified by this fix. Their earlier findings are not
claims about their current working trees. The released [unsigned evidence guide](../unsigned-magnitude.md),
[budget study](../evidence/tail-and-budgets-0712/README.md) and
[full comparison](../full-comparison-0710.md) retain their own qualification scopes.
