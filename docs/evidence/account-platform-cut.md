# Platform 0.19 / Types 0.18 adoption qualification

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

**Activated in the authorized Core 0.16 / Endpoint 0.6 release checkout.**
Testkit 0.81 is now published. Its normal client factory passes strict all-target
lint and installed admission/rollback checks, and both final endpoint guests
match the measured binaries byte for byte. The final resolved workspace has one
selected Platform family; Retired raw output (see retention notice)
are separate from the earlier managed-server experiment. The original dirty
checkout is preserved. Platform's source and
published archives supply Runtime/macros/exact guard **0.19.0**, canonical Types
**0.18.0**, and facade/PocketIC **0.19.0**. The upstream release receipt is
[d9af0f6](https://github.com/dragginzgame/ic-platform/commit/d9af0f6690459949188f904502db4677fe82d58b).
The original preparation was blocked on Testkit 0.80.2, whose factory accepts
the old builder. The published 0.81 release removes that blocker.

## Prepared implementation

The Retired raw output (see retention notice) applies to this checkout's current runtime
sources. Its root Cargo changes select Runtime 0.19, Types 0.18 and facade 0.19;
the historical patch omits the final Testkit 0.81 dependency/lock reconciliation,
which is now selected in the release checkout. There are no supplier patches, wrappers,
new packages or second schema trait. Own package version preparation and registry publication belong to the explicit
Core 0.16 / Endpoint 0.6 release transactions.

Core's Account metadata describes optional exact 32-byte buffers, matching the
published standard `serde_bytes` field adapter. Principal stays bytes(max 29),
and Account order remains owner then subaccount. For an anonymous owner, None
remains `01 04 00`; explicit zero changes from `01 04 01 + 32 zeros` to
`01 04 01 20 + 32 zeros`. Browser fields use Uint8Array. Bare Subaccount arrays
outside Account keep their existing representation.

This is an incompatible Account binary Serde change. The regression reproduces
old subaccount `[20, 00 × 31]` followed by blob `[02, aa, bb]` decoding under an
unchanged key as subaccount `[00 × 31, 03]` and blob `[aa, bb]`. A fresh key refuses
both request/reply headers even with zero payload-node allowance. The private
combined consumer replaces `echo_account` with `echo_account_bytes`, obtaining a
fresh current name-derived guard, without retaining an old route. Real consumers
must coordinate keys, client regeneration and all four budgets; retain original
retry/signature/hash bytes and inventory unframed stored data separately.
External ledger Candid DTOs and logical Account equality remain unchanged.

PocketIC setup/observations now handle ClientError explicitly. Negative installed
tests and cost samples require `ClientError::Canister` with the expected reject
code; a transport/protocol failure cannot count as successful refusal. No new
retry policy is installed. Automatic contract fingerprints and removal of macro
IDs remain a separate prepared experiment; they are not activated here.

## Published-supplier measurements

`/tmp/ic-wire-platform019-20261103/before` freezes main aa308c9 plus the existing
dirty lock (Build 0.74.1, Host 0.74.2, Testkit 0.80.2). `candidate` contains the
prepared source changes. `managed-qualification` differs only in unpublished
host harness bootstrap: Testkit's public `start_managed_server` admits/owns the
server, then the published 0.19 client connects to its explicit URL. The client
drops before the server. This bridge is **not in the adoption patch** and does
not qualify the still-unpublished Testkit 0.81 factory.

Rust **1.99.1**, wasm32, opt-z/opt-3, fat LTO, one codegen unit, abort and stripped
symbols match in each pair. Candid **0.10.39**, Postcard **1.1.3**, Serde
**1.0.229** and serde_bytes **0.11.19** are unchanged. Each source uses independent
Cargo targets. Freshly built managed-harness guests match all four measured
Account/Candid artifacts byte for byte. Inputs, frame outputs, baked producer
hashes, module hashes and actual physical trace module identities are checked.
The instrumented server is **16.1.0**, SHA-256
`b930e8bb2d5a1dbed7281a6a8717565c80576fca4c857b5b0a2cabcfc56842ff`.

| Linked guest / profile | Total uncompressed Wasm, before → after | Code section, before → after |
| --- | ---: | ---: |
| 17 endpoints / opt-z | 100,637 → 100,413 | 86,698 → 86,476 |
| 17 endpoints / opt-3 | 93,997 → 93,997 | 81,012 → 81,012 |
| 58-case Wire / opt-z | 226,987 → 224,725 | 193,245 → 190,983 |
| 58-case Wire / opt-3 | 297,355 → 294,980 | 265,780 → 263,405 |
| 58-case Candid / opt-z | 688,646 → 688,283 | 629,202 → 628,831 |
| 58-case Candid / opt-3 | 996,156 → 996,929 | 938,605 → 939,369 |

Do not add savings from different linked guests. The Candid opt-3 module grows
773 bytes; these are whole-workload supplier changes, not symbol attribution or
application-size claims. Compressed size is not used.

All **356 endpoint case/profile pairs** retain request/reply hashes, outcomes and
physical-execution counts across **5,696 compared measured calls** (one excluded
warmup plus eight samples per case/profile/lane). No instruction/cycle/page-cost
regression occurs. Opt-3 metrics are unchanged; opt-z reductions are 4–28
instructions on affected cases. Their cause is not attributed to a specific
source change. Retired raw output (see retention notice).

The complete 58-case contract-shape cohort yields **1,230 operation pairs**, three
samples each. The 23 Account/map/set cases win 85/92 primary encode/decode
comparisons against Candid after adoption. At opt-3, 128 explicit-zero Accounts:

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Wire encode instructions | 482,927.25 | 122,726.25 | −74.59% |
| Wire decode instructions | 569,479.5 | 104,711.5 | −81.61% |
| Wire frame bytes | 8,077 | 8,205 | +128 |

Candid encode is 104,255.5 instructions, so Wire remains 1.177× for this case;
Candid decode is 1,636,131.5. Preserve small-input regressions: absent scalar
Account encode adds **12 instructions (+1.24%)** at opt-3; empty sparse 8/32/128
record encode adds **9 instructions** at opt-z (at most +0.265%). Three related
rejection rows also increase; Retired raw output (see retention notice) retain them. Eighteen
Candid opt-z decode rows improve by at most 0.532%; opt-3 counts and all Candid
frames match. No universal unchanged-Candid or general native-time claim is made.
Retired raw output (see retention notice).

## Checks and remaining adoption gate

- Core all-feature native library/tests: 86 pass. Standalone schema-only/combined
  consumers: 6/7 tests, strict Clippy, Rust 1.99.1 host/Wasm, default-Core isolation,
  bounded Principal and exact Account bytes/refusals pass.
- Current and simulated-next facade/Endpoint consumers retain one canonical Types
  and Runtime guard identity. Native endpoint fixture: 3 tests; compiler
  diagnostics: 11 expected failures plus passing control; strict library lint pass.
- Default and focused 58-case benchmark roundtrip suites pass six tests each;
  shape-specific guest tests pass three. Temporary managed-client all-target lint,
  installed malformed/limits/borrowed/async/rollback checks and both endpoint cost
  profiles pass.
- Published Testkit 0.81 factory compilation/lint, installed checks, graph
  reconciliation and both exact fresh-guest comparisons now pass. Complete release
  gates, final own-version/package artifacts and registry-only installed Wire
  consumers remain separate delivery evidence.
