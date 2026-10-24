# Hotspot evidence

The [report](../../hotspot-encoding-0613.md) owns interpretation and tradeoffs.
`identity.json` hashes the artifacts; `checks.json` records validation. CSVs retain
all IC variants, standard-corpus changes, native two-run medians and uncompressed
Wasm total/code sizes. Counts include overlapping control cohorts.

`raw.tar.gz` is evidence storage, not a gzip-size optimization measurement. It
contains the exact baseline source/lock, research source overlays, the retained
commented implementation, 48 Wasms, all repeated IC/native samples and identities,
build/validation logs, parameters, source hashes and analysis/replay scripts.
Only the i16 writer is retained in production; overlays are rejected controls.
No native frame/input files are duplicated per run: fixtures reproduce them and
their recorded hashes validate against IC identities.

To replay, extract outside the working tree and use a fresh absolute scratch path:

```sh
tar -xzf raw.tar.gz -C /tmp/hotspot-evidence
POCKET_IC_BIN=/prepared/pocket-ic \
  bash /tmp/hotspot-evidence/raw/scripts/replay.sh \
  /tmp/hotspot-replay /path/to/ic-wire
```

Create the extraction directory first. Prepare the pinned toolchains, Wasm target,
PocketIC 16.1.0 and Cargo dependencies beforehand; the replay builds offline.
The original checkout supplies read-only Git identity to the native runner;
archived sources and hashes identify actual candidates. No staging or commits
are performed. Expect a substantial benchmark run. The constituent scripts ran
successfully; a second complete replay has not been executed. The initial stale
shared-cache screen was discarded; retained runs touch local sources when
switching copies. The u16-inlining probe produces identical Wasms and has no
extra IC sample set.
