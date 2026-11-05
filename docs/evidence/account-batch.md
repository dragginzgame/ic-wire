# Rebased Account batch preparation

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Prepared on 2 November 2026 against committed Core 0.15.1
`aa308c9c859687f505cdde33a5d47c96c99ac601` and Platform
`868c3c788a3b4ee93df3994a06bb44e6746331c8` (Types 0.17).
This supersedes the old pending-version patches in
[the first study](hotspots.md), not its historical measurements.

## Implementation and compatibility

Retired raw output (see retention notice) selects the standard
`serde_bytes` field adapter on the existing `Option<[u8; 32]>`. It retains
`serde(default)` for omitted optional map fields, adds raw-field tests (Account
equality alone normalizes absent and zero), and preserves the current SHA-2 0.11
supplier, source notices and cryptographic bodies. It adds no codec or type wrapper.

Retired raw output (see retention notice) describes the subaccount through
the existing schema trait as optional bytes with minimum/maximum 32, and extends
the standalone Rust–TypeScript byte, refusal and admission fixtures. Generated
Some values use `Uint8Array`. Principal remains bounded to 29 bytes. Ordinary
integer-vector benchmarks remain visible; Serde is not implicitly rewritten.

This is a coordinated binary contract cut. For a present zero subaccount,
`01 + 32 zero bytes` becomes **`01 20 + 32 zero bytes`**. None remains `00`.
Owner/subaccount order, raw absent versus zero distinction, equality, ordering,
hashing, text and external Candid DTOs are preserved. Other Serde formats observe
byte serialization and need separate qualification. Budgets must be requalified;
keep original retained retry, signature and encoder hash inputs. There is no
legacy reader or automatic migration, and arbitrary old frames are not guaranteed
always to fail. Coordinate identities and consumers rather than relying on rejection.

Both contributions are reviewable isolated source preparation. The normal
Wire workspace deliberately retains the published Types 0.17 array contract.
No sibling working checkout, own package version, staging, commit, PR, release
or publication is changed. The unrelated primary lock/readiness work is preserved.
