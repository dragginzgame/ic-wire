//! Scalar and heterogeneous SQL-shaped performance contracts, owned by this fixture.
use candid::CandidType;
use ic_wire::{Bytes, FiniteF32, FiniteF64, Limits, List, Method, Text, Wire};
use serde::{Deserialize, Serialize};

/// Standalone scalar contract with no collection allowance.
pub struct Scalar;
impl Method for Scalar {
    type Request = u64;
    type Response = u64;
    const ID: u64 = 0x5343414c4152;
    const NAME: &'static str = "benchmark_scalar";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 21,
        allocation: 21,
        nodes: 1,
        depth: 1,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
/// Eight representative SQL cell shapes; not an IcyDB contract.
#[derive(Clone, Debug, PartialEq, Wire)]
pub enum Cell {
    /// SQL null.
    #[wire(id = 1)]
    Null,
    /// Boolean.
    #[wire(id = 2)]
    Bool(bool),
    /// Signed integer.
    #[wire(id = 3)]
    Int(i64),
    /// Unsigned integer.
    #[wire(id = 4)]
    Nat(u64),
    /// Finite single precision.
    #[wire(id = 5)]
    F32(FiniteF32),
    /// Finite double precision.
    #[wire(id = 6)]
    F64(FiniteF64),
    /// Up to 32 UTF-8 bytes.
    #[wire(id = 7)]
    Text(Text<32>),
    /// Up to 32 opaque bytes.
    #[wire(id = 8)]
    Blob(Bytes<32>),
}
/// Finite page with at most eight columns and eight rows.
#[derive(Clone, Debug, PartialEq, Wire)]
pub struct Page {
    /// Column labels, each at most 32 UTF-8 bytes.
    #[wire(id = 1)]
    pub columns: List<Text<32>, 8>,
    /// Rows contain at most eight cells.
    #[wire(id = 2)]
    pub rows: List<List<Cell, 8>, 8>,
    /// Synthetic page counter, incremented by the echo handler.
    #[wire(id = 3)]
    pub next: u64,
}
/// Page echo measures decoding and encoding of the reply-shaped value.
pub struct PageEcho;
impl Method for PageEcho {
    type Request = Page;
    type Response = Page;
    const ID: u64 = 0x50414745454348;
    const NAME: &'static str = "benchmark_page";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 5120,
        allocation: 16384,
        nodes: 256,
        depth: 8,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
/// Ordinary Candid identity wrapper, shared by the two new control endpoints.
#[derive(Clone, Debug, PartialEq, CandidType, Serialize, Deserialize)]
pub struct Frame<T> {
    /// Method discriminator.
    pub method: u64,
    /// Zero request, one reply.
    pub direction: u8,
    /// Logical payload.
    pub value: T,
}
/// Ordinary Candid projection of Cell.
#[derive(Clone, Debug, PartialEq, CandidType, Serialize, Deserialize)]
pub enum CandidCell {
    /// SQL null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Signed integer.
    Int(i64),
    /// Unsigned integer.
    Nat(u64),
    /// Single precision.
    F32(f32),
    /// Double precision.
    F64(f64),
    /// Text.
    Text(String),
    /// Opaque bytes.
    Blob(Vec<u8>),
}
/// Ordinary Candid projection of Page, checked after quota-limited decoding.
#[derive(Clone, Debug, PartialEq, CandidType, Serialize, Deserialize)]
pub struct CandidPage {
    /// Column labels.
    pub columns: Vec<String>,
    /// Heterogeneous rows.
    pub rows: Vec<Vec<CandidCell>>,
    /// Synthetic page counter.
    pub next: u64,
}
impl Page {
    /// Deterministic rows exercise every variant, including both signed zeros.
    pub fn fixture(rows: usize) -> Self {
        let columns = ["null", "bool", "int", "nat", "f32", "f64", "text", "blob"];
        Self {
            columns: List::new(
                columns
                    .into_iter()
                    .map(|s| Text::new(s.into()).unwrap())
                    .collect(),
            )
            .unwrap(),
            rows: List::new(
                (0..rows)
                    .map(|i| {
                        List::new(vec![
                            Cell::Null,
                            Cell::Bool(i % 2 == 0),
                            Cell::Int(i64::MIN + i as i64),
                            Cell::Nat(u64::MAX - i as u64),
                            Cell::F32(FiniteF32::new(i as f32 * -0.25).unwrap()),
                            Cell::F64(FiniteF64::new(i as f64 * -0.125).unwrap()),
                            Cell::Text(Text::new("é".repeat(16)).unwrap()),
                            Cell::Blob(Bytes::new(vec![i as u8; 32]).unwrap()),
                        ])
                        .unwrap()
                    })
                    .collect(),
            )
            .unwrap(),
            next: 42,
        }
    }
    /// Project values outside measured canister regions.
    pub fn candid(&self) -> CandidPage {
        CandidPage {
            columns: self
                .columns
                .as_slice()
                .iter()
                .map(|s| s.as_str().to_owned())
                .collect(),
            rows: self
                .rows
                .as_slice()
                .iter()
                .map(|r| {
                    r.as_slice()
                        .iter()
                        .map(|c| match c {
                            Cell::Null => CandidCell::Null,
                            Cell::Bool(v) => CandidCell::Bool(*v),
                            Cell::Int(v) => CandidCell::Int(*v),
                            Cell::Nat(v) => CandidCell::Nat(*v),
                            Cell::F32(v) => CandidCell::F32(v.get()),
                            Cell::F64(v) => CandidCell::F64(v.get()),
                            Cell::Text(v) => CandidCell::Text(v.as_str().to_owned()),
                            Cell::Blob(v) => CandidCell::Blob(v.as_slice().to_vec()),
                        })
                        .collect()
                })
                .collect(),
            next: self.next,
        }
    }
}
impl CandidPage {
    /// Candid control bounds; quota limits precede these semantic checks.
    pub fn valid(&self) -> bool {
        self.columns.len() <= 8
            && self.columns.iter().all(|s| s.len() <= 32)
            && self.rows.len() <= 8
            && self.rows.iter().all(|r| {
                r.len() <= 8
                    && r.iter().all(|c| match c {
                        CandidCell::F32(v) => v.is_finite(),
                        CandidCell::F64(v) => v.is_finite(),
                        CandidCell::Text(v) => v.len() <= 32,
                        CandidCell::Blob(v) => v.len() <= 32,
                        _ => true,
                    })
            })
    }
}

// Two concrete endpoints share identical measurement boundaries. No conversions
// between Candid DTOs and Wire values occur inside the measured region.
macro_rules! endpoint {
    ($name:ident, $method:ty, $dto:ty, $valid:expr, $value:ident, $handle:block) => {
        #[ic_cdk::update(manual_reply = true)]
        fn $name() {
            // Candid's type table needs a different frame allowance than Wire.
            #[cfg(feature = "candid-codec")]
            let frame_limit = crate::MAX_WIRE;
            #[cfg(not(feature = "candid-codec"))]
            let frame_limit = <$method>::REQUEST_LIMITS.bytes;
            if ic0::msg_arg_data_size() > frame_limit {
                ic_cdk::api::msg_reject("wire limit");
                return;
            }
            let bytes = ic_cdk::api::msg_arg_data();
            let start = ic_cdk::api::instruction_counter();
            #[cfg(feature = "candid-codec")]
            let result = crate::decode_value::<Frame<$dto>>(&bytes).and_then(|v| {
                if v.method != <$method>::ID || v.direction != 0 {
                    return Err(crate::CodecError::Identity);
                }
                if !($valid)(&v.value) {
                    return Err(crate::CodecError::Limit);
                }
                Ok(v)
            });
            #[cfg(not(feature = "candid-codec"))]
            let result = ic_wire::decode_request::<$method>(&bytes);
            let mut value = match result {
                Ok(v) => v,
                Err(e) => {
                    ic_cdk::api::msg_reject(format!("{e:?}"));
                    return;
                }
            };
            let decoded = ic_cdk::api::instruction_counter();
            #[cfg(feature = "candid-codec")]
            let $value = {
                value.direction = 1;
                &mut value.value
            };
            #[cfg(not(feature = "candid-codec"))]
            let $value = &mut value;
            $handle;
            let handled = ic_cdk::api::instruction_counter();
            #[cfg(feature = "candid-codec")]
            let reply = crate::encode(&value).unwrap();
            #[cfg(not(feature = "candid-codec"))]
            let reply = ic_wire::encode_response::<$method>(&value).unwrap();
            let encoded = ic_cdk::api::instruction_counter();
            crate::COUNTERS
                .with_borrow_mut(|v| *v = [decoded - start, handled - decoded, encoded - handled]);
            #[cfg(feature = "candid-codec")]
            ic_cdk::api::msg_reply(&reply);
            #[cfg(not(feature = "candid-codec"))]
            ic_cdk::api::msg_reply(reply.as_bytes());
        }
    };
}
endpoint!(benchmark_scalar, Scalar, u64, |_: &u64| true, v, {
    *v = v.wrapping_add(1);
});
endpoint!(
    benchmark_page,
    PageEcho,
    CandidPage,
    CandidPage::valid,
    v,
    {
        v.next = v.next.wrapping_add(1);
    }
);
