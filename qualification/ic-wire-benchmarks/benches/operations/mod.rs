//! Private host/guest measurement protocol; unrelated to production Wire frames.
pub const ENCODE: usize = 0;
pub const DECODE: usize = 1;
pub const REUSE: usize = 2;
pub const BAD_HEADER: usize = 3;
pub const TRUNCATED: usize = 4;
pub const LOOP: usize = 5;
pub const OPERATIONS: [&str; 6] = [
    "encode-fresh",
    "decode-owned-or-labelled-borrowed",
    "encode-reuse",
    "reject-bad-header",
    "reject-truncated",
    "loop-control",
];
pub const COUNTS_BYTES: usize = OPERATIONS.len() * size_of::<u64>();
pub const FRAME_OFFSET: usize = COUNTS_BYTES + size_of::<u64>();
