# Native borrowed-view qualification

Historical finite-codec research. Its runner was retired from the active tree in
0.6.4; the commands below apply to an isolated checkout of
`9a1c06e6829da053190d997cdeed92a861addedb` (0.5.22), with that revision’s
lockfile and finite sources. Retained results are not current-codec benchmarks.

Borrowed bytes remain the most promising existing API for read-only blob work.
This experiment measures owned and borrowed projections using the public native
API at 0.5.16 (`24c1250`). It introduces no production optimization or protocol
cut: framing, identities, canonical checks, limits and accepted inputs are unchanged.

## Results

All times below are native nanoseconds per operation. Ranges span four process
medians, each from eleven batches. Percentage reductions compare owned and
borrowed within each process; they are not confidence intervals. Both runner lanes
use identical production code, so their names indicate repeat controls, not a
before/after implementation change.

| Blob bytes / text pairs | Stage | Owned ns | Borrowed ns | Time reduction |
| --- | --- | --- | --- | --- |
| 0 | decode | 62.7–63.7 | 16.8–18.2 | 71.4–73.3% |
| 0 | decode_scan | 58.8–61.2 | 17.1–17.6 | 70.6–71.6% |
| 32 | decode | 39.7–41.6 | 17.0–17.5 | 56.2–58.4% |
| 32 | decode_scan | 39.1–39.8 | 21.1–22.6 | 42.9–46.0% |
| 4096 | decode | 122.3–127.5 | 23.4–25.5 | 79.7–81.7% |
| 4096 | decode_scan | 550.9–570.9 | 453.3–464.7 | 17.3–18.8% |
| 262144 | decode | 7689.9–8496.7 | 24.0–25.3 | 99.7–99.7% |
| 262144 | decode_scan | 36008.2–36806.6 | 27605.6–28383.8 | 21.6–24.1% |
| 1048576 | decode | 32933.4–34072.0 | 23.9–25.5 | 99.9–99.9% |
| 1048576 | decode_scan | 144643.8–148422.9 | 111238.3–114921.4 | 21.5–24.7% |
| Text pairs: 1 | decode | 115.2–128.6 | 82.8–84.8 | 28.2–35.2% |
| Text pairs: 1 | decode_scan | 132.2–142.4 | 100.3–106.3 | 21.9–29.5% |
| Text pairs: 16 | decode | 1568.0–1643.3 | 731.2–761.8 | 52.5–53.7% |
| Text pairs: 16 | decode_scan | 1826.8–1926.4 | 1004.7–1048.5 | 45.0–46.2% |

`decode_scan` sums every byte after decoding. It includes output destruction,
but excludes frame creation and transport. It is a processing-cost probe, not
cryptographic verification or an end-to-end application benchmark. At 1 MiB,
pure borrowed decoding takes about 24–26 ns, but decoding and scanning takes
111–115 microseconds: the useful-work benefit is 21–25%, not a near-total saving.
The sixteen-pair case saves 45–46% with scanning. Borrowed text still allocates
the outer Vec; borrowed outputs require their input frame to remain alive.

## Consumer applicability

Read-only inspection of ic-blob-storage at `5fcb43c` found borrowed `TextRef`
already used by the CLI download descriptor decoder, and bounded borrowed
principal bytes in the tenant contract. Its read-chunk transport returns an owned
`Vec<u8>` through a provider-specific interface, followed by content verification.
These synthetic DTOs do not establish that replacing that transport with ic-wire
would help. No sibling files were changed or sibling suites run.

The next application-level decision is whether a concrete consumer can retain
its input frame and process borrowed content throughout its hot path. Copying a
view back into an owned buffer forfeits much of this benefit. No wire-format cut
is justified by these measurements alone. Bulk growth-policy tradeoffs remain
in the [reservation study](bulk-reservation-study.md).

## Reproduction and controls

```sh
IC_WIRE_NATIVE_VIEWS_ONLY=1 bash scripts/benchmarks/protocol/run.sh \
  /tmp/ic-wire-native-views-new 24c1250
```

The runner builds disposable release snapshots with the current lockfile, runs
baseline/candidate then candidate/baseline, and compares every emitted frame.
The native-view mode checks decoded content, identical re-encoding and pointers
into the input frame. It does not run the default layout probe's malformed-input
cases. No production runtime behavior changed, so broad runtime suites were not
rerun for this benchmark addition.

Environment: Linux x86_64, AMD Threadripper 7970X, rustc 1.99.1 / LLVM 23.1.1,
release opt-level 3, no LTO or extra RUSTFLAGS, no fixed CPU affinity. Warmup is
100 operations, calibration targets 10 ms capped at 65,536 repetitions, followed
by eleven timed batches. Frames are constructed outside the timed region.
Blob contents are repeated byte 42 at 0, 32, 4,096, 262,144 and 1,048,576 bytes;
method ID 11, owned `Bytes<1048576>` versus `BytesRef<1048576>`. Text fixtures
use method ID 12 and one/sixteen `(header-N, 128 repeated v characters)` pairs,
`Vec<(String, String)>` versus `Vec<(TextRef<4096>, TextRef<4096>)>`.
Both use limits: bytes 2 MiB, allocation 64 MiB, nodes 4,096, depth 32. Native
admission conservatively charges even borrowed data. An initial 8 MiB allocation
limit correctly refused the largest case; the completed runs use 64 MiB without
changing production admission rules. These limits are benchmark settings, not
an exact consumer endpoint contract.

[Raw samples](evidence/native-views/samples.csv),
[process medians](evidence/native-views/medians.csv) and
[source/frame identities](evidence/native-views/identity.txt) retain the evidence.
No IC instruction, Wasm linked-size, deployment-size or browser claim is made.
