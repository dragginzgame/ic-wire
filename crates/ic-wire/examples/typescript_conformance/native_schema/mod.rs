//! Native declaration projection and exact-frame conformance; no field IDs.
use ic_wire::{
    Limits, Wire, native,
    typescript::{self, GenerationError, SchemaContext},
};
use serde as serde_reexport;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fmt::Write as _, fs, path::Path};

#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
#[serde(transparent)]
struct CowText(std::borrow::Cow<'static, str>);

#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
#[serde(crate = "serde")]
struct Page<T> {
    entries: Vec<T>,
    total: u64,
}
#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
struct User {
    name: String,
    optional: Option<Result<Vec<u16>, String>>,
    key: [u8; 2],
}
#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
struct Count(u64);
#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
struct Blob {
    data: ic_wire::ByteBuf,
}
#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct AdaptedBlob {
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
}
type OptionalBytes = Option<Vec<u8>>;
#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
struct OptionalBlob {
    #[serde(with = "serde_bytes", rename = "cursor")]
    data: OptionalBytes,
}
#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
enum Renamed {
    #[serde(rename = "Int")]
    Int64(i64),
    #[serde(rename = "Record")]
    Named {
        #[serde(rename = "items")]
        values: Vec<Option<u8>>,
    },
}
// Recommended mixed DTO: bulk binary/numeric/hash fields and an ordinary
// collection of labels. This is a new fixture, not a migration of User/Page;
// their ordinary-vector layouts remain compatibility and small-value controls.
#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
struct BulkTransfer {
    payload: ic_wire::Bytes<4096>,
    samples: ic_wire::Packed<i16>,
    chunk_hashes: ic_wire::Packed<[u8; 32]>,
    labels: Vec<String>,
}
#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
#[serde(transparent)]
struct Transparent {
    value: Count,
}
#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
#[serde(crate = "self::serde_reexport")]
enum Choice<T> {
    Unit,
    One(T),
    Pair(String, u16),
    Named { count: Transparent, page: Page<T> },
}
#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
struct Nested {
    pages: Page<Page<User>>,
    choice: Choice<User>,
    network: Network,
}
// Explicit owner contract: the stored bytes serialize as lowercase hex text.
// This is intentionally manual, never inferred from the array's Rust layout.
#[derive(Debug, PartialEq)]
struct Network([u8; 32]);
impl Serialize for Network {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let text: String = self.0.iter().map(|b| format!("{b:02x}")).collect();
        serializer.serialize_str(&text)
    }
}
impl<'de> Deserialize<'de> for Network {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        if text.len() != 64
            || !text
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(serde::de::Error::custom("canonical network hex required"));
        }
        let mut bytes = [0; 32];
        for (byte, pair) in bytes.iter_mut().zip(text.as_bytes().as_chunks::<2>().0) {
            let digit = |b| if b <= b'9' { b - b'0' } else { b - b'a' + 10 };
            *byte = digit(pair[0]) * 16 + digit(pair[1]);
        }
        Ok(Self(bytes))
    }
}
impl typescript::Wire for Network {
    fn native_schema(_: &mut SchemaContext) -> Result<Value, GenerationError> {
        Ok(json!({"k":"text","max":64,"check":"networkHex"}))
    }
}
#[derive(Wire, Serialize)]
enum Recursive {
    End,
    Next(Box<Self>),
}

#[derive(Debug, Deserialize, Wire, PartialEq, Serialize)]
#[serde(transparent)]
struct NonzeroWords(
    #[serde(with = "ic_wire::serde_packed::nonzero_i64")] Vec<std::num::NonZeroI64>,
);

#[derive(Debug, Deserialize, Wire, Serialize)]
#[serde(transparent)]
struct HeapWords(
    #[serde(with = "ic_wire::serde_packed::heap_u64")] std::collections::BinaryHeap<u64>,
);

