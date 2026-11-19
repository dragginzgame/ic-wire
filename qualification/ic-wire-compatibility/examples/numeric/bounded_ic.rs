//! Bounded numeric instruction probe.
mod bounded;
mod guest;
mod wire_codec;
mod workload;

use bounded::{Value, prepare};
use wire_codec::{decode, encode};
