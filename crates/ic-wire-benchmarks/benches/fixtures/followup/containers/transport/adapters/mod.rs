//! Public borrowed adapters versus a private copying encoder control.
//! Copy and borrowed encoders have the same packed bytes and owned field sizes;
//! both decoders validate domain values through the existing Packed reader.
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct NonzeroValue(#[serde(with = "nonzero")] pub Vec<NonZeroI64>);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct HeapValue(#[serde(with = "heap")] pub BinaryHeap<u64>);

mod nonzero {
    use super::*;
    pub(super) fn serialize<S: serde::Serializer>(
        values: &[NonZeroI64],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        #[cfg(feature = "borrowed-containers")]
        {
            ic_wire::serde_packed::nonzero_i64::serialize(values, serializer)
        }
        #[cfg(not(feature = "borrowed-containers"))]
        {
            nonzero_to_wire(values).serialize(serializer)
        }
    }
    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<NonZeroI64>, D::Error> {
        ic_wire::serde_packed::nonzero_i64::deserialize(deserializer)
    }
}

mod heap {
    use super::*;
    pub(super) fn serialize<S: serde::Serializer>(
        values: &BinaryHeap<u64>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        #[cfg(feature = "borrowed-containers")]
        {
            ic_wire::serde_packed::heap_u64::serialize(values, serializer)
        }
        #[cfg(not(feature = "borrowed-containers"))]
        {
            heap_to_wire(values).serialize(serializer)
        }
    }
    pub(super) fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BinaryHeap<u64>, D::Error> {
        ic_wire::serde_packed::heap_u64::deserialize(deserializer)
    }
}
