# Current handoff

## Current batch

Released 0.5.4 (`1e1ea56`) is the baseline. Pending 0.5.5 improves allocation
benchmark interpretation: `--codec-only` removes the application payload scan,
while the default retains it. Both modes include canonical checks and destruction;
constant-time packed-length checks remain. Encode/decode calibration is independent.
Reports use format 2 with separate repetition counts and a decode-work label.
The incoming IC Build/Host 0.62.1 lockfile edit was preserved.

## Experiments and decisions

An isolated 60-case IC probe measured finite output reuse, retained retry copies,
owned/borrowed decoding, scan separation and UTF-8 validation before allocation.
Warm reuse saves 10.5–34.8% encode instructions under opt-3 for tested blob sizes,
but copying the result for retry retention loses the gain. A 64-case follow-up
adds mixed-size chunk cycles: reuse saves about 23%, including initially empty
cycles. Finite `encode_request_into`/`encode_response_into` are now supported.
The fresh encoder stays intact: matched fresh-case instruction counts are unchanged.
Callers own retained capacity and must keep exact retry bytes independently.

Both pre-allocation UTF-8 candidates sharply reduce malformed-input work but
regress small valid strings under opt-3 and grow linked Wasm. Neither was adopted.
Text decoding remains unchanged. See
[methodology and complete results](../evaluation.md#finite-reuse-text-validation-and-scan-separation).
Sources, raw runs, locks and prototypes are retained at `target/reuse-text-scan/`.

## Qualification

Strict evaluation Clippy, evaluation tests and Rust 1.91 checks pass. Both native
benchmark modes completed all 44 cases with identical frames and capacities;
format-2 repetition fields were checked. Isolated reuse tests cover changed
sizes, borrowed values, frozen retries and failure clearing with spare capacity.
Runtime tests, strict Clippy, Rust 1.91 host/Wasm checks and documentation links
also pass for finite scratch APIs. See the
[follow-up evidence](../evaluation.md#finite-scratch-buffer-follow-up).
Full CI, macOS and installed-consumer validation were not run.

## Retained contract

Production remains bounded Postcard; the released output-growth/header improvements
remain. Rejected preference and pointer-comparison experiments remain historical.
Shared Tooling is 0.25.8. The release editor remains separately pinned to IC Build
0.24.0. Staging, commits, versions, releases and publication remain maintainer-owned.
