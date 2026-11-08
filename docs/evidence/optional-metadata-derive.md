# Optional metadata dependency

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Core's existing `typescript` feature now selects the optional `ic-wire-derive`
dependency and its `Wire` re-export. Codec-only consumers need ordinary Serde
implementations; metadata consumers explicitly select `typescript` or its
`platform-types`/Endpoint forwarding features. No second derive switch or codec
is introduced. Serde's own derive remains independently available.

This is a pending Core 0.17 API cut, following the independently qualified
[Endpoint macro ownership move](endpoint-macro-owner.md). The baseline
here already includes that move and the incoming Build 0.74.3 update. Existing
package versions remain Core/derive 0.16.2 and Endpoint/macros 0.6.1. Registry
selections match: Runtime/macros/guard 0.19.1, Types 0.18, facade 0.21,
Testkit 0.84.1, Host/Build 0.74.3. Rust/Cargo 1.99.1, LLVM 23.1.1,
x86_64-unknown-linux-gnu host and wasm32-unknown-unknown guest.

## Results and tradeoffs

Fresh standalone default-library Cargo records contain **one Wire derive
artifact before and zero afterward**, with Core features empty in both builds.
This establishes removal from the codec-only compilation graph. Whole-project
compile time was not sampled repeatedly; no compile-time percentage is claimed.

Both fresh whole Endpoint fixture builds use LTO, one codegen unit, abort and
stripped output, differing only in optimization level. Sizes are uncompressed.

| Profile | Before total / code bytes | After total / code bytes | Change total / code |
| --- | ---: | ---: | ---: |
| opt-z | 100,413 / 86,476 | 100,826 / 86,886 | +413 / +410 (+0.41% / +0.47%) |
| opt-3 | 93,997 / 81,012 | 93,997 / 81,012 | 0 / 0 |

Even the equal-size opt-3 modules have different hashes. Dependency feature
selection can affect compiler output; the precise cause of this layout drift
has not been isolated. This is a build-dependency cleanup with a small opt-z
size regression, **not a Wasm optimization claim**.

The separate two-method counter fixture measures entry/copy/decode/handler and
encoding up to a fixed-width counter field. It excludes final counter
serialization, reply, cleanup, transport and physical cycles. All eleven samples
per row execute with unique callers to prevent query-cache reuse, using the
same `0xa5` byte buffers and unchanged four limits. Unit and empty controls are
included. PocketIC server 16.1.0 SHA-256:
`b44e1eccd66e02328146b3e209e3403e3a3428a25d25555aca2f67500e5ae9db`.

| Input | opt-z before → after instructions | opt-3 before → after instructions |
| --- | ---: | ---: |
| Unit | 245,619 → 245,518 (-0.041%) | 244,270 → 244,270 |
| Empty bytes | 246,165 → 246,020 (-0.059%) | 244,516 → 244,516 |
| 32 bytes | 246,606 → 246,461 (-0.059%) | 244,950 → 244,950 |
| 65,536 bytes | 764,435 → 764,290 (-0.019%) | 762,655 → 762,655 |

These partial endpoint counters are not projected onto complete application
costs or the larger fixture's full endpoint roster. No native codec timing or
Candid matrix rerun is claimed: serialization code and parameters are unchanged.
All 176 samples and Retired raw output (see retention notice) are retained.

## Focused qualification and reproduction

Default/schema Core tests and strict lint, nine metadata diagnostics plus valid
control, standalone host/Wasm feature isolation, golden frames, Principal/Account
Rust–TypeScript bytes and malformed inputs, combined Endpoint/schema forwarding,
Endpoint diagnostics/Wasm and pins/boundaries/docs/format pass. Standalone Cargo
invocations ensure workspace unification cannot hide the default derive selection.
Forwarding metadata to Wasm intentionally still selects its host proc macro for
shared DTO declarations; metadata implementations/JSON remain native-only.

Endpoint's fixture now guards schema assertions with its existing `schema`
feature and checks default tests separately. The counter caller uses a shared
before/after runner and fresh sample identities; this qualification correction is
applied identically to both frozen sources. Final test-only assertion guards and
updated focused check scripts are also archived separately. Production protocol,
golden frame bytes, limits, acceptance rules and retained retry bytes are unchanged.

`qualification.tar.xz` contains frozen sources/locks/toolchain, actual Wasms,
fresh Cargo artifact records, raw samples and focused logs; ignored node_modules
and build targets are excluded. Compression is storage only. Extract and run:

```sh
python3 verify.py /absolute/extracted-directory
```

The following reproduction uses the historical sources: `endpoint-cost` and its
metadata comparison have been retired from the current workspace. Current
endpoint measurements use `make evaluate-endpoints`.

For a rebuild of that historical study, place the extracted `before`/`after` sources in a new study
directory and use `../endpoint-macro-owner/wasm-pair.sh STUDY` with an empty
`STUDY/wasm` and separate source target directories. Rebuild each default library
with `cargo build --locked --offline -p ic-wire --message-format=json`. For IC
counts, build `--example endpoint-cost` instead of `--lib` for both profiles;
copy its guest artifacts to `counter-PROFILE/{before,after}.wasm`, then run the
host example test `tests::metadata_dependency_cost -- --exact --ignored` with
`--features ic`, `ENDPOINT_COST_DIR` and the admitted `POCKET_IC_BIN` set.
The existing host feature/diagnostic scripts regenerate consumer qualification.
No full CI, staging, commits, versions, release or publication was performed.
