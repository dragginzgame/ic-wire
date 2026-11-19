//! Ordinary numeric instruction probe.
mod guest;
mod ordinary;
mod wire_codec;
mod workload;

use ordinary::{Value, prepare};
use wire_codec::{decode, encode};
