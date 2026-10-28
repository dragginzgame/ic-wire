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
    let method = json!({"name":"codec","mode":"query","id":"17", "request":schema,"response":schema,
        "requestBytes":4096,"responseBytes":4096,"requestNodes":512,"responseNodes":512,"requestDepth":32,"responseDepth":32});
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
    benchmark(directory);
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
console.log('Wire: 29 Rust/TypeScript exact-frame vectors, custom scalar checks and both directions passed');
"#);
    for bad in ["AB".repeat(32), "ab".into(), "gg".repeat(32)] {
        let frame = native::encode_request(17, &bad, LIMITS).unwrap();
        assert!(native::decode_request::<Network>(17, &frame, LIMITS).is_err());
    }
    checks
}

// Compare the former handwritten metadata path with projection, including the
// unchanged renderer. Identical concrete schemas/output; host time only.
fn benchmark(directory: &Path) {
    use std::{hint::black_box, time::Instant};
    fn manual() -> Value {
        json!({"k":"record","fields":[
            {"name":"entries","value":{"k":"list","item":{"k":"u","bits":64}}},
            {"name":"total","value":{"k":"u","bits":64}}
        ]})
    }
    assert_eq!(manual(), typescript::native_schema::<Page<u64>>().unwrap());
    let mut rows = Vec::new();
    for count in [1, 100] {
        let render = |derived: bool| {
            let types: BTreeMap<String, Value> = (0..count)
                .map(|i| {
                    (
                        format!("Page{i}"),
                        if derived {
                            typescript::native_schema::<Page<u64>>().unwrap()
                        } else {
                            manual()
                        },
                    )
                })
                .collect();
            typescript::generate_registry(
                &types,
                &[],
                "./wire-runtime.js",
                &Default::default(),
                |report| eprint!("{report}"),
            )
            .unwrap()
        };
        assert_eq!(render(false), render(true));
        black_box(render(false));
        black_box(render(true));
        for sample in 0..7 {
            for derived in if sample % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let start = Instant::now();
                for _ in 0..100 {
                    black_box(render(derived));
                }
                rows.push(json!({"types":count,"derived":derived,"sample":sample,"iterations":100,"ns_per_iteration":start.elapsed().as_nanos() / 100}));
            }
        }
    }
    fs::write(
        directory.join("native-schema-host-benchmark.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
