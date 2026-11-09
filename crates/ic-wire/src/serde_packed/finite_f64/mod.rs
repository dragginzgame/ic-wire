//! Explicit finite `Vec<f64>` byte fields, rendered as `Float64Array`.
//!
//! Select `#[serde(with = "ic_wire::serde_packed::finite_f64")]`.
//! A singleton's element prefix `01` becomes byte prefix `08`; value bits stay exact.
//! NaNs/infinities are rejected; signed zero and subnormal bits are preserved.
//! Coordinate identities/clients, preserve field constraints and retained retry
//! bytes, and requalify buffer budgets. No ordinary list is silently rewritten.
super::finite::adapter!(f64, 8; 0, 1, 2, 3, 4, 5, 6, 7);
