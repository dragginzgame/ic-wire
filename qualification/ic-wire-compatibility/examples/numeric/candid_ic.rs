//! Candid numeric instruction probe.
mod candid;
mod guest;
mod workload;

use candid::{Value, decode, encode, prepare};
