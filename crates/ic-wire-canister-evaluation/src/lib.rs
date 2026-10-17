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

/// Shared runtime contracts used by the installed adoption trial.
#[cfg(feature = "browser-fixture")]
pub mod commands;

#[cfg(feature = "browser-fixture")]
mod browser;

use candid::CandidType;
use serde::{Deserialize, Serialize, de};
use std::{cell::RefCell, fmt};

/// Maximum accepted transport envelope, checked before copying IC arguments.
pub const MAX_WIRE: usize = 5120;
/// Maximum payload bytes, shared with generated browser contract metadata.
pub const MAX_BLOB: usize = 4096;

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

thread_local! { static COUNTERS: RefCell<[u64; 3]> = const { RefCell::new([0; 3]) }; }

#[cfg(feature = "candid-codec")]
#[derive(Debug)]
enum CodecError {
    Encode,
    Decode,
    Trailing,
    Identity,
    Noncanonical,
    Limit,
}

#[cfg(feature = "candid-codec")]
fn encode<T: Serialize + CandidType>(value: &T) -> Result<Vec<u8>, CodecError> {
    let bytes = candid::encode_one(value).map_err(|_| CodecError::Encode)?;
    if bytes.len() > MAX_WIRE {
        return Err(CodecError::Limit);
    }
    Ok(bytes)
}

#[cfg(feature = "candid-codec")]
fn decode_value<T: serde::de::DeserializeOwned + Serialize + CandidType>(
    bytes: &[u8],
) -> Result<T, CodecError> {
    if bytes.len() > MAX_WIRE {
        return Err(CodecError::Limit);
    }
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
    if encode(&value)? != bytes {
        return Err(CodecError::Noncanonical);
    }
    Ok(value)
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
