# Initial Canic evidence

Investigation date: 2026-10-16. These are historical local artifact measurements,
not a qualification of the current dirty Canic checkout or a before/after codec test.

| Attribution | Blob consumer | Embedded Blob consumer |
| --- | ---: | ---: |
| Functions with Candid in their names | 2,386,510 bytes | 2,393,453 bytes |
| Subset also naming Canic Core or Contracts | 995,252 bytes | 997,120 bytes |

The first artifact has a 5,233,877-byte code section. Its total file is 9,713,650
bytes including 4,058,794 bytes of custom sections. Attribution used twiggy 0.8.0
shallow sizes, selecting names matching `\bcandid\[` and, for the subset,
`\bcanic_(?:core|contracts)\[`. Debug-name metadata is excluded from those sums.

The artifact was present under Canic's target/wasm32-unknown-unknown/fast directory
and had an October 14 timestamp. Its SHA-256 and exact counts are retained in
[evidence metadata](evidence/canic-blob-attribution.json). The binaries and full
symbol tables are not vendored here. These figures cannot be reproduced from this
scaffold alone, and their build-source identity is not established by the hash.

This is code attribution, not removable-byte accounting. Generic/shared code,
external interfaces and new codec code affect net savings. No runtime speedup,
compressed-byte saving or replacement wire-size result has been measured.

Source inspection also found Candid in Canic endpoint generation, RPC traits,
auth argument extraction, replay response caches and several authority hashes.
Current Core dependencies retain Candid through ic-cdk and ic-auth-protocol-types.
The initial target is zero Candid in internal wire contracts, not an unsupported
claim of zero Candid in every linked canister.
