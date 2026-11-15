/// Wire failure categories, independent of transport and application errors.
///
/// Allocation is reserved for fallible codec-owned output allocation; consumer
/// deserialization can allocate outside the codec's control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// A wire, collection, traversal or output allowance was exceeded.
    Limit,
    /// Allocation failed within an admitted allowance.
    Allocation,
    /// The representation is invalid or truncated.
    Malformed,
    /// Input remains after the expected value.
    Trailing,
    /// The envelope marker, method identity or direction differs.
    Identity,
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Limit => "wire limit",
            Self::Allocation => "wire allocation",
            Self::Malformed => "wire malformed",
            Self::Trailing => "wire trailing",
            Self::Identity => "wire identity",
        })
    }
}

impl core::error::Error for Error {}
