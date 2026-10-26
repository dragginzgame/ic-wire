# Current handoff

## Released 0.7.16; pending 0.7.17 tooling adoption

Baseline is `27304e56004d2bc42aa94de3b655b6634aa55b37` (released 0.7.16).
The earlier documentation batch applied the task-specific workflow from
[PR #27](https://github.com/dragginzgame/ic-wire/pull/27) locally. The current
adoption selects Shared Tooling 0.26.20 at
`a7f85d45d4212d4c39225f58597580560a333550` through the canonical exporter.
Its compact baseline now owns those workflow rules, so the temporary local
exception and redundant sibling wording are removed.

IC Build/Testkit minimums and the managed release editor select 0.68.7;
Platform selects 0.6.18. The incoming lockfile already selected these versions
and is preserved byte-for-byte. Runtime/generator code, protocol bytes,
admission rules and Wire package versions remain unchanged. No staging, commit,
merge, push, publication or release was performed.
PR #27 remains a separate hosted branch; local validation does not establish
that its complete hosted CI matrix has passed.

## Runtime and qualification

There is one production Serde/Postcard codec. Released 0.7.16 retains explicit
three-group u16 parsing, also used for i16 zigzag decoding. Encoding, shortest
representations, overflow rejection, resource admission and typed errors remain
unchanged. No consumer migration is required for that optimization.

The [runtime study](../evidence/runtime-0716/README.md) owns its measured IC,
native-time and Wasm tradeoffs, including small-input regressions. It uses frozen
Build/Testkit/host 0.68.5 and Platform 0.6.16. The later
[encoder study](../evidence/encoder-followup-0716/README.md) uses 0.68.6/0.6.17
in both variants; those results are not interchangeable dependency qualification.
All encoder candidates remain research, preserving the production two-pass
admission/sizing contract. Later temporary experiments retained no runtime change.

The released 0.7.16 [CI matrix](https://github.com/dragginzgame/ic-wire/actions/runs/40307479761)
passed Ubuntu, macOS ARM and macOS Intel. Issues #20–#26 were closed after
checking their implementations, published source and that complete matrix.
Consumer rollout and installed endpoint acceptance remain consumer-owned.

## Current validation and boundaries

Focused Linux checks pass: snapshot integrity, documentation links, dependency
pins/boundaries, actual Make/hook routing, release-adapter recovery, compiler
diagnostics, benchmark corpus roundtrips and strict Clippy for the changed
IC Build and Testkit consumers. The 0.68.7 editor is installed and admitted.
The existing PocketIC admission probe passes 144 cases with three samples each,
including the updated owned startup/cleanup lifecycle; local evidence remains
under `target/tooling-adoption.jpvRC7/results`.
Full CI, the complete Candid performance matrix and fresh macOS execution were
not run; no runtime performance improvement is claimed.

Consumers choose byte buffers/Packed for bulk fields and bind schemas to deployed
identities. Preserve original frames for exact retries and signatures; collection
normalization remains consumer-owned. See the [runtime contract](../runtime.md).
