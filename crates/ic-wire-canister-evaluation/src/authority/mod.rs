//! Synthetic authority tree with finite depth and checked collection bounds.
//! Declaration order is this experiment's layout, not a durable schema identity.
use candid::CandidType;
use serde::{Deserialize, Serialize, de};
use std::{fmt, marker::PhantomData};

#[cfg(not(feature = "runtime-benchmark"))]
use super::COUNTERS;
#[cfg(any(test, not(feature = "runtime-benchmark")))]
use super::{CodecError, MAX_WIRE, decode_value, encode};

const CONTRACT: u32 = 0x4155;
const LIMIT: usize = 8;

#[derive(Clone, Debug, PartialEq, Serialize, CandidType)]
#[serde(transparent)]
struct List<T>(Vec<T>);

struct ExcessElement;
impl<'de> Deserialize<'de> for ExcessElement {
    fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
        Err(de::Error::custom("element limit"))
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for List<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>> de::Visitor<'de> for Visitor<T> {
            type Value = List<T>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("at most eight elements")
            }
            fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                if seq.size_hint().is_some_and(|length| length > LIMIT) {
                    return Err(de::Error::custom("element limit"));
                }
                let mut values = Vec::new();
                values.try_reserve_exact(LIMIT).map_err(de::Error::custom)?;
                while values.len() < LIMIT {
                    let Some(value) = seq.next_element()? else {
                        return Ok(List(values));
                    };
                    values.push(value);
                }
                // An unknown-length sequence must fail before decoding element nine.
                let _: Option<ExcessElement> = seq.next_element()?;
                Ok(List(values))
            }
        }
        d.deserialize_seq(Visitor(PhantomData))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
struct Grant {
    target: u64,
    method: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
struct Delegate {
    subject: u64,
    expires_at: Option<u64>,
    grants: List<Grant>,
}

/// Finite request/reply fixture; not an authentication or authorization protocol.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, CandidType)]
pub struct Authority {
    contract: u32,
    direction: u8,
    revision: u64,
    delegates: List<Delegate>,
}
impl Authority {
    /// Build deterministic values after checking both collection dimensions.
    pub fn fixture(delegates: usize, grants: usize) -> Option<Self> {
        if delegates > LIMIT || grants > LIMIT {
            return None;
        }
        Some(Self {
            contract: CONTRACT,
            direction: 0,
            revision: 42,
            delegates: List(
                (0..delegates)
                    .map(|i| Delegate {
                        subject: 1000 + i as u64,
                        expires_at: (i % 2 == 0).then_some(1_800_000_000),
                        grants: List(
                            (0..grants)
                                .map(|j| Grant {
                                    target: 2000 + j as u64,
                                    method: j as u32,
                                })
                                .collect(),
                        ),
                    })
                    .collect(),
            ),
        })
    }
    /// Apply the shared stateless handler; no authorization semantics are modeled.
    pub fn reply(mut self) -> Self {
        self.direction = 1;
        self.revision = self.revision.wrapping_add(1);
        self
    }
}
#[cfg(any(test, not(feature = "runtime-benchmark")))]
fn decode(bytes: &[u8]) -> Result<Authority, CodecError> {
    let value: Authority = decode_value(bytes)?;
    if value.contract != CONTRACT || value.direction != 0 {
        return Err(CodecError::Identity);
    }
    Ok(value)
}
#[cfg(not(feature = "runtime-benchmark"))]
#[ic_cdk::update(manual_reply = true)]
fn evaluate_authority() {
    if ic0::msg_arg_data_size() > MAX_WIRE {
        ic_cdk::api::msg_reject("wire limit");
        return;
    }
    let bytes = ic_cdk::api::msg_arg_data();
    let start = ic_cdk::api::instruction_counter();
    let value = match decode(&bytes) {
        Ok(value) => value,
        Err(error) => {
            ic_cdk::api::msg_reject(format!("{error:?}"));
            return;
        }
    };
    let decoded = ic_cdk::api::instruction_counter();
    let value = value.reply();
    let handled = ic_cdk::api::instruction_counter();
    let reply = encode(&value).expect("finite authority reply");
    let encoded = ic_cdk::api::instruction_counter();
    COUNTERS.with_borrow_mut(|counts| {
        *counts = [decoded - start, handled - decoded, encoded - handled]
    });
    ic_cdk::api::msg_reply(reply);
}

/// Host-only malformed producers: outer/inner list overflow in Postcard and Candid.
#[cfg(not(target_arch = "wasm32"))]
pub fn oversized_requests() -> [(Vec<u8>, Vec<u8>); 2] {
    [true, false].map(|outer| {
        let mut invalid = Authority::fixture(1, 1).unwrap();
        if outer {
            invalid.delegates.0 = vec![invalid.delegates.0[0].clone(); 9];
        } else {
            invalid.delegates.0[0].grants.0 = vec![
                Grant {
                    target: 1,
                    method: 2
                };
                9
            ];
        }
        (
            postcard::to_allocvec(&invalid).unwrap(),
            candid::encode_one(&invalid).unwrap(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_shapes_and_limits() {
        assert!(Authority::fixture(9, 0).is_none());
        assert!(Authority::fixture(0, 9).is_none());
        for (delegates, grants) in [(0, 0), (2, 3), (8, 8)] {
            let value = Authority::fixture(delegates, grants).unwrap();
            let bytes = encode(&value).unwrap();
            assert_eq!(decode(&bytes).unwrap(), value);
            assert!(bytes.len() < MAX_WIRE);
            for end in 0..bytes.len() {
                assert!(decode(&bytes[..end]).is_err());
            }
            let mut trailing = bytes;
            trailing.push(0);
            assert!(decode(&trailing).is_err());
            assert!(decode(&encode(&value.reply()).unwrap()).is_err());
        }
    }

    #[test]
    fn nested_limits_reject_both_codecs() {
        for (postcard, candid) in oversized_requests() {
            assert!(postcard::from_bytes::<Authority>(&postcard).is_err());
            assert!(candid::decode_one::<Authority>(&candid).is_err());
        }
    }

    #[test]
    fn frozen_empty_authority() {
        assert_eq!(
            postcard::to_allocvec(&Authority::fixture(0, 0).unwrap()).unwrap(),
            [0xd5, 0x82, 0x01, 0, 42, 0]
        );
    }

    #[test]
    fn list_limit_precedes_element_decoding() {
        use serde::de::value::{Error, SeqDeserializer};
        use std::cell::Cell;
        thread_local! { static VISITS: Cell<usize> = const { Cell::new(0) }; }
        struct Counted;
        impl<'de> Deserialize<'de> for Counted {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                VISITS.with(|visits| visits.set(visits.get() + 1));
                u8::deserialize(d)?;
                Ok(Self)
            }
        }
        let known = SeqDeserializer::<_, Error>::new([1u8; 9].into_iter());
        assert!(List::<Counted>::deserialize(known).is_err());
        assert_eq!(VISITS.with(Cell::get), 0);
        let mut remaining = 9;
        let unknown = SeqDeserializer::<_, Error>::new(std::iter::from_fn(|| {
            if remaining == 0 {
                None
            } else {
                remaining -= 1;
                Some(1u8)
            }
        }));
        assert!(List::<Counted>::deserialize(unknown).is_err());
        assert_eq!(VISITS.with(Cell::get), 8);
    }
}
