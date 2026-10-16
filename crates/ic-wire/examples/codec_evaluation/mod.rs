//! Payload-size screening only; not a runtime codec or selected wire contract.
mod bounded;
mod fixed;
mod fixtures;

use candid::CandidType;
use fixed::Fixed;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::fmt;

const MAX_WIRE: usize = 65_536;

#[derive(Debug, PartialEq)]
enum Error {
    Limit,
    Encode,
    Decode,
    Trailing,
    NonCanonical,
    Mismatch,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, Serialize)]
enum Codec {
    Candid,
    Postcard,
    Cbor,
    Fixed,
}
const CODECS: [Codec; 4] = [Codec::Candid, Codec::Postcard, Codec::Cbor, Codec::Fixed];

fn encode<T: Serialize + CandidType + Fixed>(codec: Codec, value: &T) -> Result<Vec<u8>, Error> {
    // Only the finite, trusted corpus calls this encoder. The final size check
    // is not an allocation budget for arbitrary user-defined Serialize impls.
    let bytes = match codec {
        Codec::Candid => candid::encode_one(value).map_err(|_| Error::Encode)?,
        Codec::Postcard => postcard::to_allocvec(value).map_err(|_| Error::Encode)?,
        Codec::Cbor => {
            let mut v = Vec::new();
            ciborium::into_writer(value, &mut v).map_err(|_| Error::Encode)?;
            v
        }
        Codec::Fixed => {
            let mut v = Vec::new();
            value.write(&mut v);
            v
        }
    };
    if bytes.len() > MAX_WIRE {
        return Err(Error::Limit);
    }
    Ok(bytes)
}

fn decode<T: Serialize + DeserializeOwned + CandidType + Fixed>(
    codec: Codec,
    bytes: &[u8],
) -> Result<T, Error> {
    if bytes.len() > MAX_WIRE {
        return Err(Error::Limit);
    }
    let mut rest = bytes;
    let value = match codec {
        Codec::Candid => {
            let mut config = candid::de::DecoderConfig::new();
            config
                .set_decoding_quota(100_000)
                .set_skipping_quota(1000)
                .set_max_type_len(64)
                .set_max_header_len(1024);
            let mut decoder = candid::de::IDLDeserialize::new_with_config(bytes, &config)
                .map_err(|_| Error::Decode)?;
            let v = decoder.get_value::<T>().map_err(|_| Error::Decode)?;
            decoder.done().map_err(|_| Error::Trailing)?;
            v
        }
        Codec::Postcard => {
            let (v, r) = postcard::take_from_bytes(bytes).map_err(|_| Error::Decode)?;
            rest = r;
            v
        }
        Codec::Cbor => ciborium::de::from_reader_with_recursion_limit(&mut rest, 16)
            .map_err(|_| Error::Decode)?,
        Codec::Fixed => T::read(&mut rest)?,
    };
    if !matches!(codec, Codec::Candid) && !rest.is_empty() {
        return Err(Error::Trailing);
    }
    if encode(codec, &value)? != bytes {
        return Err(Error::NonCanonical);
    }
    Ok(value)
}

