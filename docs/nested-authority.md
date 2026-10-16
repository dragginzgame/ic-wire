# Installed nested authority evaluation

The pending 0.1.3 experiment adds a synthetic authority update to the existing
matched canisters. It exercises two bounded collection levels and optional expiry.
It is not a Canic authentication contract or a production IC Wire API.

## Contract and limits

The Postcard layout is the following explicit declaration order:

- Authority: contract u32 (0x4155), direction u8, revision u64, delegates sequence.
- Delegate: subject u64, optional expiry u64, grants sequence.
- Grant: target u64, method u32.

Both sequence levels permit at most eight elements: at most eight delegates and
64 total grants. There is no recursive type. The visitor rejects an oversized
known length before reserving collection storage or decoding an element. For an
unknown length, element nine rejects before its type is deserialized. Reservations
use only the fixed limit, never an untrusted count. A host test instruments element
decoding to verify both paths. These finite bounds do not establish a generic
recursive allocation budget; Candid also allocates within its bounded decoder.

The outer 5,120-byte argument limit is checked before copying. Both encodings use
the existing strict canonical re-encoding check and decoder settings from the
[original experiment](canister-comparison.md). Candid quotas/type-header limits
are unchanged. Internal typed errors become IC rejection text. The handler only
sets reply direction and increments revision; it does not authorize a caller.
Field order remains the experimental wire authority; durable schema identity and
independence from incidental declaration order remain unresolved.

Fixtures use revision 42, subjects 1000+i, targets 2000+j, methods j, and expiry
1,800,000,000 for even-numbered delegates, absent for odd ones. Shapes are empty,
2 delegates × 3 grants, and 8 × 8. The frozen empty Postcard request is
`d5 82 01 00 2a 00`. Replies increment revision to 43 and set direction to 1.

## Measured results

[Saved evidence](evidence/nested-authority.json) includes exact compiler, flags,
lockfile, source, input and artifact hashes. Rust 1.99.0, Wasm, the unchanged
`canister-evaluation` profile, PocketIC server 16.0.0 and client 16.1.0 were used.
Postcard 1.1.3 and Candid 0.10.38 remain locked. No dependency changes were needed.

| Artifact | Complete Wasm bytes | Code-section payload bytes |
| --- | ---: | ---: |
| Candid, internal | 726,691 | 365,939 |
| Postcard, internal | 91,864 | 43,767 |
| Candid, management | 800,797 | 409,751 |
| Postcard, management | 691,432 | 346,385 |

Both endpoints are linked in each artifact. Differences within these matched
pairs are 634,827 bytes (87.36%) for internal-only and 109,365 bytes (13.66%) when
both retain Candid management traffic. This is fixture evidence, not an
application savings projection. Do not subtract the earlier snapshot's sizes;
its source, lockfile and endpoint set differ. No compressed size was measured.

The table combines observed ranges across internal and management artifacts,
three sequential samples per shape per artifact. Decode includes canonical
re-encoding and identity validation. Handler counters were 207 in all samples.
Counter overhead is included; argument copying, reply delivery, scheduling and
management calls are outside the measured regions. No native timing is reported.

| Shape | Candid request/reply bytes | Postcard request/reply bytes | Candid strict decode | Postcard strict decode | Candid encode | Postcard encode |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Empty | 86 | 6 | 345,055–438,854 | 9,118–9,345 | 197,883–198,603 | 6,961–6,966 |
| 2 × 3 | 186 | 37 | 422,302–425,782 | 20,568–20,834 | 208,785–210,109 | 11,939–11,944 |
| 8 × 8 | 966 | 250 | 915,987–921,713 | 100,117–100,338 | 276,669–281,350 | 49,072 |

Postcard uses fewer bytes and instructions for every shape here. Ranges are
observations, not confidence intervals or guaranteed steady-state costs.

## Verification and reproduction

Run `make check-canister-evaluation` for native tests under both codec selections,
Clippy and Rust 1.99/1.91 host/Wasm checks. With the prepared server, run:

```sh
POCKET_IC_BIN=/absolute/path/to/pocket-ic make evaluate-canisters
```

The driver verifies 36 authority replies and 36 original flat/blob replies,
including exact canonical response bytes across retained-request repetitions.
Authority inputs reject outer oversize, truncation, trailing bytes, reply direction,
nine delegates and nine nested grants: 72 installed rejections. The original
corpus adds 60 rejections; both management artifacts perform real `raw_rand` calls.
Native tests also reject every truncated prefix for every authority shape under
both selected codecs and freeze the empty Postcard vector. These tests do not
prove transaction deduplication, arbitrary hostile-input safety or canonical
business hashing. Runtime evidence is Linux-local; macOS execution is unqualified.

The production library stays dependency-free. The next bounded work is installed
schema-artifact qualification and explicit layout/client generation, including
nested browser guards. IcyDB storage and ordered keys remain outside scope.
