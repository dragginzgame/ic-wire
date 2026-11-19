//! Candid numeric allocation probe.
mod candid;
mod host;
mod workload;

use candid::{Value, decode, encode, prepare};

fn main() {
    host::allocations();
}
