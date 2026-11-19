//! Packed numeric instruction probe.
mod guest;
mod packed;
mod wire_codec;
mod workload;

use packed::{Value, prepare};
use wire_codec::{decode, encode};
