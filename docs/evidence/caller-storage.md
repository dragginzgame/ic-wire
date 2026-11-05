# Caller-storage and payload-client qualification (0.7.19 draft)

> Historical findings only. Raw measurements, snapshots, logs and build artifacts
> were retired from this tree. A summary/hash cannot verify missing payloads.
> Use the maintained development commands for fresh measurements.

Prepared for [#29](https://github.com/dragginzgame/ic-wire/issues/29) and
[#30](https://github.com/dragginzgame/ic-wire/issues/30), against released 0.7.18
`c748379325266d7fa614fefee9d745eb4a88acca`. These are codec/host measurements,
not SDK echo costs, installed consumer qualification or a Candid comparison.
Existing wire bytes, primitive acceptance, identities and admission are unchanged.
Explicit TS payload codecs add Rust payload parity: u64 7 is `07`, unit is empty.
