# Endpoint generator cleanup

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Borrowed handlers no longer construct the encoder identifier, encoder-path
literal or response-alias identifier used only by owned handlers. Both ownership
expansions remain direct. Generated declarations, admission, argument-buffer
lifetime, reply encoding and rollback remain unchanged. No public API, frame,
method identity, schema or budget changes; consumers need no migration.

Two attempted deduplications were rejected: sharing the complete wrapper, then
sharing just contract/admission tokens. Both emitted identical normalized Rust
tokens, but intermediate token assembly raised host expansion medians by up to
6.8%. Their exact source and measurements remain in the archive. Keeping a small
amount of explicit quotation is simpler than adding machinery to recover that cost.

The maintained `evaluate-endpoint-generation.sh` now admits the pinned compiler,
compares **both** sources for current default/explicit settings and borrowing,
and asserts normalized token equality before timing. The former one-sided
`defaults` research lane is retired; its historical report points to its archived
harness. Eight controls cover unit/scalar, explicit/default settings, async
update, borrowed query/update, renamed composite query and local-name collisions.

Rust/Cargo **1.99.1**, x86_64 Linux; one `rustc -O` harness with identical Cargo
dev-built proc-macro2/quote/Syn libraries. Each control has 11 alternating paired
batches, 8 warmups and 1,000 expansions per measured batch. This measures Wire
parsing/emission and token consumption, excluding Platform expansion, type
checking and const evaluation. It is host generation, not native codec time or
an IC instruction measurement.

| Control | Before ns | After ns | Median change |
| --- | ---: | ---: | ---: |
| Unit/default | 62,207 | 60,332 | −3.01% |
| Scalar/default | 72,742 | 74,156 | +1.94% |
| Scalar/explicit | 92,216 | 91,588 | −0.68% |
| Async update | 117,184 | 117,176 | −0.01% |
| Borrowed query | 87,669 | 87,768 | +0.11% |
| Borrowed update | 91,687 | 87,872 | −4.16% |
| Renamed composite | 97,229 | 97,307 | +0.08% |
| Local collisions | 97,538 | 96,910 | −0.64% |

The scalar/default control regresses about 1.41 µs; borrowed query and renamed
composite also increase slightly. No general speedup is claimed. Summary and
samples (raw summary retired), Retired raw output (see retention notice), Retired raw output (see retention notice)
and Retired raw output (see retention notice) retain absolute results and regressions.

Separate frozen sources and fresh target directories use identical manifests,
locks and fixture inputs. The pre-existing Testkit 0.80.1 / Runtime 0.18.1 graph
and earlier pending schema cleanup are held fixed; this is not supplier adoption.
Retired raw output (see retention notice) bind those inputs. Guest builds use
`wasm32-unknown-unknown`; Cargo reports `fresh=false` for each initial artifact.

| Profile | Total uncompressed Wasm, before → after | Code section, before → after |
| --- | ---: | ---: |
| evaluation (opt-z) | 100,637 → 100,637 | 86,698 → 86,698 |
| benchmark-speed (opt-3) | 93,997 → 93,997 | 81,012 → 81,012 |

Both complete modules are byte-identical: **0 bytes / 0% change**.
Retired raw output (see retention notice) and Retired raw output (see retention notice)
retain the result. No IC instruction speedup or application-size saving is claimed;
compressed size is not a Wasm metric.

Focused endpoint qualification passes native metadata/default/borrowing tests,
strict library/fixture lint, 11 expected compiler diagnostics and a passing
control, Wasm build/graph isolation and the installed 17-export fixture's
admission, reply failures, async calls, borrowing and rollback. No full CI or
release is inferred. Earlier pending work and unrelated readiness evidence stay
untouched.

Retired raw output (see retention notice) retain selected/rejected sources,
compiler/artifact messages, raw timings, the runner and frozen workspaces.
From the current checkout, prepare the locked cache explicitly and run:

```sh
bash scripts/ci/evaluate-endpoint-generation.sh BEFORE_ENDPOINT_MOD_RS NEW_OUTPUT_DIRECTORY
```

Both sources must support the current endpoint options. Rebuild the archived
before/after workspaces with separate Cargo target directories for guest checks;
reconcile artifact paths when relocating the comparison inputs. Original artifacts
remain under `target/endpoint-cleanup-0151-20261102/`.
Retired raw output (see retention notice) binds this record.
