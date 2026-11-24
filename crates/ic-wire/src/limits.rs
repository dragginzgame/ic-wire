/// Consumer-selected Serde traversal allowances.
///
/// Custom Serde code is trusted: work and allocation outside visited values
/// cannot be constrained by these allowances. Allocation is conservative
/// accounting, not a live-heap cap, and depends on target type layouts.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum encoded/input bytes, including the header when framed.
    pub bytes: usize,
    /// Reserve all configured bytes, then eight times visited value sizes and
    /// byte/text spans. Borrowed spans pay the same charge as owned spans.
    pub allocation: usize,
    /// Maximum visits, including seed visits and zero-sized values.
    pub nodes: usize,
    /// Maximum visit depth, starting at one.
    pub depth: usize,
}

impl Limits {
    pub(crate) fn without_prefix(self, prefix: usize) -> Result<Self, crate::Error> {
        Ok(Self {
            bytes: self
                .bytes
                .checked_sub(prefix)
                .ok_or(crate::Error::Limit(crate::LimitKind::Bytes))?,
            allocation: self
                .allocation
                .checked_sub(prefix)
                .ok_or(crate::Error::Limit(crate::LimitKind::Allocation))?,
            ..self
        })
    }
}
