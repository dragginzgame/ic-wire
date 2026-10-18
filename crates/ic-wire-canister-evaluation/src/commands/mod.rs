//! Shared contracts consumed unchanged by the canister and its typed caller.
use ic_wire::{Limits, List, Method, Text, Wire};

/// Disposable browser update input.
#[derive(Debug, PartialEq, Wire)]
pub struct Command {
    /// Caller-selected correlation identity.
    #[wire(id = 4)]
    pub id: u64,
    /// Amount added to the local counter.
    #[wire(id = 5)]
    pub amount: u64,
}
/// Typed application response.
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
    Overflow(Box<Receipt>),
}
/// Typed update binding.
pub struct Apply;
impl Method for Apply {
    type Request = Command;
    type Response = Result<Receipt, Failure>;
    // This is a new experimental contract, not a reader for the old tuple probe.
    const ID: u64 = 0x494357425250;
    const NAME: &'static str = "runtime_apply";
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
    pub inspection: Box<Inspection>,
}
/// Typed ordinary query binding.
pub struct Inspect;
impl Method for Inspect {
    type Request = Box<Inspection>;
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

/// Native recursive Serde fixture; no finite Wire implementation is inferred.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NativeTree {
    /// Native text and wide integer tuple.
    Leaf(String, u128),
    /// Native dynamic recursive sequence.
    Branch(Vec<Self>),
}
/// Consumer-owned identity of the disposable native echo query.
pub const NATIVE_ID: u64 = 0x4943574e4154;
/// Explicit allowances for the native echo fixture, including its frame.
pub const NATIVE_LIMITS: Limits = Limits {
    bytes: 4096,
    allocation: 65536,
    nodes: 4096,
    depth: 64,
};