#[derive(Debug, Deserialize, Wire, Serialize, PartialEq)]
#[serde(transparent)]
struct BoolWords(#[serde(with = "ic_wire::serde_packed::bools")] Vec<bool>);
impl PartialEq for HeapWords {
    fn eq(&self, other: &Self) -> bool {
        self.0.clone().into_sorted_vec() == other.0.clone().into_sorted_vec()
    }
}

#[derive(Debug, Deserialize, Wire, Serialize)]
struct DomainRecord {
    tag: u8,
    #[serde(with = "ic_wire::serde_packed::nonzero_i64")]
    ids: Vec<std::num::NonZeroI64>,
    #[serde(with = "ic_wire::serde_packed::heap_u64")]
    queue: std::collections::BinaryHeap<u64>,
}
impl PartialEq for DomainRecord {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
            && self.ids == other.ids
            && self.queue.clone().into_sorted_vec() == other.queue.clone().into_sorted_vec()
    }
}

const LIMITS: Limits = Limits {
    bytes: 4096,
    allocation: 32768,
    nodes: 512,
    depth: 32,
};
fn add<T: typescript::Wire + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(
    directory: &Path,
    checks: &mut String,
    index: usize,
    value: T,
    javascript: &str,
) {
    let schema = typescript::native_schema::<T>().unwrap();
    let mut method =
        json!({"name":"codec","mode":"query","id":"17", "request":schema,"response":schema});
    method.as_object_mut().unwrap().append(
        typescript::method_limits(LIMITS, LIMITS)
            .unwrap()
            .as_object_mut()
            .unwrap(),
    );
    fs::write(
        directory.join(format!("projection{index}.ts")),
        typescript::generate_registry(
            &BTreeMap::new(),
            &[method],
            "./projection-runtime.js",
            &Default::default(),
            |report| eprint!("{report}"),
        )
        .unwrap(),
    )
    .unwrap();
    let request = native::encode_request(17, &value, LIMITS).unwrap();
    let response = native::encode_response(17, &value, LIMITS).unwrap();
    assert_eq!(
        native::decode_request::<T>(17, &request, LIMITS).unwrap(),
        value
    );
    assert_eq!(
        native::decode_response::<T>(17, &response, LIMITS).unwrap(),
        value
    );
    assert!(native::decode_request::<T>(18, &request, LIMITS).is_err());
    for limits in [
        Limits {
            bytes: 11,
            ..LIMITS
        },
        Limits {
            allocation: 0,
            ..LIMITS
        },
        Limits { nodes: 0, ..LIMITS },
        Limits { depth: 0, ..LIMITS },
    ] {
        assert!(native::decode_request::<T>(17, &request, limits).is_err());
    }
    writeln!(checks, "import {{ codec as projection{index} }} from './compiled/projection{index}.js';\ncheck(projection{index}, {javascript}, {request:?}, {response:?});").unwrap();
}

