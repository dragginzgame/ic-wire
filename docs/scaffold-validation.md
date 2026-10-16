# Scaffold validation

Validated on 2026-10-16 with Rust 1.99.0 in an isolated staging directory before
installation at /home/adam/projects/ic-wire:

- `cargo check --offline -p ic-wire` passed and generated the dependency-free lockfile.
- `cargo check --offline --locked -p ic-wire --target wasm32-unknown-unknown` passed.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --offline --locked -p ic-wire --lib -- -D warnings` passed.

These checks cover an empty no_std library, not codec behavior or performance.
The declared Rust 1.91 floor has not been tested. No tests were added for an
empty API. Local Markdown links were checked before installation.
