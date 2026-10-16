# Matched canister experiment

Measured 2026-10-16 in local PocketIC. These are installed canister results for a
small controlled fixture, not projected savings for Canic, Relay or IcyDB.
Postcard is a promising encoding engine; this does not select a production format.

The [nested authority extension](nested-authority.md) records a later source
snapshot with both endpoints linked. The values below remain historical; current
`make evaluate-canisters` runs the extended corpus.

## Linked Wasm

| Artifact | Complete Wasm bytes | Code-section payload bytes |
| --- | ---: | ---: |
| Candid, internal only | 649,987 | 321,500 |
| Postcard, internal only | 76,815 | 36,460 |
| Candid, with management | 727,740 | 366,368 |
| Postcard, with management | 678,026 | 339,905 |

The matched internal pair differs by 573,172 bytes (88.2%). When both artifacts
retain a real Candid management call, the difference is 49,714 bytes (6.83%).
That smaller second difference is the relevant caution for applications that
cannot remove Candid. Both variants depend on the same CDK; link-time elimination,
not dependency manifests alone, determines which code remains in these artifacts.

Complete Wasm includes retained custom sections. Code-section payload length is
reported separately using wasmparser's range. There is no wasm-opt processing,
gzip compression or deployment-size measurement. No historical symbol attribution
is used to derive these differences.

## Instruction counters and payloads

Ranges across three sequential calls per input, without discarded warmups.
Decode means **strict decode including identity checks and canonical re-encoding**.
Encode means reply encoding. These are `performance_counter(0)` deltas, including
counter instrumentation overhead; they exclude argument copying, reply delivery,
whole-call scheduling and management calls. The handler region measured 207 in
all samples, including its counter overhead.

| Artifact | Blob bytes | Request/reply bytes each | Strict decode instructions | Encode instructions |
| --- | ---: | ---: | ---: | ---: |
| Candid, internal | 0 | 63 | 170,870–206,103 | 98,913–99,585 |
| Postcard, internal | 0 | 7 | 9,022 | 7,120 |
| Candid, internal | 128 | 192 | 175,781–176,478 | 100,413 |
| Postcard, internal | 128 | 136 | 13,245 | 7,514 |
| Candid, internal | 4096 | 4160 | 293,316–293,352 | 107,920 |
| Postcard, internal | 4096 | 4104 | 128,305 | 15,444 |
| Candid, management | 0 | 63 | 173,227–197,444 | 99,306–99,336 |
| Postcard, management | 0 | 7 | 9,078 | 7,146 |
| Candid, management | 128 | 192 | 176,683–176,925 | 100,630 |
| Postcard, management | 128 | 136 | 13,296–13,429 | 7,540 |
| Candid, management | 4096 | 4160 | 292,483–293,504 | 107,736–107,742 |
| Postcard, management | 4096 | 4104 | 128,501 | 15,542 |

Postcard used fewer instructions in both measured regions for every input here.
The large blob narrows the decode advantage. Candid results vary with earlier
calls and allocations; these ranges are observations, not confidence intervals
or guaranteed steady-state costs. No native wall-clock timing was measured.

## Equivalent behavior and rejection evidence

One [canister source](../crates/ic-wire-canister-evaluation/src/lib.rs) builds four
artifacts with compile-time codec/management selection. Each artifact contains
only one application encoding. This is experiment configuration, not a runtime
codec framework or parallel compatibility lanes.

The explicit schema is `(contract: u32, direction: u8, account: u64,
revision: u64, payload: bytes)`. Postcard uses that declared order; Candid uses its
record field identities. Requests use contract 0x4241, direction 0, account 7 and
revision 42. Payload byte i is i modulo 251. The handler changes direction to 1,
increments revision with wrapping arithmetic and echoes the remaining values.
Both codecs use exactly that handler and instrumentation. Payload hashes and
artifact/source hashes are in the [saved JSON](evidence/canister-comparison.json).

The driver installs all four canisters, checks 36 successful replies against the
expected typed value and canonical bytes, and performs 60 rejected calls: outer
oversize, final-byte truncation, trailing data, wrong contract and wrong direction
for each of three inputs on each artifact. It resends identical retained request
bytes and checks identical responses. The handler is stateless: this is not a
proof of transactional replay or exactly-once delivery.

Each management-enabled artifact additionally performs a real `raw_rand` call to
the management canister, Candid-decodes the response, validates its 32-byte length
and returns `ok`. This runs before its application samples, outside measured
regions. The method is absent from the internal-only artifact.

The IC argument length is checked against 5120 before copying. The blob visitor
checks 4096 before copying payload bytes. Postcard encodes into a 5120-byte stack
buffer; Candid encodes the finite bounded value then checks output length. Candid
decode settings: decoding quota 100,000, skipping quota 1,000, maximum type length
64 and maximum header length 1024. Both reject remaining/noncanonical input.
Typed internal error categories become IC rejection text at the adapter boundary.
No general nested-container budget, authentication, arbitrary hostile-input proof,
fuzz campaign or production error contract is supplied by this finite fixture.

## Reproduce

```sh
# Prepare dependencies explicitly if missing:
cargo fetch --locked
# Use an already prepared compatible server; no implicit server download:
POCKET_IC_BIN=/absolute/path/to/pocket-ic make evaluate-canisters
```

The report is written to `target/canister-evaluation-evidence/report.json` only
after verification succeeds. The four Wasm files and build provenance remain
beside it. Emulator logs are separate from JSON. This uses only local PocketIC;
it does not deploy to the live IC. Localhost binding must be permitted.

Recorded environment: Rust 1.99.0 (b940084d7), wasm32-unknown-unknown, profile
`canister-evaluation`: opt-level z, LTO, one codegen unit, panic abort, debug info
stripped. No custom Rust flags. PocketIC client 16.1.0 and server 16.0.0, default
application subnet. Candid 0.10.38, Postcard 1.1.3 with alloc/defaults disabled,
CDK 0.20.3 and ic0 1.2.0. The saved lockfile hash fixes transitive inputs; PocketIC
requires thiserror 2.0.18, which replaces 2.0.21 in the shared lock resolution.
The server binary hash is recorded. Read the source and manifests for all
configuration. The initial native payload screen remains historical evidence.

`make check-canister-evaluation` checks host tooling and both Wasm feature extremes
without starting an emulator; CI includes it. The complete runtime experiment is
explicit because it requires a prepared PocketIC server. Linux runtime execution
and checks passed locally; this new fixture has not been qualified on macOS.
The production `ic-wire` library still has no normal dependencies. The separate,
unpublished evaluation package contains CDK/Candid dependencies; host PocketIC
and analysis dependencies are development-only there.

The [IC counter API](https://docs.rs/ic-cdk/0.20.3/ic_cdk/api/fn.performance_counter.html)
and [PocketIC calls](https://docs.rs/pocket-ic/16.1.0/pocket_ic/struct.PocketIc.html)
define the execution/measurement boundaries used here.

## Decision

Continue with Postcard rather than building a bespoke serializer. The measured
fixture supports that direction even when Candid remains linked, but does not
establish savings for any existing application. Before adoption, extend the
experiment to nested authority and schema payloads, qualify schema/field identity
and full bounds, and prove the intended typed Canic/client ergonomics. IcyDB
sharing remains optional and limited to suitable messages/schema artifacts;
database storage and ordered keys remain outside this experiment.
