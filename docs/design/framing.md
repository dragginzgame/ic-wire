# Initial portable framing slice

Scope: method IDs, fixed request-header admission and the reference error
vocabulary. This is an unpublished evaluation package, not the complete codec
or the first end-to-end workload required by the architecture.

## Destination contract and provenance

Reference commit: `2dc434b10bec2fca92345d940dbfedc05993e9d5` in `../ic-wire`.
Source was read with `git show`, never from the moving working tree. Selected
committed files were hashed with SHA-256:

| Source under `crates/ic-wire/src/` | SHA-256 |
| --- | --- |
| `native/mod.rs` | `49f7dd52976426b38949683d7568c47b055e2baf9f32cb3271ce2c4750d0e8a8` |
| `contract/mod.rs` | `b6ad6ae29b28b8a723dbe2bdf260f46f91632619806292e74a77dbf7df63d7d3` |
| `lib.rs` | `54531fcb75c6a69e246722ea5b1487c76dc729981b41b5239960de91b551df0d` |

The initialization's `reference.sha256` file is absent. These selected hashes
record inspected inputs; they do not establish verification of that absent
manifest or the full reference tree. Source licensing is retained at root.

| Old contract | Candidate contract |
| --- | --- |
| `native::method_id(name)` | Root `method_id(name)`, same FNV-1a-64 domain, UTF-8 bytes, constants and wrapping arithmetic |
| `native::validate_request_header(id, &[u8; 11])` | Root function with the same fixed-size input and acceptance |
| Marker/method/direction mismatch returns `Error::Identity` | Same category, including response direction 1 and every other nonzero direction |
| Header check ignores payload and budgets | Same scope; no payload or resource admission claim |
| Five typed errors and their display strings | Same vocabulary; only Identity is emitted by this initial slice |

The implementation retains necessary protocol constants, error vocabulary and
display mapping from those contracts, with a small new framing implementation.
No old codec orchestration, Serde visitor, release system or endpoint is copied.
There are no production dependencies, allocation or unsafe code. No alternative
codec or per-type specialization is introduced. Root API paths are investigational;
consumer migration remains a separate decision. No reference API is removed.

## Focused qualification

Integration tests use literal headers, including zero, maximum and asymmetric
method IDs, plus name-derived `ping`. Every differing byte at each header offset
is checked against `Error::Identity` (2,805 single-byte mutations). Method vectors
cover empty, ASCII, case-sensitive, embedded-NUL and multibyte UTF-8 names.
Expected IDs were calculated independently using Perl Math::BigInt arithmetic:
XOR each byte into the offset basis, multiply by 1099511628211, then reduce modulo
18446744073709551616. Tests do not generate expectations with candidate code.

`make test`, `make lint`, `make doc`, `make check-wasm` and `make fmt-check` are the
focused checks. Wasm compilation proves the target builds, not instruction cost
or a measured size improvement. No native macOS run or performance comparison
is claimed. Full reference/candidate differential qualification remains pending.

## Subsequent implementation

The [admission ledger](admission.md) now captures the reference accounting and
precedence. Bounded decoding is implemented using the frozen Serde/Postcard
cohort; encoding now shares admission and preflight across output destinations.
The full representative workload remains next. The historical
no-production-dependency statement above applies only to the initial framing slice.
