//! Packed numeric allocation probe.
mod host;
mod packed;
mod wire_codec;
mod workload;

use packed::{Value, prepare};
use wire_codec::{decode, encode};

fn main() {
    host::allocations();
}
