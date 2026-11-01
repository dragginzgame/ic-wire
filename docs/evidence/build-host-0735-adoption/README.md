# Build/Host 0.73.5 adoption

Adopted 2026-11-01. Align Wire's private Build diagnostic dependency minimum and
managed release editor on published **0.73.5**, with library defaults disabled
and `cli` enabled only for explicit editor installation. The incoming lockfile
already selected Build and Host Artifacts/FS/Process 0.73.5 and remains
byte-identical. Testkit stays 0.76.0; own Core/Endpoint versions stay 0.13.3/0.3.0.

## Supplier review

Authoritative sparse-index entries confirm 0.73.5 is the latest stable, unyanked
release observed for each selected package. Cargo archive bytes match the
registry checksums in [supplier-review.json](supplier-review.json). All four
archives identify source commit `bc1f4f392757d43639505d99fdca90711c6bf5dd`.
The immutable [Foundation release notes](https://github.com/dragginzgame/ic-tooling/blob/bc1f4f392757d43639505d99fdca90711c6bf5dd/releases/foundation/CHANGELOG.md)
describe the new existing-archive qualification API.

Build changes two Rust source files from 0.73.4, adding a callback-based boundary
for qualifying existing archives and preserving its convenience test roster.
Wire retains its existing diagnostic caller. All **147 Rust files** in the three
selected Host crates match 0.73.4; Build's four release-editor source files match
the previously selected editor 0.73.3. Per-file reviews and hashes are retained.
No sibling checkout was modified.

## Focused checks

- Prepare the exact 0.73.5 editor through the canonical installer and admit it
  offline; selected binary SHA-256/path are in [preservation.json](preservation.json).
- Run Wire's actual compiler-diagnostic caller: **nine expected diagnostics and
  the passing control**, plus strict caller Clippy and Rust **1.91.0** compilation.
- Confirm Core and endpoint Wasm normal/build graphs exclude Build, Host and
  Testkit. These are graph checks, not fresh Wasm footprint measurements.
- Exercise local Make routing, release/publication sentinels and native formatting
  in the existing disposable fixture.
- Both real release-group adapter fixtures pass with the newly selected editor;
  commits, tags and pushes are confined to temporary repositories/local bare
  remotes. Dirty/conflicting refusal, failed validation, exact metadata/tag/push,
  interrupted resume and receipt rejection pass for both Core and Endpoint.
  Outcomes are retained in [release-fixtures.log](release-fixtures.log).

The exact [qualification runner](qualify.sh) records commands and selected
features. Copy it to `target/build-host-0735-adoption/qualify.sh` to replay from
the repository; prepare the editor first with `make install-release-editor`.
Checks run offline with Rust 1.99.1/LLVM 23.1.1 on Linux x86_64, and the compiler
floor uses Rust 1.91.0. They neither install tools implicitly nor execute a real
checkout release. The two fixture script snapshots retain their effect boundaries.
No full local CI, hosted/macOS matrix or registry publication is implied.

## Preservation and reproduction

The manifest, lock, Makefile and toolchain snapshots bind the adopted settings;
`.before` files retain incoming inputs. [preservation.json](preservation.json)
proves the incoming lock is unchanged. The source base is
`c2d3dfe5266e696a9a98983bdac53fb38024e522` plus the previously accepted pending
batch; the selected runtime cleanup and its measurements remain in the
[cost study](../cost-experiments-0134/README.md).

Only private host-tool requirements, managed editor selection and documentation
change in this adoption. Published runtime graphs, codec/generator source,
protocol bytes, field order, budgets, malformed acceptance and exact retained
retry/hash inputs are unchanged. No before/after codec benchmark applies to
this host-only alignment, and no instruction, Wasm-size or build-time saving is
claimed. Earlier cost/Testkit reports retain their own frozen lockfiles and are
not cross-lock regression comparisons. Uncompressed total/code Wasm bytes remain
the footprint objective.

[SHA256SUMS](SHA256SUMS) covers all retained inputs and focused logs. Work remains
uncommitted, with no own version changes or release/publication effects.
