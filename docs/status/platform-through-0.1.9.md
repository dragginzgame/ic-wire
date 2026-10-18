# Earlier Platform qualification through 0.1.9

These records retain their original dependency, compiler and validation scopes.
They do not qualify the current graph; see the [current handoff](current.md).

Published Platform 0.1.9 is selected from registry source `491c63d2`, archive
SHA-256 `bdfb66170e95bae421449fbd50e3e2a26196fbd6277ae13556026bf3001974c3`.
Its published README and feature/API surface were inspected; only its version and
checksum change in Cargo.lock. Strict evaluation Clippy, three native pipeline
tests, Rust 1.91 host/Wasm, direct boundary/pin checks and documentation checks pass.
Metadata/declaration scans include aliases, normal/build/dev and target-specific
edges; exactly ic0 is selected, no direct Candid/raw ABI dependency is restored,
and one upstream executor 2.0.0/ic0 1.2.0 identity remains. Production Wire's
Wasm/TypeScript graph stays Platform/CDK/Candid-free. Shared direct checks do not
enforce feature/transitive deployment exclusions; those graphs were inspected
separately, without claiming Candid absence from fixture Wasm.

Installed PocketIC 16.0.0 macro and Chromium browser trials pass. Both reports
match all prior non-provenance/Wasm-hash fields, including exact bytes, exports,
auth failures, typed replies and sixteen macro rejection/rollback cases. Testkit
retains simulator ownership. Reports and retained-build
checks finish successfully. No broad CI, native macOS or live-IC qualification ran.
Logs: `/tmp/ic-wire-platform019-*`; prior reports: `/tmp/ic-wire-platform019-reports-before/`.

The Platform 0.1.8 follow-up passes formatting/doc links, direct boundaries/pins,
strict evaluation Clippy, three native fixture tests and Rust 1.91.0 host/Wasm.
Only Platform's version/checksum changed in the lockfile. Metadata and graphs
confirm exactly ic0 enabled, no direct Candid/parser, unchanged executor/ic0
identities and production Wire independent of Platform/CDK/Candid. Installed
trials were not repeated for this version-only change; the 0.1.7 results below
remain separately qualified. No broad CI, native macOS or live IC ran.
Logs: `/tmp/ic-wire-platform018-*`.

The Platform 0.1.7 adoption passes direct boundaries/pins, formatting/doc links,
strict evaluation Clippy, three native fixture tests and Rust 1.91.0 host/Wasm.
Installed PocketIC 16.0.0 macro and Chromium browser trials pass: all prior macro
fields except provenance/Wasm hash match, including exact bytes, callable exports,
failures and 16 rejection/rollback cases. All browser non-provenance fields match.
Only Platform's version/checksum change in the lockfile; metadata confirms only
ic0 enabled, no direct Candid/parser and one upstream executor/ic0 identity.
Production Wire's Wasm/TypeScript graph remains Platform/CDK/Candid-free. No
broad CI, native macOS or installed-IC qualification ran for this follow-up.
Logs: `/tmp/ic-wire-platform017-*`; prior reports: `/tmp/ic-wire-platform017-reports-before/`.

The Platform 0.1.6 follow-up passes formatting, boundary/pin and documentation
checks, strict evaluation Clippy, three native fixture tests and Rust 1.91.0
host/Wasm checks. The lockfile changes only Platform's version and checksum;
the selected ic0/Principal/executor identities and enabled features are unchanged.
Metadata and graphs confirm no direct Candid and no Platform Candid feature.
Installed trials were not repeated for this dependency-only follow-up; the
0.1.5 installed evidence below remains separately identified.
Logs: `/tmp/ic-wire-platform016-*`.

The Wire-only retirement on Platform 0.1.5 passed the following checks:

Direct-boundary/pin checks, formatting/doc links, retained native tests and Clippy,
Rust 1.91 host/Wasm, both diagnostic suites on development/MSRV compilers, and
disposable Make/release fixtures pass. TypeScript conformance passes 84 vectors
and 16,384 differential mutations. Installed PocketIC 16.0.0 macro and Chromium
browser trials pass: callable exports, bytes, typed failures, controller rejection
and 16 macro rejection/rollback cases match the prior reports. Only provenance
and the macro Wasm hash are excluded from those result comparisons.

Cargo metadata confirms no direct Candid/parser declaration; Platform enables only
ic0. The production Wasm/TypeScript graph excludes Platform/CDK/Candid, while the
retained test graph documents the expected transitive SDK dependencies. The
pre-existing redundant license/license-file manifest warnings remain. No broad
CI, native macOS or installed-IC qualification ran. Logs use
`/tmp/ic-wire-no-candid-*`; prior installed reports are under
`/tmp/ic-wire-no-candid-reports-before/`.
