//! Explicit finite `Vec<f32>` byte fields, rendered as `Float32Array`.
//!
//! Select `#[serde(with = "ic_wire::serde_packed::finite_f32")]`.
//! `[1.0]` changes from `01 00 00 80 3f` to `04 00 00 80 3f`.
//! NaNs/infinities are rejected; signed zero and subnormal bits are preserved.
//! Coordinate identities/clients, preserve field constraints and retained retry
//! bytes, and requalify buffer budgets. No ordinary list is silently rewritten.
super::finite::adapter!(f32, 4; 0, 1, 2, 3, 4, 5, 6, 7; 0, 1, 2, 3);
