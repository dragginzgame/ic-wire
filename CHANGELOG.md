# Changelog

## [0.1.0]

- Initialize the maintainer-authorized private `dragginzgame/ic-wire` GitHub
  repository and activate the local formatting hook; package publication remains
  disabled.
- Add an evaluation-only comparison of Candid, Postcard, CBOR and an independently
  authored fixed-layout comparator across seven synthetic fixtures, including an
  IcyDB-shaped schema artifact. Record payload sizes/hashes, strict round trips,
  malformed-input tests and the positional field-identity limitation. Keep the
  runtime library dependency-free; no codec selection or adoption is implied.
- Establish an unpublished, dependency-free Rust workspace and no_std scaffold.
- Record scope, proposed endpoint ergonomics, codec evaluation criteria and
  preliminary Canic code-size attribution. No codec or protocol is implemented.
- Adopt reviewed Shared Tooling 0.15.4 for explicit developer tool setup,
  formatting, Git hooks, snapshot verification and workspace counts. Add local
  Make checks and Linux/macOS CI; release delivery remains disabled.
- Evaluate optional codec sharing with IcyDB. Identify reusable bounded byte
  handling and preserve database-specific format requirements; sharing remains
  a candidate to measure, with no codec or adoption selected.
