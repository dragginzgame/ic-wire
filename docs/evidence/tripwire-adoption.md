# Tripwire ideas applied to Wire

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Measured 30 October 2026 against current 0.7.24 source; pending 0.7.25.
One production Serde/Postcard codec remains. The retained endpoint change checks
the existing eleven-byte header before copying the complete ingress and shares
cold error formatting across admission, decoding and reply encoding. No dependency,
wire-byte, limit, typed-error or accepted-input change. For ID 7 / u8 42 the frame
remains `49 57 07 00 00 00 00 00 00 00 00 2a`. Consumers need no migration.

Generated guards call `Contract::admit_request()`. Existing standalone
`admit_request(Limits)` remains a size-only primitive for manually composed
Platform guards. Both use the same size check; the contract method also uses the
existing header validator. Full decoder validation remains mandatory. Platform
still owns exports/execution; framework authentication ordering is unchanged.