pub(super) fn generate(directory: &Path) -> String {
    let _ = Recursive::Next(Box::new(Recursive::End));
    assert_eq!(
        typescript::native_schema::<Recursive>(),
        Err(GenerationError::Complexity)
    );
    fs::write(directory.join("projection-runtime.ts"), r#"import { createCodec as base, WireError } from './wire-runtime.js';
export function createCodec(method: unknown, types: unknown) {
  return base(method, types, { networkHex: value => { if (typeof value !== 'string' || !/^[0-9a-f]{64}$/.test(value)) throw new WireError('value'); } });
}
"#).unwrap();
    let mut checks = String::new();
    let user = || User {
        name: "é".into(),
        optional: Some(Ok(vec![0, 128, 65535])),
        key: [1, 255],
    };
    let user_js = "{name:'é',optional:{some:{ok:[0,128,65535]}},key:[1,255]}";
    add(
        directory,
        &mut checks,
        0,
        Page {
            entries: vec![user()],
            total: u64::MAX,
        },
        &format!("{{entries:[{user_js}],total:18446744073709551615n}}"),
    );
    add(
        directory,
        &mut checks,
        1,
        Choice::<User>::Unit,
        "{tag:'Unit',value:null}",
    );
    add(
        directory,
        &mut checks,
        2,
        Choice::One(user()),
        &format!("{{tag:'One',value:{user_js}}}"),
    );
    add(
        directory,
        &mut checks,
        3,
        Choice::<User>::Pair("pair".into(), 128),
        "{tag:'Pair',value:['pair',128]}",
    );
    add(
        directory,
        &mut checks,
        4,
        Choice::<User>::Named {
            count: Transparent { value: Count(128) },
            page: Page {
                entries: vec![],
                total: 0,
            },
        },
        "{tag:'Named',value:{count:128n,page:{entries:[],total:0n}}}",
    );
    add(
        directory,
        &mut checks,
        5,
        Nested {
            pages: Page {
                entries: vec![Page {
                    entries: vec![user()],
                    total: 1,
                }],
                total: 1,
            },
            choice: Choice::Unit,
            network: Network([0xab; 32]),
        },
        &format!(
            "{{pages:{{entries:[{{entries:[{user_js}],total:1n}}],total:1n}},choice:{{tag:'Unit',value:null}},network:'{}'}}",
            "ab".repeat(32)
        ),
    );
    add(
        directory,
        &mut checks,
        6,
        User {
            name: String::new(),
            optional: None,
            key: [0, 0],
        },
        "{name:'',optional:null,key:[0,0]}",
    );
    add(
        directory,
        &mut checks,
        7,
        User {
            name: "error".into(),
            optional: Some(Err("denied".into())),
            key: [0, 0],
        },
        "{name:'error',optional:{some:{err:'denied'}},key:[0,0]}",
    );
    add(
        directory,
        &mut checks,
        8,
        (Count(128), Transparent { value: Count(1) }),
        "[128n,1n]",
    );
    add(
        directory,
        &mut checks,
        9,
        Network([0xab; 32]),
        &format!("'{}'", "ab".repeat(32)),
    );
    for (offset, data) in [vec![], vec![0, 127, 255], vec![42; 128]]
        .into_iter()
        .enumerate()
    {
        let javascript = format!("{{data:Uint8Array.from({data:?})}}");
        add(
            directory,
            &mut checks,
            10 + offset,
            Blob { data: data.into() },
            &javascript,
        );
    }
    add(
        directory,
        &mut checks,
        13,
        Page {
            entries: vec![vec![0_u16, 128, 65535].into_boxed_slice()],
            total: 1,
        },
        "{entries:[[0,128,65535]],total:1n}",
    );
    assert!(serde_json::from_value::<AdaptedBlob>(json!({"data":[],"extra":1})).is_err());
    assert_eq!(
        typescript::native_schema::<AdaptedBlob>().unwrap(),
        typescript::native_schema::<Blob>().unwrap()
    );
    for (offset, data) in [vec![], vec![0, 127, 255], vec![42; 128]]
        .into_iter()
        .enumerate()
    {
        let javascript = format!("{{data:Uint8Array.from({data:?})}}");
        let value = AdaptedBlob { data: data.clone() };
        assert_eq!(
            native::encode_request(17, &value, LIMITS).unwrap(),
            native::encode_request(17, &Blob { data: data.into() }, LIMITS).unwrap()
        );
        add(directory, &mut checks, 14 + offset, value, &javascript);
    }
    for (offset, count) in [0, 1, 32].into_iter().enumerate() {
        let payload = vec![0xab; count];
        let samples: Vec<i16> = (0..count)
            .map(|i| [i16::MIN, -1, 0, 64, i16::MAX][i % 5])
            .collect();
        let chunk_hashes = vec![[0xcd; 32]; count];
        let javascript = format!(
            "{{payload:Uint8Array.from({payload:?}),samples:Int16Array.from({samples:?}),chunk_hashes:new Uint8Array({}).fill(205),labels:['sensor']}}",
            count * 32,
        );
        add(
            directory,
            &mut checks,
            17 + offset,
            BulkTransfer {
                payload: ic_wire::Bytes::new(payload).unwrap(),
                samples: ic_wire::Packed::new(samples),
                chunk_hashes: ic_wire::Packed::new(chunk_hashes),
                labels: vec!["sensor".into()],
            },
            &javascript,
        );
    }
    for (offset, text) in ["", "template-é🦀"].into_iter().enumerate() {
        let borrowed = CowText(std::borrow::Cow::Borrowed(text));
        let owned = CowText(std::borrow::Cow::Owned(text.to_owned()));
        assert_eq!(
            native::encode_request(17, &borrowed, LIMITS).unwrap(),
            native::encode_request(17, &owned, LIMITS).unwrap()
        );
        assert_eq!(
            native::encode_response(17, &borrowed, LIMITS).unwrap(),
            native::encode_response(17, &owned, LIMITS).unwrap()
        );
        let javascript = serde_json::to_string(text).unwrap();
        add(
            directory,
            &mut checks,
            20 + offset * 2,
            borrowed,
            &javascript,
        );
        add(directory, &mut checks, 21 + offset * 2, owned, &javascript);
    }
    for (offset, (value, js)) in [
        (None, "{cursor:null}"),
        (Some(vec![]), "{cursor:{some:new Uint8Array()}}"),
        (
            Some(vec![0, 128, 255]),
            "{cursor:{some:new Uint8Array([0,128,255])}}",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        add(
            directory,
            &mut checks,
            24 + offset,
            OptionalBlob { data: value },
            js,
        );
    }
    add(
        directory,
        &mut checks,
        27,
        Renamed::Int64(-7),
        "{tag:'Int',value:-7n}",
    );
    add(
        directory,
        &mut checks,
        28,
        Renamed::Named {
            values: vec![None, Some(7)],
        },
        "{tag:'Record',value:{items:[null,{some:7}]}}",
    );
    for (offset, values) in [vec![], vec![-1_i64], vec![i64::MIN, 1, i64::MAX]]
        .into_iter()
        .enumerate()
    {
        let js = format!(
            "new BigInt64Array([{}])",
            values
                .iter()
                .map(|v| format!("{v}n"))
                .collect::<Vec<_>>()
                .join(",")
        );
        add(
            directory,
            &mut checks,
            29 + offset,
            NonzeroWords(
                values
                    .into_iter()
                    .map(|v| std::num::NonZeroI64::new(v).unwrap())
                    .collect(),
            ),
            &js,
        );
    }
    for (offset, values) in [vec![], vec![1_u64], vec![0, 7, 7, u64::MAX]]
        .into_iter()
        .enumerate()
    {
        let heap = std::collections::BinaryHeap::from(values);
        let js = format!(
            "new BigUint64Array([{}])",
            heap.as_slice()
                .iter()
                .map(|v| format!("{v}n"))
                .collect::<Vec<_>>()
                .join(",")
        );
        add(directory, &mut checks, 32 + offset, HeapWords(heap), &js);
    }
    add(
        directory,
        &mut checks,
        35,
        DomainRecord {
            tag: 7,
            ids: vec![std::num::NonZeroI64::new(-1).unwrap()],
            queue: std::collections::BinaryHeap::from(vec![7, 0]),
        },
        "{tag:7,ids:new BigInt64Array([-1n]),queue:new BigUint64Array([7n,0n])}",
    );
    for (offset, len) in [0, 1, 127, 128, 256].into_iter().enumerate() {
        let values: Vec<_> = (0..len).map(|i| i % 2 == 1).collect();
        let js = serde_json::to_string(&values).unwrap();
        add(directory, &mut checks, 36 + offset, BoolWords(values), &js);
    }
    checks.push_str(
        r#"
for (const direction of ['Request', 'Response']) {
  const encode = 'encode'+direction, decode = 'decode'+direction;
  const header = [73,87,17,0,0,0,0,0,0,0,direction === 'Request' ? 0 : 1];
  const codec = projection39;
  for (const value of [[0], [1], [null], [undefined], new Array(1), new Uint8Array([1])])
    rejects(() => codec[encode](value), 'value');
  for (let invalid = 2; invalid <= 255; invalid++)
    rejects(() => codec[decode](Uint8Array.from([...header,1,invalid])), 'malformed');
  rejects(() => codec[decode](Uint8Array.from([...header,128,0])), 'malformed');
  rejects(() => codec[decode](Uint8Array.from([...header,2,1])), 'malformed');
  rejects(() => codec[decode](Uint8Array.from([...header,0,0])), 'malformed');
  const frame = codec[encode]([false,true]);
  const values = codec[decode](frame);
  frame.fill(0);
  assert.deepEqual(values, [false,true]);
}
const bulkFlags = createCodec({id:'17',request:{k:'packed',word:'bool'},response:{k:'packed',word:'bool'},requestBytes:4096,responseBytes:4096,requestNodes:1,responseNodes:1,requestDepth:1,responseDepth:1});
const manyFlags = Array.from({length:128}, (_,i) => i % 2 === 1);
assert.deepEqual(bulkFlags.decodeRequest(bulkFlags.encodeRequest(manyFlags)), manyFlags);
"#,
    );
    checks.push_str(
        r#"
for (const direction of ['Request', 'Response']) {
  const nonzero = projection30, heap = projection33;
  const encode = 'encode'+direction, decode = 'decode'+direction;
  rejects(() => nonzero[encode](new BigInt64Array([0n])), 'value');
  rejects(() => nonzero[encode](new BigInt64Array([1n, 0n])), 'value');
  const header = [73,87,17,0,0,0,0,0,0,0,direction === 'Request' ? 0 : 1];
  rejects(() => nonzero[decode](Uint8Array.from([...header,8,0,0,0,0,0,0,0,0])), 'malformed');
  for (const codec of [nonzero, heap]) {
    rejects(() => codec[decode](Uint8Array.from([...header,1,0])), 'malformed');
    rejects(() => codec[decode](Uint8Array.from([...header,8,1])), 'limit');
    rejects(() => codec[decode](Uint8Array.from([...header,0,0])), 'malformed');
  }
  // Browser values preserve received order/duplicates. Rust reconstructs a heap.
  const bytes = Uint8Array.from([...header,24,1,0,0,0,0,0,0,0,7,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0]);
  assert.deepEqual(heap[decode](bytes), new BigUint64Array([1n,7n,1n]));
}
"#,
    );
    checks.push_str(r#"
rejects(() => projection26.encodeRequest({cursor:{some:[0,128,255]}}), 'value');
rejects(() => projection27.encodeRequest({tag:'Int64',value:-7n}), 'value');
rejects(() => projection28.encodeRequest({tag:'Record',value:{values:[]}}), 'value');

rejects(() => projection14.encodeRequest({data:new Uint8Array(),extra:1}), 'value');
for (const bad of ['AB'.repeat(32), 'ab', 'gg'.repeat(32)]) {
  rejects(() => projection9.encodeRequest(bad), 'value');
  const bytes = new TextEncoder().encode(bad);
  rejects(() => projection9.decodeRequest(Uint8Array.from([73,87,17,0,0,0,0,0,0,0,0,bytes.length,...bytes])), 'value');
}
rejects(() => createCodec({id:'17',request:{k:'text',check:'networkHex'},response:{k:'unit'},requestBytes:4096,responseBytes:4096,requestNodes:512,responseNodes:512,requestDepth:32,responseDepth:32}).encodeRequest('ab'.repeat(32)), 'value');
console.log('Wire: 41 Rust/TypeScript exact-frame vectors, custom scalar checks and both directions passed');
"#);
    for bad in ["AB".repeat(32), "ab".into(), "gg".repeat(32)] {
        let frame = native::encode_request(17, &bad, LIMITS).unwrap();
        assert!(native::decode_request::<Network>(17, &frame, LIMITS).is_err());
    }
    checks
}
