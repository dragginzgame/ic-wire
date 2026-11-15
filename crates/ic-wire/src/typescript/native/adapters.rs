//! Sealed projection of the exact standard packed-adapter input containers.
use super::{GenerationError, SchemaContext};
use serde_json::{Value, json};
mod sealed {
    pub trait Input<const KIND: u8> {
        const WORD: &'static str;
        const NONZERO: bool = false;
    }
}
/// Qualified input for one explicit packed field adapter. Not consumer-extensible.
///
/// ```compile_fail
/// fn bool_input<T: ic_wire::typescript::PackedField<0>>() {}
/// bool_input::<Vec<u8>>();
/// ```
/// ```compile_fail
/// fn floats<T: ic_wire::typescript::PackedField<1>>() {}
/// floats::<Vec<ic_wire::FiniteF32>>();
/// ```
#[doc(hidden)]
pub trait PackedField<const KIND: u8>: sealed::Input<KIND> {}
impl<T: sealed::Input<KIND>, const KIND: u8> PackedField<KIND> for T {}
impl sealed::Input<0> for alloc::vec::Vec<bool> {
    const WORD: &'static str = "bool";
}
impl sealed::Input<1> for alloc::vec::Vec<f32> {
    const WORD: &'static str = "finite-f32";
}
impl sealed::Input<2> for alloc::vec::Vec<f64> {
    const WORD: &'static str = "finite-f64";
}
impl sealed::Input<3> for alloc::vec::Vec<core::num::NonZeroI64> {
    const WORD: &'static str = "i64";
    const NONZERO: bool = true;
}
impl sealed::Input<4> for alloc::collections::BinaryHeap<u64> {
    const WORD: &'static str = "u64";
}
impl SchemaContext {
    /// Project a qualified adapter as an inline field, without inventing named
    /// types for adapters. Final graph admission accounts for the scalar node.
    /// # Errors
    /// Rejects a previously failed context; ordinary final graph limits apply.
    #[doc(hidden)]
    pub fn describe_packed<T: PackedField<KIND>, const KIND: u8>(
        &mut self,
    ) -> Result<Value, GenerationError> {
        if self.failed {
            return Err(GenerationError::Schema);
        }
        Ok(if T::NONZERO {
            json!({"k":"packed","word":T::WORD,"nonzero":true})
        } else {
            json!({"k":"packed","word":T::WORD})
        })
    }
}
