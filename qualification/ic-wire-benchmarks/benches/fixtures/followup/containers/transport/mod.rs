//! Explicit consumer DTO conversions using the shipped Packed field.
//! No codec/Serde override: conversion work is inside the measured closures.
#[cfg(any(test, not(feature = "borrowed-containers")))]
use ic_wire::Packed;
use std::{collections::BinaryHeap, num::NonZeroI64};
pub(super) mod adapters;

#[cfg(any(test, not(feature = "borrowed-containers")))]
pub(super) fn nonzero_to_wire(values: &[NonZeroI64]) -> Packed<i64> {
    Packed::new(values.iter().map(|n| n.get()).collect())
}

#[cfg(test)]
pub(super) fn nonzero_from_wire(values: Packed<i64>) -> Option<Vec<NonZeroI64>> {
    // Keep domain admission after bounded decoding and before use by a handler.
    // Conversion may allocate; it is not an additional Wire budget guarantee.
    values.into_vec().into_iter().map(NonZeroI64::new).collect()
}

#[cfg(any(test, not(feature = "borrowed-containers")))]
pub(super) fn heap_to_wire(values: &BinaryHeap<u64>) -> Packed<u64> {
    // Borrowed domain input requires a copy. Include it in the codec timings.
    Packed::new(values.as_slice().to_vec())
}

#[cfg(test)]
pub(super) fn heap_from_wire(values: Packed<u64>) -> BinaryHeap<u64> {
    // Linear heap reconstruction reuses the received Vec's allocation. Never
    // assume the remote sender supplied heap order; this also handles duplicates.
    BinaryHeap::from(values.into_vec())
}

#[cfg(test)]
mod tests;
