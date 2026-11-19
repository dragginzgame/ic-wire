//! Ordinary numeric allocation probe.
mod host;
mod ordinary;
mod wire_codec;
mod workload;

use ordinary::{Value, prepare};
use wire_codec::{decode, encode};

fn main() {
    host::allocations();
}
