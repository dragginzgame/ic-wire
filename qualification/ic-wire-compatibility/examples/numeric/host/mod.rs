#[cfg(test)]
mod tests;

use super::{
    decode, encode,
    workload::{cases, workload},
};
use std::hint::black_box;

#[global_allocator]
static ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;

pub(crate) fn allocations() {
    for (len, stride) in cases() {
        let value = workload(len, stride);
        let warm = encode(&value);
        assert_eq!(decode(&warm), value);
        let region = stats_alloc::Region::new(ALLOCATOR);
        let bytes = black_box(encode(black_box(&value)));
        let encoding = region.change();
        let region = stats_alloc::Region::new(ALLOCATOR);
        let roundtrip = black_box(decode(black_box(&bytes)));
        let decoding = region.change();
        assert_eq!(roundtrip, value);
        println!(
            "len={len} stride={stride} payload={} encode={encoding:?} decode={decoding:?}",
            bytes.len()
        );
    }
}
