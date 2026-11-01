# Upstream adoption, 2026-11-01

Select published **Build/Host 0.73.3** and **Testkit 0.75.2**, including the managed
Build release metadata editor. [Official supplier selections](suppliers.json)
retain registry checksums and Rust requirements. Platform/macros/guard 0.15.3,
canonical Types 0.15.1 and Metrics 0.24.1 are already current and stay selected.
Other direct dependencies retain their latest compatible selections; the external
sha2 0.11 API migration is separate from this supplying IC dependency update.

Refresh all 119 Shared payloads through the canonical exporter at committed
revision `619d194875369e0f72b45477b66adb430acbf6b7`, recorded **0.27.23**.
[Impact review](shared-impact.log) and [refresh](shared-refresh.log) retain exact
selection and ownership. The source's pending standalone-VERSION workflow does
not apply to Cargo release groups. No root VERSION, new release group or manual
snapshot patch is introduced.

The incoming lock already selected these versions and remains byte-identical.
[Preservation](preservation.json) checks that lock, own package versions and
unchanged codec/generator source. The normalized production endpoint
[Wasm graph](normalized-endpoint-wasm-graph.txt) equals the delivered baseline.
Core stays 0.13.2 and Endpoint stays 0.2.1. Pending compatible Core notes select
0.13.3; no Endpoint package change requires new Endpoint notes. No staging,
commits, real releases, publication or sibling checkout edits were performed.

## Supplier changes and executed checks

[Source review](source-review.json) compares released registry src files.
Artifacts/FS/Process are byte-identical. Build adds bounded package-command
evidence capture and consolidates its cache-acquisition state; Wire does not
replace its own runtime with either mechanism. Testkit reuses Host's bounded port
reader in simulator startup, so the installed endpoint consumer is exercised.

[Focused logs](checks/) record:

- Managed Build 0.73.3 editor preparation and offline admission.
- Nine expected diagnostics plus the passing control through Wire's actual
  Build diagnostic caller. Four qualification tests, strict qualification Clippy,
  Rust 1.91 and native/Wasm fuzz/profiler isolation checks.
- Three default Wire/Candid roundtrip/refusal tests and the Rust 1.91 benchmark
  all-target build with wire,candid,ic.
- Actual installed endpoint admission, malformed refusals, transaction and reply
  rollback using the explicitly prepared PocketIC 16.0.0 server.
- A registry-only Rust 1.91 typed/schema consumer with all four native Metrics
  schemas, Principal/Account, Endpoint Contract and Testkit Limits/Error plus typed
  query/update signatures. Exactly one Core/derive/Types/Testkit/Metrics identity
  and the selected Host 0.73.3 suppliers resolve. This separate API/schema test
  does not execute remote typed calls.

Both real Wire group adapters pass their disposable release fixtures with Shared
0.27.23 and Build 0.73.3: dirty/conflicting source refusal, retained failed
validation, exact selected metadata/tag/push, interrupted resume and receipt
rejection. No real checkout delivery effect occurs. Formatting, snapshot integrity,
pins, dependency boundaries, documentation links and runner syntax pass.
No full local CI, new hosted/macOS matrix or fresh codec performance study is
implied by these focused checks. Host execution uses Rust
1.99.1 / LLVM 23.1.1 on Linux x86_64; minimum-version checks use Rust 1.91.0.

## Reproduction and unchanged protocol

[Baseline revision](baseline-revision.txt) plus [input patch](inputs.patch) freeze
the adopted source selections and snapshot. Apply only in a fresh isolated copy;
the patch does not stage, release or upload. Complete temporary outputs remain
under target/adoptions/upstream-20261101. With selected sources, compilers and
tools prepared, the focused commands are:

```sh
make release-editor-check check-typed-diagnostics check-qualification
cargo test --locked --offline -p ic-wire-benchmarks --features wire,candid --test roundtrip
cargo +1.91.0 check --locked --offline -p ic-wire-benchmarks --all-targets --features wire,candid,ic
POCKET_IC_BIN=/absolute/path/to/prepared/pocket-ic make check-endpoint-ic
bash scripts/ci/test-local-release.sh core
bash scripts/ci/test-local-release.sh endpoint
bash docs/evidence/upstream-20261101/qualify-consumer.sh /tmp/wire-upstream-20261101-consumer
make fmt-check shared-tooling-check check-pins check-boundary check-doc-links
```

The consumer runner uses frozen manifest/lock snapshots and reuses the earlier
typed fixture source. It installs nothing and has no Git or publication effects.
Release fixtures confine all Git effects to disposable repositories.

Codec/schema bytes, field order, budgets, malformed-input acceptance, rollback
and original retry/hash inputs are unchanged. Principal remains max-29 bounded
bytes; Account absent `01 04 00` and explicit zero `01 04 01` plus 32 zero bytes
remain distinct. No runtime/generator implementation or production dependency
edge changes, so a new before/after performance benchmark does not apply. No
instruction, Wasm-size or build-time saving is claimed.
