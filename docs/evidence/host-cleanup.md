# Derive and registry host cleanup

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Baseline: released 0.7.9, `00b1686a80cd7a3068be0a24aaee0dc8239009c7`.
Candidate: the source patch and before/after source copies in Retired raw output (see retention notice).
Both changes are host work. Runtime bytes, admission, method identity, metadata
acceptance and production codecs are unchanged. No client regeneration is required
for correctness; generated tokens and TypeScript source match the baseline.
