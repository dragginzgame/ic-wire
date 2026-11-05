# Native marked-buffer attribution

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Investigated on 2 November 2026 after IC Persistence reported a repeated 64 KiB
Canic marked-Wire decode cost near 15 µs against approximately 1 µs plain/framed
Wire. The observation is reproducible. The large gap comes from native bulk-copy
placement sensitivity: a raw copy of the same source reproduces it, and identical
Wire bytes move between slow and fast results when their memory offset changes.
This identifies a native copy-path effect, not additional per-byte Wire parsing.

Keep the completed structured-record cuts and current production codec. No
production source, dependency, bytes, admission, ordered keys or storage contract
changes are selected by this study. No sibling checkout or original report was
modified. IcyDB engine formats remain outside this investigation.

## Exact reproduction and copy control

The original eight-case benchmark was copied into an isolated workspace. Its
Rust source is unchanged; only an empty workspace table was appended to its
manifest for isolation. Both fresh executions reproduce the marked slowdown:

| 64 KiB decode, ns/op | Run 1 | Run 2 |
| --- | ---: | ---: |
| Plain Wire | 1,262.22 | 1,159.53 |
| Framed Wire | 1,188.24 | 1,316.79 |
| Canic marked Wire | 14,963.27 | 14,984.44 |

A second frozen executable retains the original codec comparisons and adds raw
copies from each still-live encoded buffer's payload. For marked Wire, this
removes **all** decoding while retaining the source bytes/placement:

| 64 KiB operation, ns/op | Run 1 | Run 2 |
| --- | ---: | ---: |
| Marked decode | 14,778.41 | 14,843.01 |
| Raw copy from marked payload | 14,927.93 | 18,682.23 |
| Raw copy from plain payload | 997.48 | 997.07 |
| Raw copy from framed payload | 1,001.85 | 1,001.78 |

The slower raw-copy second run has a 15,115.32–18,794.08 ns p10–p90 interval;
retain its variability rather than selecting the faster sample. Original marked
payload page offset is 3,735, output offset 3,744, a difference of nine bytes
modulo 4,096. Repeated process executions preserve these relative offsets even
when absolute addresses vary. Original plain/framed payload offsets are 3,619 /
3,678 against the same output offset. The benchmark rotated codec order but
kept each encoded allocation alive, so repeated timing batches did not vary this
copy geometry.

## Same-source factorial and offset sweep

The third executable varies byte offset within one arena while keeping the
encoded frame and logical value identical. It separately checks body versus
marked tuple, small versus Canic limits, raw copying and borrowed views. There
are 29/1,024-byte controls and fifteen offsets for 65,536-byte payloads.

Run 1, Canic limits, ns/op:

| Arena offset | Payload page offset | Output page offset | Plain body decode | Marked tuple decode | Raw copy |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0 | 1,543 | 1,552 | 15,367.58 | 14,962.67 | 15,366.78 |
| 16 | 1,559 | 1,552 | 1,035.31 | 1,068.61 | 1,024.21 |
| 64 | 1,607 | 1,552 | 1,071.78 | 1,111.91 | 1,071.16 |
| 4,096 | 1,543 | 1,552 | 14,894.94 | 14,848.67 | 14,779.08 |

Run 2 repeats the pattern: marked decode 14,984.90 → 1,090.64 ns when offset
changes 0 → 16; 14,862.28 ns at offset 4,096. Plain body decode is also slow
at the bad placement. A 64-byte shift keeps cache-line alignment but removes the
large penalty, while a 4,096-byte shift restores it. This is strong evidence of
4 KiB alias sensitivity, beyond a generic unaligned-load explanation.

Borrowed marked decode stays approximately 42–49 ns at these offsets. Small
29-byte controls cost 32.67 / 34.25 ns for body small/large limits and 60.57 /
64.28 ns for tuple small/large limits in run 1; at 1,024 bytes the corresponding
figures are 43.57 / 43.78 and 73.15 / 71.68 ns. The extra tuple visits have modest
fixed cost. Neither removing the tuple nor changing the allowance removes the
copy-placement cliff. Body-only controls have different admission charges and
are research controls, not proposed replacement storage readers. Borrowing
requires retaining the input; owned Storable output still needs ownership.

## Native library diagnostic

Disassembly of the original binary shows both paths using the same Blob visitor
and `Vec<u8>::from(&[u8])` copy, which calls libc `memcpy`. With the **same copy
executable and unchanged recorded buffer offsets**, selecting
`GLIBC_TUNABLES=glibc.cpu.hwcaps=-ERMS,-FSRM` produces:

| Marked 64 KiB operation, ns/op | Diagnostic run 1 | Diagnostic run 2 |
| --- | ---: | ---: |
| Decode | 981.63 | 973.69 |
| Raw copy | 922.17 | 913.59 |

glibc documents [CPU-feature selection](https://sourceware.org/glibc/manual/2.39/html_node/Hardware-Capability-Tunables.html).
Its [2.39 x86 copy implementation](https://raw.githubusercontent.com/bminor/glibc/release/2.39/master/sysdeps/x86_64/multiarch/memmove-vec-unaligned-erms.S)
includes CPU-feature, size and address-dependent paths. Together with the
offset sweep, this attributes the observation to the native copy path. Exact
hardware stall counters and the individual ERMS-versus-FSRM contribution were
not profiled. The tunable is confined to the diagnostic process; production
settings remain unchanged. These results do not establish IC instruction/cycle
cost or a Wasm regression, and propose no custom memcpy implementation.

## Method, scope and practical follow-up

Actual rustc/Cargo 1.99.1, LLVM 23.1.1, x86_64 Linux, glibc 2.39, AMD Threadripper
7970X, CPU 2 affinity. Opt-z, fat LTO, one codegen unit, panic abort; no profile
or dependency changes between controls. Warmup and eleven >=15 ms batches per
operation; timing includes output allocation and destruction. The three sources
have independent Cargo target directories. Both benchmark and Wire artifacts
are fresh=false in each initial build. Exact manifests/locks match; only the
attribution source differs. All controls round-trip before timing.

Published Wire 0.14.2, Postcard 1.1.3, Serde 1.0.228, serde_bytes 0.11.19,
serde_cbor 0.11.2 and Candid 0.10.37 match the supplied benchmark lock. Candid is
an historical benchmark control here, not a new latest-Candid comparison. The
five relevant codec source files exactly match current Wire's working source.
The Canic wrapper algorithm is reproduced from the inspected source; no Canic
application or stable-memory workload was built. There are no IC instructions,
cycle debits, stable-memory IO or Wasm-size measurements in this native study.

Native large-buffer qualification should retain a raw-copy control over the same
source and vary buffer placement as well as codec order. Keep both fast and slow
placements visible; never select padding to manufacture a codec improvement.
The structured-record improvements and fixed-key ordering conclusions from the
original report remain intact. `serde_bytes` already selects bulk-byte decoding
in this case; a different storage format is not justified by this observation.

## Evidence and reproduction

Retired raw output (see retention notice), Retired raw output (see retention notice),
Retired raw output (see retention notice), Retired raw output (see retention notice),
Retired raw output (see retention notice),
Retired raw output (see retention notice), Retired raw output (see retention notice) and Retired raw output (see retention notice).
The raw archive also retains the supplied Persistence report/results and input
hashes for provenance. Original artifacts remain under target/ in both owners.
