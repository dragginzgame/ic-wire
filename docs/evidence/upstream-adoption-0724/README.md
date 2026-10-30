# Platform, Host and Shared Tooling adoption

Adopt Platform **0.8.3**, IC Tooling **0.70.3** (Build, Testkit and the coordinated
Host family), and Shared Tooling **0.27.9** at
`3c458c28cca132784e6e70fbaf219bdea1e4cf8f`. Manifest minimums and the managed release
editor are aligned. The incoming Wire lockfile already selected the new library
versions and is preserved byte-for-byte. Wire's own package versions stay 0.7.23;
0.7.24 is a pending changelog, not a release performed here.

## Selection and integration

The selected upstream commits were checked against GitHub:
[Platform release](https://github.com/dragginzgame/ic-platform/commit/f69a759bb94e84f054d5f8701461d745b8f8eed4),
[Tooling release](https://github.com/dragginzgame/ic-tooling/commit/c80d061f9030222d4da9ee1ed8b098b76ef5fd46),
[Shared snapshot](https://github.com/dragginzgame/shared-tooling/commit/3c458c28cca132784e6e70fbaf219bdea1e4cf8f).
The local Tooling 0.70.4 draft was not selected. Host is supplied by IC Tooling;
the retired Host checkout is not an independent dependency source.

Shared was refreshed with the canonical exporter after its `--impact` preview.
The 112-file snapshot adds the declared validation-input checkpoint companion;
release metadata admission and governance updates retain their owning suppliers.
Host/Cargo/IC executable catalog pins did not change. No snapshot file was patched
by hand. The release editor was prepared through the standard installer and
subsequently admitted offline.

Wire runtime, derive, endpoint fixture and cost-harness Rust source are identical
to the [pre-adoption matrix](../endpoint-matrix-0724/README.md). Relative to that
frozen baseline, the only lock selections changed are Platform/macros 0.8.2 →
0.8.3 and Build, Testkit, Host Artifacts/FS/Process 0.70.2 → 0.70.3. No other locked
packages changed. Platform's selected CDK and entrypoint-generator source has no
diff between those releases. Protocol bytes, IDs, limits and acceptance rules
are unchanged by this adoption. No consumer or sibling source was edited.

## Before/after results

All **1,760 measured calls / 1,936 physical executions** pass under the adopted
versions, plus 220 excluded warmups. For every one of the 110 cases under both
profiles, all eight instruction, page-charge and cycle-debit samples are identical
to the baseline: **0% change** where the metric is nonzero. Query debit remains
zero; a percentage change from zero is undefined, not a speedup. Caller sequences,
request hashes, reply hashes and success/refusal oracles also match exactly.

Representative absolute results are below (eight-sample means; before = after).
Empty/small controls, boundary cases and all malformed/refusal paths remain in
[evaluation.json](evaluation.json), [benchmark-speed.json](benchmark-speed.json)
and the per-case [before/after comparison](comparison.json).

| Case | opt-z instructions | opt-z cycles | opt-3 instructions | opt-3 cycles |
| --- | ---: | ---: | ---: | ---: |
| echo / blob-0 | 253,966 | 0 | 247,107 | 0 |
| echo / blob-32 | 256,381 | 0 | 249,366 | 0 |
| default_blob / blob-65522 | 3,884,150 | 0 | 3,879,223 | 0 |
| unit-call / unit | 488,803 | 6,984,803 | 484,676 | 6,980,676 |
| borrowed_mutate / reply-32 | 492,869 | 7,070,869 | 487,680 | 7,065,680 |
| composite / self-call | 1,484,952 | 0 | 1,469,820 | 0 |
| async_mutate / reply-0 | 1,245,543 | 18,038,543 | 1,229,259 | 18,022,259 |
| async_mutate / reply-117 | 1,727,846 | 18,520,846 | 1,710,736 | 18,503,736 |
| mutate / oversized | 82,912 | 6,810,912 | 82,914 | 6,810,914 |

| Whole 17-endpoint fixture | Before total Wasm | After total Wasm | Total change | Before code | After code |
| --- | ---: | ---: | ---: | ---: | ---: |
| opt-z | 103,638 | 103,642 | +4 bytes (+0.0039%) | 89,112 | 89,112 |
| opt-3 | 98,503 | 98,503 | 0 bytes (0%) | 84,974 | 84,974 |

The opt-z four-byte total-size increase is retained as a regression; code-section
size and measured work are unchanged. Artifact hashes differ, so cost qualification
was repeated rather than inferred solely from source similarity. These are whole
fixture sizes, not per-endpoint attribution. No gzip objective or savings claim.

## Reproduction and limits

The [receipt](receipt.json) records source/lock identities, raw before/after
evidence paths, compiler, server and artifact hashes. Both sides use Rust 1.99.1,
wasm32, the existing opt-z/opt-3 profiles, LTO, one codegen unit, abort panics and
stripped symbols. Input frames, caller sequence, case order, limits and server
binary are identical. The two profiles are separate measurements, not pooled
samples. These are complete physical guest instructions and observed cycle
debits, not host/native timing or production throughput.

```sh
POCKET_IC_BIN=/absolute/prepared/instrumented/pocket-ic-server \
  make evaluate-endpoints EVIDENCE_DIR=/absolute/new/adoption-matrix
```

Use the prepared instrumented PocketIC 16.1.0 binary identified by the receipt;
IC Testkit owns its private lifecycle. Retain the original frozen baseline for
before/after comparisons. The normal PocketIC binary separately qualifies the
installed endpoint transactions. Local socket tests required sandbox approval.
No engine rebuild or server upgrade is mixed into the dependency comparison.

Focused checks cover snapshot integrity, tool admission, dependency pins and
boundaries, native endpoint/schema tests, compiler diagnostics, strict Clippy,
Wasm and Rust 1.91, installed sync/async/borrowed/guarded transactions, and the
complete endpoint cost matrix. Disposable Make/hook and release adapter fixtures
pass, including failed-validation retention and resume/refusal behavior. They
use temporary Git repositories and a temporary bare remote, never real-checkout
release effects. Full CI, native macOS and the broad Candid codec matrix were not
rerun. No commit, push, release, publication or deployment was performed.