#[derive(Serialize)]
struct Row {
    fixture: &'static str,
    codec: Codec,
    bytes: usize,
    payload_sha256: String,
    value_sha256: String,
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn measure<T: Serialize + DeserializeOwned + CandidType + Fixed + PartialEq>(
    name: &'static str,
    value: &T,
    rows: &mut Vec<Row>,
) -> Result<(), Error> {
    let value_sha256 = hash(&serde_json::to_vec(value).map_err(|_| Error::Encode)?);
    for codec in CODECS {
        let bytes = encode(codec, value)?;
        if encode(codec, value)? != bytes {
            return Err(Error::NonCanonical);
        }
        if decode::<T>(codec, &bytes)? != *value {
            return Err(Error::Mismatch);
        }
        rows.push(Row {
            fixture: name,
            codec,
            bytes: bytes.len(),
            payload_sha256: hash(&bytes),
            value_sha256: value_sha256.clone(),
        });
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let compiler = std::process::Command::new("rustc").arg("-vV").output()?;
    let report = serde_json::json!({
        "scope": "native payload bytes and strict typed round trips; no timing, IC instructions or Wasm size measurement",
        "compiler_observed_at_execution": String::from_utf8(compiler.stdout)?,
        "profile": if cfg!(debug_assertions) { "dev" } else { "release" },
        "target": std::env::consts::ARCH,
        "lock_sha256": hash(include_bytes!("../../../../Cargo.lock")),
        "fixture_source_sha256": hash(include_bytes!("fixtures/mod.rs")),
        "harness_source_sha256": hash(include_bytes!("mod.rs")),
        "bounds_source_sha256": hash(include_bytes!("bounded/mod.rs")),
        "fixed_source_sha256": hash(include_bytes!("fixed/mod.rs")),
        "limits": { "wire_bytes": MAX_WIRE, "text_bytes": 64, "blob_bytes": 4096, "cbor_recursion": 16, "candid_decoding_quota": 100000, "candid_skipping_quota": 1000, "candid_max_types": 64, "candid_max_header_bytes": 1024 },
        "rows": fixtures::corpus()?
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fixtures::Request;
    #[test]
    fn positional_layout_does_not_supply_field_identity() {
        #[derive(Serialize)]
        struct Original {
            account: u64,
            revision: u64,
        }
        #[derive(serde::Deserialize)]
        struct Reordered {
            revision: u64,
            account: u64,
        }
        let bytes = postcard::to_allocvec(&Original {
            account: 7,
            revision: 42,
        })
        .unwrap();
        let decoded: Reordered = postcard::from_bytes(&bytes).unwrap();
        assert_eq!((decoded.account, decoded.revision), (42, 7));
    }
    #[test]
    fn all_fixtures_round_trip() {
        assert_eq!(fixtures::corpus().unwrap().len(), 28);
    }
    #[test]
    fn recorded_payload_vectors_match() {
        let recorded: serde_json::Value =
            serde_json::from_str(include_str!("../../../../docs/evidence/codec-screen.json"))
                .unwrap();
        assert_eq!(
            serde_json::to_value(fixtures::corpus().unwrap()).unwrap(),
            recorded["rows"]
        );
    }
    #[test]
    fn malformed_requests_reject() {
        for codec in CODECS {
            let bytes = encode(
                codec,
                &Request {
                    account: 7,
                    include_pending: true,
                },
            )
            .unwrap();
            for end in 0..bytes.len() {
                assert!(
                    decode::<Request>(codec, &bytes[..end]).is_err(),
                    "{codec:?} truncation {end}"
                );
            }
            let mut trailing = bytes.clone();
            trailing.push(0);
            assert!(decode::<Request>(codec, &trailing).is_err());
            assert_eq!(
                decode::<Request>(codec, &vec![0; MAX_WIRE + 1]),
                Err(Error::Limit)
            );
        }
    }
    #[test]
    fn frozen_request_bytes() {
        let value = Request {
            account: 7,
            include_pending: true,
        };
        assert_eq!(encode(Codec::Postcard, &value).unwrap(), [7, 1]);
        assert_eq!(
            encode(Codec::Fixed, &value).unwrap(),
            [0, 0, 0, 0, 0, 0, 0, 7, 1]
        );
        assert!(decode::<Request>(Codec::Postcard, &[0x87, 0, 1]).is_err());
        assert!(decode::<Request>(Codec::Fixed, &[0, 0, 0, 0, 0, 0, 0, 7, 2]).is_err());
    }
    #[test]
    fn bounded_values_reject_before_copy() {
        assert!(bounded::Text::new(&"x".repeat(65)).is_err());
        assert!(bounded::Bytes::new(&[0; 4097]).is_err());
        assert!(postcard::from_bytes::<bounded::List<u64, 8>>(&[255, 255, 255, 255, 15]).is_err());
        assert!(
            ciborium::from_reader::<bounded::List<u64, 8>, _>(
                &[0x9f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xff][..]
            )
            .is_err()
        );
        assert!(<bounded::Bytes as Fixed>::read(&mut &[255, 255, 255, 255][..]).is_err());
    }
}
