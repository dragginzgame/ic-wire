use super::{Value, prepare};

pub(super) fn cases() -> impl Iterator<Item = (usize, usize)> {
    [1, 16, 127, 128, 4096]
        .into_iter()
        .flat_map(|len| [0, 16, 1].map(move |stride| (len, stride)))
}

pub(super) fn workload(len: usize, stride: usize) -> Value {
    prepare(
        (0..len)
            .map(|i| {
                if stride != 0 && i % stride == 0 {
                    u32::MAX
                } else {
                    1
                }
            })
            .collect(),
    )
}
