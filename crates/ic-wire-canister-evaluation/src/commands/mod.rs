//! Shared contracts consumed unchanged by the canister and its typed caller.
use ic_wire::{Bytes, Limits, List, Method, Text, Wire};

/// A caller-bound command; principal validation remains with the adapter.
#[derive(Debug, PartialEq, Wire)]
pub struct Command {
    /// Original caller's principal bytes.
    #[wire(id = 1)]
    pub subject: Bytes<29>,
    /// Receiving canister's principal bytes.
    #[wire(id = 2)]
    pub audience: Bytes<29>,
    /// Exclusive expiry, in IC nanoseconds.
    #[wire(id = 3)]
    pub expiry: u64,
    /// Caller-scoped request identity.
    #[wire(id = 4)]
    pub id: u64,
    /// Amount added to the local counter.
    #[wire(id = 5)]
    pub amount: u64,
}
/// Durable application receipt.
#[derive(Debug, PartialEq, Wire)]
pub struct Receipt {
    /// Request identity.
    #[wire(id = 1)]
    pub id: u64,
    /// Counter after handling the command.
    #[wire(id = 2)]
    pub balance: u64,
}
/// Application failure with the original receipt context.
#[derive(Debug, PartialEq, Wire)]
pub enum Failure {
    /// Addition did not fit; the unchanged balance is retained.
    #[wire(id = 7)]
    Overflow(Receipt),
}
/// Typed update binding.
pub struct Apply;
impl Method for Apply {
    type Request = Command;
    type Response = Result<Receipt, Failure>;
    // This is a new experimental contract, not a reader for the old tuple probe.
    const ID: u64 = 0x49435752504c;
    const NAME: &'static str = "replay_command";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 128,
        allocation: 512,
        nodes: 32,
        depth: 8,
    };
    const RESPONSE_LIMITS: Limits = Limits {
        bytes: 48,
        allocation: 128,
        nodes: 16,
        depth: 8,
    };
}
/// Nested query contract used to qualify bounds beyond scalar fields.
#[derive(Debug, PartialEq, Wire)]
pub struct Inspection {
    /// Finite labels echoed in the snapshot.
    #[wire(id = 1)]
    pub labels: List<Text<32>, 4>,
    /// Optional caller-selected correlation value.
    #[wire(id = 2)]
    pub correlation: Option<u64>,
}
/// Current counter and the checked nested request.
#[derive(Debug, PartialEq, Wire)]
pub struct Snapshot {
    /// Current counter.
    #[wire(id = 1)]
    pub balance: u64,
    /// Echo of the bounded query.
    #[wire(id = 2)]
    pub inspection: Inspection,
}
/// Typed ordinary query binding.
pub struct Inspect;
impl Method for Inspect {
    type Request = Inspection;
    type Response = Snapshot;
    const ID: u64 = 0x49435752454144;
    const NAME: &'static str = "runtime_inspect";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 256,
        allocation: 1024,
        nodes: 32,
        depth: 8,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
