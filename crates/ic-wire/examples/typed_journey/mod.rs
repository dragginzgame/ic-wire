//! Native evaluation of explicit layouts and a typed method binding, not an IC adapter.
use serde::{Deserialize, Serialize, de::DeserializeOwned};

const CAPACITY: usize = 64;
const BALANCE_ID: u32 = 0x4241;
const BALANCE_NAME: &str = "balance";

// The tuple conversion owns wire order; Rust field order does not.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(into = "(u64, u64)", from = "(u64, u64)")]
struct Request {
    account: u64,
    at_revision: u64,
}
impl From<Request> for (u64, u64) {
    fn from(value: Request) -> Self {
        (value.account, value.at_revision)
    }
}
impl From<(u64, u64)> for Request {
    fn from((account, at_revision): (u64, u64)) -> Self {
        Self {
            account,
            at_revision,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(into = "(u8, u64)", try_from = "(u8, u64)")]
enum Outcome {
    Balance(u64),
    NotFound,
}
impl From<Outcome> for (u8, u64) {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Balance(amount) => (0, amount),
            Outcome::NotFound => (1, 0),
        }
    }
}
impl TryFrom<(u8, u64)> for Outcome {
    type Error = &'static str;
    fn try_from(value: (u8, u64)) -> Result<Self, Self::Error> {
        match value {
            (0, amount) => Ok(Self::Balance(amount)),
            (1, 0) => Ok(Self::NotFound),
            _ => Err("invalid outcome tag or payload"),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Error {
    Limit,
    Decode,
    Trailing,
    Identity,
    NonCanonical,
    Transport,
}

// Owning immutable bytes lets a caller retry without encoding again.
struct Prepared {
    bytes: [u8; CAPACITY],
    len: usize,
}
impl Prepared {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

fn encode<T: Serialize>(direction: u8, value: &T) -> Result<Prepared, Error> {
    let mut bytes = [0; CAPACITY];
    let len = postcard::to_slice(&(BALANCE_ID, direction, value), &mut bytes)
        .map_err(|_| Error::Limit)?
        .len();
    Ok(Prepared { bytes, len })
}

fn decode<T: Serialize + DeserializeOwned>(direction: u8, bytes: &[u8]) -> Result<T, Error> {
    if bytes.len() > CAPACITY {
        return Err(Error::Limit);
    }
    // Validate the contract and direction before asking the body to decode.
    let (id, rest) = postcard::take_from_bytes::<u32>(bytes).map_err(|_| Error::Decode)?;
    let (actual_direction, _) = postcard::take_from_bytes::<u8>(rest).map_err(|_| Error::Decode)?;
    if id != BALANCE_ID || actual_direction != direction {
        return Err(Error::Identity);
    }
    let ((_, _, value), rest): ((u32, u8, T), _) =
        postcard::take_from_bytes(bytes).map_err(|_| Error::Decode)?;
    if !rest.is_empty() {
        return Err(Error::Trailing);
    }
    if encode(direction, &value)?.as_bytes() != bytes {
        return Err(Error::NonCanonical);
    }
    Ok(value)
}

// One concrete binding owns the method name, layout identity and both types.
struct BalanceMethod;
impl BalanceMethod {
    fn prepare(request: &Request) -> Result<Prepared, Error> {
        encode(0, request)
    }
    fn dispatch(bytes: &[u8], handler: impl FnOnce(Request) -> Outcome) -> Result<Prepared, Error> {
        let request = decode::<Request>(0, bytes)?;
        encode(1, &handler(request))
    }
    fn call(
        request: &Prepared,
        transport: impl FnOnce(&str, &[u8]) -> Result<Prepared, Error>,
    ) -> Result<Outcome, Error> {
        let reply = transport(BALANCE_NAME, request.as_bytes())?;
        decode(1, reply.as_bytes())
    }
}

fn balance(request: Request) -> Outcome {
    if request.account == 7 && request.at_revision == 42 {
        Outcome::Balance(1_000_000)
    } else {
        Outcome::NotFound
    }
}

fn main() {
    let prepared = BalanceMethod::prepare(&Request {
        account: 7,
        at_revision: 42,
    })
    .unwrap();
    let outcome = BalanceMethod::call(&prepared, |name, bytes| {
        if name != BALANCE_NAME {
            return Err(Error::Transport);
        }
        BalanceMethod::dispatch(bytes, balance)
    })
    .unwrap();
    assert_eq!(outcome, Outcome::Balance(1_000_000));
    println!(
        "Native typed balance journey passed; request={} bytes; no IC transport exercised",
        prepared.len
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_round_trip_and_application_error() {
        for (account, expected) in [(7, Outcome::Balance(1_000_000)), (9, Outcome::NotFound)] {
            let request = BalanceMethod::prepare(&Request {
                account,
                at_revision: 42,
            })
            .unwrap();
            assert_eq!(
                BalanceMethod::call(&request, |_, bytes| BalanceMethod::dispatch(bytes, balance)),
                Ok(expected)
            );
        }
    }
    #[test]
    fn retry_retains_exact_request_and_separates_transport_failure() {
        let request = BalanceMethod::prepare(&Request {
            account: 7,
            at_revision: 42,
        })
        .unwrap();
        let retained = request.as_bytes().to_vec();
        assert_eq!(
            BalanceMethod::call(&request, |_, bytes| {
                assert_eq!(bytes, retained);
                Err(Error::Transport)
            }),
            Err(Error::Transport)
        );
        assert_eq!(
            BalanceMethod::call(&request, |_, bytes| {
                assert_eq!(bytes, retained);
                BalanceMethod::dispatch(bytes, balance)
            }),
            Ok(Outcome::Balance(1_000_000))
        );
    }
    #[test]
    fn rejects_identity_direction_and_malformed_input_before_handler() {
        assert!(matches!(
            encode(0, &&[0u8; CAPACITY + 1][..]),
            Err(Error::Limit)
        ));
        let request = BalanceMethod::prepare(&Request {
            account: 7,
            at_revision: 42,
        })
        .unwrap();
        let mut wrong = request.as_bytes().to_vec();
        wrong[0] ^= 1;
        assert_eq!(decode::<Request>(0, &wrong), Err(Error::Identity));
        assert_eq!(
            decode::<Outcome>(1, request.as_bytes()),
            Err(Error::Identity)
        );
        for end in 0..request.len {
            assert!(
                BalanceMethod::dispatch(&request.as_bytes()[..end], |_| panic!("handler invoked"))
                    .is_err()
            );
        }
        let mut trailing = request.as_bytes().to_vec();
        trailing.push(0);
        assert_eq!(decode::<Request>(0, &trailing), Err(Error::Trailing));
        assert_eq!(decode::<Request>(0, &[0; CAPACITY + 1]), Err(Error::Limit));
        let invalid = encode(1, &(2u8, 0u64)).unwrap();
        assert_eq!(decode::<Outcome>(1, invalid.as_bytes()), Err(Error::Decode));
    }
    #[test]
    fn explicit_layout_survives_field_reordering_and_freezes_bytes() {
        #[derive(Serialize)]
        #[serde(into = "(u64, u64)")]
        #[derive(Clone)]
        struct Reordered {
            at_revision: u64,
            account: u64,
        }
        impl From<Reordered> for (u64, u64) {
            fn from(value: Reordered) -> Self {
                (value.account, value.at_revision)
            }
        }
        let original = BalanceMethod::prepare(&Request {
            account: 7,
            at_revision: 42,
        })
        .unwrap();
        let reordered = encode(
            0,
            &Reordered {
                at_revision: 42,
                account: 7,
            },
        )
        .unwrap();
        assert_eq!(original.as_bytes(), reordered.as_bytes());
        assert_eq!(original.as_bytes(), [0xc1, 0x84, 0x01, 0, 7, 42]);
        assert_eq!(
            decode::<Request>(0, reordered.as_bytes()).unwrap(),
            Request {
                account: 7,
                at_revision: 42
            }
        );
        let nonminimal = [0xc1, 0x84, 0x01, 0, 0x87, 0, 42];
        assert_eq!(decode::<Request>(0, &nonminimal), Err(Error::NonCanonical));
    }
}
