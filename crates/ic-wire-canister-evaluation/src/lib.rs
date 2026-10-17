//! Finite, unpublished matched canister fixture; not the IC Wire runtime.
/// Installed proof of the raw CDK endpoint companion.
#[cfg(feature = "cdk-fixture")]
pub mod cdk_fixture;

/// Evaluation-only Canic-shaped macro integration fixture.
#[cfg(feature = "macro-fixture")]
pub mod macro_fixture;

/// Production-runtime performance comparison against Candid.
#[cfg(feature = "runtime-benchmark")]
pub mod runtime_benchmark;

/// Bounded nested evaluation contract.
pub mod authority;

#[cfg(feature = "peer")]
mod peer;

/// Shared runtime contracts used by the installed adoption trial.
#[cfg(feature = "replay")]
pub mod commands;

#[cfg(feature = "replay")]
mod replay;

use candid::CandidType;
use serde::{Deserialize, Serialize, de};
use std::{cell::RefCell, fmt};

/// Maximum accepted transport envelope, checked before copying IC arguments.
pub const MAX_WIRE: usize = 5120;
/// Maximum payload bytes, shared with generated browser contract metadata.
pub const MAX_BLOB: usize = 4096;
/// Frozen experimental method identity; schema changes require deliberate review.
pub const CONTRACT: u32 = 0x4241;

/// Bounded payload, with a byte-string representation in both codecs.
#[derive(Clone, Debug, PartialEq, CandidType)]
pub struct Blob(Vec<u8>);
impl Blob {
    /// Construct a finite fixture, rejecting oversize before copying.
    pub fn new(bytes: &[u8]) -> Option<Self> {
        (bytes.len() <= MAX_BLOB).then(|| Self(bytes.to_vec()))
    }
}
impl Serialize for Blob {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(&self.0)
    }
}
impl<'de> Deserialize<'de> for Blob {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl de::Visitor<'_> for Visitor {
            type Value = Blob;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("at most 4096 bytes")
            }
            fn visit_bytes<E: de::Error>(self, bytes: &[u8]) -> Result<Blob, E> {
                Blob::new(bytes).ok_or_else(|| E::custom("blob limit"))
            }
        }
        d.deserialize_bytes(Visitor)
    }
}

/// Fixed schema: field order below is the explicit Postcard layout authority.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
pub struct Message {
    /// Contract discriminator, shared by request and response.
    pub contract: u32,
    /// Zero for a request, one for its response.
    pub direction: u8,
    /// Consumer-owned account identifier.
    pub account: u64,
    /// Application revision, incremented by the handler.
    pub revision: u64,
    /// Opaque bytes echoed without modification.
    pub payload: Blob,
}

thread_local! { static COUNTERS: RefCell<[u64; 3]> = const { RefCell::new([0; 3]) }; }

#[cfg(any(
    test,
    feature = "peer",
    feature = "candid-codec",
    not(feature = "runtime-benchmark")
))]
#[derive(Debug)]
enum CodecError {
    Encode,
    Decode,
    Trailing,
    Identity,
    Noncanonical,
    Limit,
}

#[cfg(any(
    test,
    feature = "peer",
    feature = "candid-codec",
    not(feature = "runtime-benchmark")
))]
fn encode<T: Serialize + CandidType>(value: &T) -> Result<Vec<u8>, CodecError> {
    #[cfg(feature = "candid-codec")]
    let bytes = candid::encode_one(value).map_err(|_| CodecError::Encode)?;
    #[cfg(not(feature = "candid-codec"))]
    let bytes = {
        let mut buffer = [0; MAX_WIRE];
        postcard::to_slice(value, &mut buffer)
            .map_err(|_| CodecError::Encode)?
            .to_vec()
    };
    if bytes.len() > MAX_WIRE {
        return Err(CodecError::Limit);
    }
    Ok(bytes)
}

#[cfg(any(
    test,
    feature = "peer",
    feature = "candid-codec",
    not(feature = "runtime-benchmark")
))]
fn decode_value<T: serde::de::DeserializeOwned + Serialize + CandidType>(
    bytes: &[u8],
) -> Result<T, CodecError> {
    if bytes.len() > MAX_WIRE {
        return Err(CodecError::Limit);
    }
    #[cfg(feature = "candid-codec")]
    let value = {
        let mut config = candid::de::DecoderConfig::new();
        config
            .set_decoding_quota(100_000)
            .set_skipping_quota(1000)
            .set_max_type_len(64)
            .set_max_header_len(1024);
        let mut decoder = candid::de::IDLDeserialize::new_with_config(bytes, &config)
            .map_err(|_| CodecError::Decode)?;
        let value = decoder.get_value::<T>().map_err(|_| CodecError::Decode)?;
        decoder.done().map_err(|_| CodecError::Trailing)?;
        value
    };
    #[cfg(not(feature = "candid-codec"))]
    let value = {
        let (value, rest) =
            postcard::take_from_bytes::<T>(bytes).map_err(|_| CodecError::Decode)?;
        if !rest.is_empty() {
            return Err(CodecError::Trailing);
        }
        value
    };
    if encode(&value)? != bytes {
        return Err(CodecError::Noncanonical);
    }
    Ok(value)
}

#[cfg(any(feature = "peer", not(feature = "runtime-benchmark")))]
fn decode(bytes: &[u8]) -> Result<Message, CodecError> {
    let value: Message = decode_value(bytes)?;
    if value.contract != CONTRACT || value.direction != 0 {
        return Err(CodecError::Identity);
    }
    Ok(value)
}

#[cfg(any(feature = "peer", not(feature = "runtime-benchmark")))]
#[ic_cdk::update(manual_reply = true)]
fn evaluate() {
    if ic0::msg_arg_data_size() > MAX_WIRE {
        ic_cdk::api::msg_reject("wire limit");
        return;
    }
    let bytes = ic_cdk::api::msg_arg_data();
    let start = ic_cdk::api::instruction_counter();
    let mut value = match decode(&bytes) {
        Ok(value) => value,
        Err(error) => {
            ic_cdk::api::msg_reject(format!("{error:?}"));
            return;
        }
    };
    #[cfg(feature = "peer")]
    peer::observe(&bytes);
    let decoded = ic_cdk::api::instruction_counter();
    value.direction = 1;
    value.revision = value.revision.wrapping_add(1);
    let handled = ic_cdk::api::instruction_counter();
    let reply = encode(&value).expect("finite reply");
    let encoded = ic_cdk::api::instruction_counter();
    COUNTERS.with_borrow_mut(|counts| {
        *counts = [decoded - start, handled - decoded, encoded - handled]
    });
    ic_cdk::api::msg_reply(reply);
}

#[ic_cdk::query(manual_reply = true)]
fn measurements() {
    let mut bytes = [0; 24];
    COUNTERS.with_borrow(|counts| {
        for (chunk, value) in bytes.as_chunks_mut::<8>().0.iter_mut().zip(counts) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
    });
    ic_cdk::api::msg_reply(bytes);
}

// Real external Candid traffic, identically linked and exercised in both
// management-enabled artifacts. It is outside the measured codec regions.
#[cfg(feature = "management")]
#[ic_cdk::update(manual_reply = true)]
async fn management_probe() {
    let response =
        ic_cdk::call::Call::unbounded_wait(candid::Principal::management_canister(), "raw_rand")
            .with_args(&())
            .await
            .expect("raw_rand call");
    let random: Vec<u8> = response.candid().expect("raw_rand decoding");
    assert_eq!(random.len(), 32);
    ic_cdk::api::msg_reply(b"ok");
}
