#[cfg(test)]
mod tests;

use super::{black_box, decode, encode, workload};

#[global_allocator]
static ALLOCATOR: &stats_alloc::StatsAlloc<std::alloc::System> = &stats_alloc::INSTRUMENTED_SYSTEM;

pub(crate) fn allocations() {
    for (len, stride) in [1, 16, 127, 128, 4096]
        .into_iter()
        .flat_map(|len| [0, 16, 1].map(|stride| (len, stride)))
    {
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
