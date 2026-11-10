//! Small frozen contracts shared by native, registry-consumer and installed Wasm checks.
use ic_wire::{ByteBuf, Error, Limits, Packed, native};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt::Debug;

const LIMITS: Limits = Limits {
    bytes: 4096,
    allocation: 65536,
    nodes: 4096,
    depth: 64,
};

fn frozen<T: Serialize + DeserializeOwned + Debug + PartialEq>(value: T, payload: &[u8]) {
    assert_eq!(native::encode(&value, LIMITS).unwrap(), payload);
    assert_eq!(native::decode::<T>(payload, LIMITS).unwrap(), value);
    for (direction, encode) in [
        (0, native::encode_request::<T> as fn(_, &T, _) -> _),
        (1, native::encode_response::<T>),
    ] {
        let mut frame = vec![73, 87, 7, 0, 0, 0, 0, 0, 0, 0, direction];
        frame.extend_from_slice(payload);
        assert_eq!(encode(7, &value, LIMITS).unwrap(), frame);
        let decoded: Result<T, Error> = if direction == 0 {
            native::decode_request(7, &frame, LIMITS)
        } else {
            native::decode_response(7, &frame, LIMITS)
        };
        assert_eq!(decoded.unwrap(), value);
        for end in 0..frame.len() {
            let result = if direction == 0 {
                native::decode_request::<T>(7, &frame[..end], LIMITS)
            } else {
                native::decode_response::<T>(7, &frame[..end], LIMITS)
            };
            assert!(result.is_err());
        }
        frame.push(0);
        let result = if direction == 0 {
            native::decode_request::<T>(7, &frame, LIMITS)
        } else {
            native::decode_response::<T>(7, &frame, LIMITS)
        };
        assert_eq!(result, Err(Error::Trailing));
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Record {
    amount: u16,
    data: ByteBuf,
    maybe: Option<bool>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Choice {
    Empty,
    Value(Record),
    Pair(u8, i16),
}

/// Exact bytes independent of the current Postcard encoder or generated output.
pub fn frozen_contracts() {
    frozen(128_u16, &[128, 1]);
    frozen(-65_i16, &[129, 1]);
    frozen(
        Record {
            amount: 128,
            data: ByteBuf::from(vec![0, 255]),
            maybe: Some(true),
        },
        &[128, 1, 2, 0, 255, 1, 1],
    );
    frozen(Choice::Pair(7, -65), &[2, 7, 129, 1]);
    frozen(
        Packed::new(vec![0_u32, u32::MAX]),
        &[8, 0, 0, 0, 0, 255, 255, 255, 255],
    );
    frozen(vec![Some(0_u64), None, Some(128)], &[3, 1, 0, 0, 1, 128, 1]);
    let owner = ic_platform_types::Principal::from_slice(&[4]);
    for subaccount in [None, Some([0; 32]), Some([255; 32])] {
        let account = ic_platform_types::Account { owner, subaccount };
        let mut payload = vec![1, 4, u8::from(subaccount.is_some())];
        if let Some(bytes) = subaccount {
            payload.push(32);
            payload.extend_from_slice(&bytes);
        }
        frozen(account, &payload);
        let decoded: ic_platform_types::Account = native::decode(&payload, LIMITS).unwrap();
        // Account equality intentionally treats None/zero alike; compare the raw field.
        assert_eq!(decoded.subaccount, subaccount);
    }
    // Core 0.20 follows upstream Postcard: redundant zero (80 00), formerly
    // Malformed, now decodes to zero and re-encodes as 00. Retain original bytes
    // for retry/hash identity; encoded bytes and admission budgets are unchanged.
    assert_eq!(native::decode::<u16>(&[128, 0], LIMITS), Ok(0));
    frozen(0_u16, &[0]);
    for bad in [&[255, 255, 4][..], &[128]] {
        assert_eq!(native::decode::<u16>(bad, LIMITS), Err(Error::Malformed));
    }
    assert_eq!(native::decode::<bool>(&[2], LIMITS), Err(Error::Malformed));
    assert_eq!(
        native::decode::<Option<u8>>(&[2], LIMITS),
        Err(Error::Malformed)
    );
    assert_eq!(
        native::decode::<Choice>(&[3], LIMITS),
        Err(Error::Malformed)
    );
    assert_eq!(
        native::decode::<Packed<u32>>(&[1, 0], LIMITS),
        Err(Error::Malformed)
    );
}

fn allocation_boundary<T: Serialize + DeserializeOwned + Debug>(
    value: T,
    encode_units: usize,
    decode_units: usize,
) {
    let bytes = native::encode(&value, LIMITS).unwrap();
    for (units, encode) in [(encode_units, true), (decode_units, false)] {
        let at = Limits {
            bytes: bytes.len(),
            allocation: bytes.len() + 8 * units,
            ..LIMITS
        };
        let below = Limits {
            allocation: at.allocation - 1,
            ..at
        };
        if encode {
            assert!(native::encode(&value, at).is_ok());
            assert_eq!(native::encode(&value, below), Err(Error::Limit));
        } else {
            assert!(native::decode::<T>(&bytes, at).is_ok());
            assert!(matches!(
                native::decode::<T>(&bytes, below),
                Err(Error::Limit)
            ));
        }
    }
}

/// Independent visitor-charge expectations, executed on each actual target.
pub fn admission_boundaries() {
    // Vec/String headers are three machine words. Freeze both targets explicitly,
    // rather than deriving expected charges from the codec's implementation.
    let header = match usize::BITS {
        32 => 12,
        64 => 24,
        _ => panic!("unqualified pointer width"),
    };
    allocation_boundary(ByteBuf::from(vec![1, 2, 3]), 2 * header + 3, header + 3);
    // Serde's sequence encoder visits references; its charge follows pointer
    // width, while the decoder visits owned u64 values on both targets.
    let element_references = match usize::BITS {
        32 => 16,
        64 => 32,
        _ => unreachable!(),
    };
    allocation_boundary(
        vec![1_u64, 128],
        2 * header + element_references,
        header + 32,
    );
    allocation_boundary(
        vec![vec![1_u64]],
        2 * header + element_references,
        3 * header + 16,
    );
    let raw = [2, b'h', b'i'];
    let borrowed_header = match usize::BITS {
        32 => 8,
        64 => 16,
        _ => unreachable!(),
    };
    let at = Limits {
        bytes: raw.len(),
        allocation: raw.len() + 8 * (borrowed_header + 2),
        ..LIMITS
    };
    let text = native::decode::<&str>(&raw, at).unwrap();
    assert_eq!(text, "hi");
    assert_eq!(text.as_ptr(), raw[1..].as_ptr());
    assert_eq!(
        native::decode::<&str>(
            &raw,
            Limits {
                allocation: at.allocation - 1,
                ..at
            }
        ),
        Err(Error::Limit)
    );
    for (nodes, depth, accepted) in [(4, 3, true), (3, 3, false), (4, 2, false)] {
        assert_eq!(
            native::encode(
                &7_u64,
                Limits {
                    nodes,
                    depth,
                    ..LIMITS
                }
            )
            .is_ok(),
            accepted
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn frozen_bytes() {
        super::frozen_contracts();
    }
    #[test]
    fn exact_admission() {
        super::admission_boundaries();
    }
}
