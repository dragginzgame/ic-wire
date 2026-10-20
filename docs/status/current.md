# Current handoff

## Current batch

Released 0.5.3 (`d1d9ca2`) is the baseline. Pending 0.5.4 records the incoming
IC Build diagnostic dependency update from 0.24.0 to 0.62.0, with Host Artifacts,
FS and Process at 0.62.0. The separate release metadata editor remains pinned to
0.24.0. This changelog-only batch preserves the manifest and lockfile edits;
the new dependency selection has not been qualified in this batch.

Shared Tooling remains 0.25.8 (`0a2d2ee3b5e0ae77755c2915ac28836d4d5ba359`).
The editor is prepared through `make install-tools` or `make install-release-editor`
and admitted offline by `make release-editor-check`, retaining the 8 MiB
per-document allowance. Releases select its path without manual environment setup.

## Retained decisions

The 0.5.1 bounded output-growth and 0.5.2 direct-header improvements are released.
See [header measurements](../evaluation.md#direct-header-initialization).
Per-type codec preferences and the native pointer-equality candidate were rejected;
[experimental evidence](../evaluation.md#native-borrowed-pointer-equality-experiment)
remains historical. No alternate production codec or preference API is supported.

Staging, commits, versions, tags, pushes and publication remain maintainer-owned.

## Previous qualification (IC Build 0.24.0)

Evaluation tests, strict evaluation Clippy and Rust 1.91 checks pass. All 16
compiler diagnostics and the passing control are verified. The disposable release
fixture passes with the actual 0.24.0 editor, including metadata preparation,
interrupted delivery, receipt rejection and resume. Offline editor admission,
snapshot integrity, formatting, dependency boundaries and documentation links pass.
Full CI and native macOS checks were not run; no real release effects occurred.
