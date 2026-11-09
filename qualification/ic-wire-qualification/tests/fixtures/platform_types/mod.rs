//! Standalone consumer proof transferred from Platform commit 421edbc.
use ic_platform_types::Principal;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(not(target_family = "wasm"), derive(ic_wire::Wire))]
pub struct Request {
    pub caller: Principal,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(not(target_family = "wasm"), derive(ic_wire::Wire))]
pub struct AccountRequest {
    pub account: ic_platform_types::Account,
}

// Replace the old qualification export echo_account with echo_account_bytes.
// Its new name gives the current name-derived guard a fresh key for Types 0.18's
// length-prefixed subaccount; no old route/reader remains. Real consumers must
// likewise coordinate fresh keys and stored data, pending automatic fingerprints.
#[cfg(feature = "endpoint")]
#[ic_wire_endpoint::query(contract = ACCOUNT_CONTRACT)]
pub fn echo_account_bytes(value: AccountRequest) -> AccountRequest {
    value
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests;
