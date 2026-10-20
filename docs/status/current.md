# Current handoff

## Current batch

Released 0.5.2 (`7801213`) is the baseline. Pending 0.5.3 adopts published
IC Build 0.24.0 for diagnostic qualification and the release metadata editor.
The incoming Cargo manifest/lock update was preserved; Make now selects the
same editor generation. IC Build brings Host Artifacts, FS and Process 0.32.0.
IC Wire has no direct Host values to migrate and uses the unchanged diagnostic
API. Runtime codec dependencies and wire behavior remain unchanged.

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

## Qualification

Evaluation tests, strict evaluation Clippy and Rust 1.91 checks pass. All 16
compiler diagnostics and the passing control are verified. The disposable release
fixture passes with the actual 0.24.0 editor, including metadata preparation,
interrupted delivery, receipt rejection and resume. Offline editor admission,
snapshot integrity, formatting, dependency boundaries and documentation links pass.
Full CI and native macOS checks were not run; no real release effects occurred.
