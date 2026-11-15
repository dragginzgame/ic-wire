# IC Wire — replacement evaluation

Evaluate whether rebuilding Wire around explicit contracts can materially reduce
architectural drift and maintained complexity. This is a design foundation, not
a decision that a rewrite is necessary or an implemented replacement.

The existing `../ic-wire` remains the production owner. Its released Core 0.22.4
is the reference; source review is pinned at `2dc434b10bec2fca92345d940dbfedc05993e9d5`.

The candidate keeps the `ic-wire` package name and uses version `0.23.7`, following
the published `0.22.4` line with a breaking Rust API. It remains `publish = false`;
the version selection and GitHub push do not constitute a crates.io release.

- [Architecture, contracts and qualification plan](docs/DESIGN.md)

Start with one end-to-end workload and independent acceptance tests. Compare a
small replacement against improving the current implementation before broadening
scope. No performance or Wasm savings are claimed by creating this repository.

No whole implementation, remote, release machinery or build artifacts were copied.
The initial [framing slice](docs/design/framing.md) records the selected source
contracts, hashes and qualification limits. The previously referenced
`reference.sha256` is absent; no verification against that manifest is claimed.
Licensing is retained in [LICENSE.md](LICENSE.md).

The unpublished `ic-wire` package implements method-ID derivation, fixed
request-header validation and bounded payload/request/response coding through
one `Encoding` selection. It remains `no_std`, using published
Serde 1.0.229 and Postcard 1.1.3. The [admission ledger](docs/design/admission.md)
records exact accounting, error precedence and narrow adapter reuse.
Encoding shares preflight across owned, reusable Vec and caller-slice outputs;
see the [output custody contract](docs/design/encoding.md). Bounded byte, text,
ordinary-list and borrowed adapters are implemented and exercised in a literal
record round trip; see [field contracts](docs/design/fields.md). The explicit
[`PackedI16` field](docs/design/packed.md) completes the native representative
record with an ordinary-list control. Optional host [schema admission and declaration
rendering](docs/design/schema.md) now import a defined JSON metadata subset into
one immutable typed graph. It generates typed payload and framed method modules with private
metadata, explicit browser policy and runtime-format mismatch refusal. The emitted
clients exchange the representative record with Rust using both the candidate and
published reference TypeScript runtimes in private qualification.
An optional strict derive now projects ordinary Rust structs/enums, including
concrete generic and borrowed instantiations, through that same graph boundary.
Named collection now handles shared and recursive Rust types with projection
rechecks and reference-compatible names. The [candidate runtime](docs/design/browser-runtime.md)
executes the admitted schema subset. Strict TypeScript checks cover the runtime
and generated clients; browser execution and matched performance qualification
remain outstanding.

The separate [compatibility workspace](qualification/README.md) compares the
candidate against published `ic-wire 0.22.4`, whose library source was verified
against the pinned revision. It exercises bytes, acceptance, budget boundaries
and output custody without introducing the reference into product dependencies.
Its isolated allocation executable checks selected preflight, output and field
admission boundaries, including positive controls and trusted-Serde limitations.

## Rust and tooling setup

The root is a virtual Cargo workspace pinned to Rust 1.100.0, including rustfmt,
Clippy and `wasm32-unknown-unknown`, with the real product package at
`crates/ic-wire` and the optional host macro at `crates/ic-wire-derive`. The separate `qualification/` workspace contains the unpublished
`ic-wire-compatibility 0.1.0` test package. No placeholder packages are present.

Toko Tooling 0.44.3 is installed from its published crate through its authenticated
snapshot exporter. The archive SHA-256 is
`8a9bc7b6827044baac8134cada7ee6bc31f86c70beec9608d1d39d911eba15e1`.
The root receipt owns the baseline and the executable closure in
`vendor/toko-tooling/`; do not edit installed files. Release commands are not enabled.

```sh
make snapshot-check
make install-tools
make tools-check
make deps-list
make deps-update
make deps-update DEPENDENCY_PACKAGE=toko-tooling
make test
make lint
make doc
make check-wasm
make test-compatibility
make test-allocation
make lint-compatibility
make test-host             # Focused host schema contracts
make lint-host
make doc-host
make check-wasm-host       # TypeScript feature forwarding on Wasm
make test-typescript       # Rust ↔ candidate/reference TS; explicit Node prerequisite
make lint-typescript
make test-consumer         # Independent contract against both Rust suppliers
make check-consumer-wasm   # Separate SDK-free Wasm builds and Node execution
make check-typescript      # Strict client types; install private npm compiler first
make test-derive           # Macro rejection and derived metadata contracts
make lint-derive
make fmt-check
```

`deps-list` lists local package versions; `deps-update` runs `cargo update -v`.
Both automatically include `qualification/Cargo.toml` when it exists. Updates
accept `DEPENDENCY_PACKAGE` and, with it, `DEPENDENCY_PRECISE`; an additional
workspace can be selected with `DEPENDENCY_MANIFEST=relative/Cargo.toml`.

Cargo.lock now selects the inherited `toko-tooling = "=0.44.3"` dev-dependency.
`make toko-tooling-check` verifies package selection, lock and installed bytes.
Every new member must inherit the tooling dev-dependency and workspace lints,
use the cargo-machete tooling exemption, and remain unpublished during evaluation.
`make qualification-tooling-check` admits the qualification selection against
the same verified snapshot; compatibility checks run this automatically.
Reviewed tooling updates use `make toko-tooling-preview` and
`make toko-tooling-update` after explicitly updating the Cargo selection.

For direct interactive tool use, add `.tools/host/bin` and
`.tools/rust/common/bin` under this checkout to PATH. Make sets these paths itself.
No hooks, CI workflows, publication or deployment are activated by this setup.
