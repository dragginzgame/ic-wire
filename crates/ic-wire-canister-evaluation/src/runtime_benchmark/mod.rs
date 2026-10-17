//! Matched logical payloads using the production runtime and ordinary Candid DTOs.
pub mod shapes;
use candid::CandidType;
use ic_wire::{Bytes, FiniteF64, Limits, List, Method, Text, Wire};
use serde::{Deserialize, Serialize};

/// One bounded mixed record; empty collections still retain their schema costs.
#[derive(Clone, Debug, PartialEq, Wire)]
pub struct Packet {
    /// Handler increments this value.
    #[wire(id = 1)]
    pub revision: u64,
    /// Opaque payload, at most 4096 bytes.
    #[wire(id = 2)]
    pub blob: Bytes<4096>,
    /// UTF-8 label, at most 256 bytes.
    #[wire(id = 3)]
    pub label: Text<256>,
    /// Up to 64 finite double-precision values.
    #[wire(id = 4)]
    pub samples: List<FiniteF64, 64>,
    /// Optional correlation identity.
    #[wire(id = 5)]
    pub correlation: Option<u64>,
}
/// Evaluation-only endpoint identity and finite schema allowances.
pub struct Exchange;
impl Method for Exchange {
    type Request = Packet;
    type Response = Packet;
    const ID: u64 = 0x49435742454e43;
    const NAME: &'static str = "runtime_benchmark";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 5120,
        allocation: 16384,
        nodes: 128,
        depth: 8,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
/// Candid projection with the same logical values and explicit identity.
#[derive(Clone, Debug, PartialEq, CandidType, Serialize, Deserialize)]
pub struct CandidPacket {
    /// Matches the runtime method identity.
    pub method: u64,
    /// Zero request, one response.
    pub direction: u8,
    /// Same revision as Packet.
    pub revision: u64,
    /// Same opaque bytes as Packet.
    pub blob: crate::Blob,
    /// Same UTF-8 text as Packet.
    pub label: String,
    /// Same finite f64 values as Packet.
    pub samples: Vec<f64>,
    /// Same optional correlation as Packet.
    pub correlation: Option<u64>,
}
impl Packet {
    /// Deterministic inputs shared by both codecs, outside measured regions.
    pub fn fixture(blob: usize, text: usize, samples: usize) -> Self {
        Self {
            revision: 42,
            blob: Bytes::new((0..blob).map(|i| (i % 251) as u8).collect()).unwrap(),
            label: Text::new("é".repeat(text / 2)).unwrap(),
            samples: List::new(
                (0..samples)
                    .map(|i| FiniteF64::new(i as f64 * -0.125).unwrap())
                    .collect(),
            )
            .unwrap(),
            correlation: Some(7),
        }
    }
    /// Independent Candid projection; never called in a measured canister region.
    pub fn candid(&self, direction: u8) -> CandidPacket {
        CandidPacket {
            method: Exchange::ID,
            direction,
            revision: self.revision,
            blob: crate::Blob::new(self.blob.as_slice()).unwrap(),
            label: self.label.as_str().to_owned(),
            samples: self.samples.as_slice().iter().map(|v| v.get()).collect(),
            correlation: self.correlation,
        }
    }
}

#[ic_cdk::update(manual_reply = true)]
fn runtime_benchmark() {
    if ic0::msg_arg_data_size() > crate::MAX_WIRE {
        ic_cdk::api::msg_reject("wire limit");
        return;
    }
    let bytes = ic_cdk::api::msg_arg_data();
    let start = ic_cdk::api::instruction_counter();
    #[cfg(feature = "candid-codec")]
    let value = crate::decode_value::<CandidPacket>(&bytes).and_then(|value| {
        if value.method != Exchange::ID || value.direction != 0 {
            return Err(crate::CodecError::Identity);
        }
        // The Candid control uses the existing decoder quotas; these semantic
        // bounds run after decoding. It does not claim Wire's preallocation bound.
        if value.label.len() > 256
            || value.samples.len() > 64
            || value.samples.iter().any(|v| !v.is_finite())
        {
            return Err(crate::CodecError::Limit);
        }
        Ok(value)
    });
    #[cfg(not(feature = "candid-codec"))]
    let value = ic_wire::decode_request::<Exchange>(&bytes);
    let mut value = match value {
        Ok(value) => value,
        Err(error) => {
            ic_cdk::api::msg_reject(format!("{error:?}"));
            return;
        }
    };
    let decoded = ic_cdk::api::instruction_counter();
    value.revision = value.revision.wrapping_add(1);
    #[cfg(feature = "candid-codec")]
    {
        value.direction = 1;
    }
    let handled = ic_cdk::api::instruction_counter();
    #[cfg(feature = "candid-codec")]
    let reply = crate::encode(&value).unwrap();
    #[cfg(not(feature = "candid-codec"))]
    let reply = ic_wire::encode_response::<Exchange>(&value).unwrap();
    let encoded = ic_cdk::api::instruction_counter();
    crate::COUNTERS.with_borrow_mut(|counts| {
        *counts = [decoded - start, handled - decoded, encoded - handled]
    });
    #[cfg(feature = "candid-codec")]
    ic_cdk::api::msg_reply(&reply);
    #[cfg(not(feature = "candid-codec"))]
    ic_cdk::api::msg_reply(reply.as_bytes());
}
