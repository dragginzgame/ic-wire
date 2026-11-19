//! Bounded numeric allocation probe.
mod bounded;
mod host;
mod wire_codec;
mod workload;

use bounded::{Value, prepare};
use wire_codec::{decode, encode};

fn main() {
    host::allocations();
}
