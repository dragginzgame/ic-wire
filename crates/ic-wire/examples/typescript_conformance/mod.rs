//! Generate independent Rust vectors and executable JavaScript conformance checks.
mod large_bytes;
use ic_wire::{Bytes, Limits, List, Method, Text, Wire, encode_request, encode_response};
use std::{
    fmt::Write as _,
    fs,
    io::{BufWriter, Write as _},
    marker::PhantomData,
    path::PathBuf,
};

struct Contract<T>(PhantomData<T>);
impl<T: Wire> Method for Contract<T> {
    type Request = T;
    type Response = T;
    const ID: u64 = 0xfedc_ba98_7654_3210;
    const NAME: &'static str = "conformance";
    const REQUEST_LIMITS: Limits = Limits {
        bytes: 4096,
        allocation: 16384,
        nodes: 512,
        depth: 32,
    };
    const RESPONSE_LIMITS: Limits = Self::REQUEST_LIMITS;
}
#[derive(Wire)]
struct Record {
    #[wire(id = 9)]
    r#type: Text<32>,
    #[wire(id = 1)]
    __proto__: Bytes<8>,
    #[wire(id = 5)]
    optional: Option<Option<()>>,
}
#[derive(Wire)]
enum Choice {
    #[wire(id = 128)]
    Payload(Record),
    #[wire(id = 3)]
    Empty,
}
#[derive(Wire)]
enum BoxedChoice {
    #[wire(id = 128)]
    Payload(Box<Record>),
    #[wire(id = 3)]
    Empty,
}
// Consumer-shaped SQL page, not IcyDB's public DTO or storage representation.
#[derive(Wire)]
enum SqlCell {
    #[wire(id = 1)]
    Null,
    #[wire(id = 2)]
    Signed(i64),
    #[wire(id = 3)]
    Unsigned(u64),
    #[wire(id = 4)]
    Real(ic_wire::FiniteF64),
    #[wire(id = 5)]
    Single(ic_wire::FiniteF32),
    #[wire(id = 6)]
    Text(Text<64>),
    #[wire(id = 7)]
    Blob(Bytes<32>),
    #[wire(id = 8)]
    Boolean(bool),
}
#[derive(Wire)]
struct SqlPage {
    #[wire(id = 1)]
    columns: List<Text<32>, 8>,
    #[wire(id = 2)]
    rows: List<List<SqlCell, 8>, 4>,
    #[wire(id = 3)]
    next: Option<Bytes<16>>,
}
// Native Serde shape oracle, independent of the JavaScript descriptor.
#[derive(serde::Serialize)]
struct NativeCount(u64);
#[derive(serde::Serialize)]
enum NativeNode {
    Leaf(String, NativeCount),
    Branch(Vec<Self>),
}
struct Fixture {
    directory: PathBuf,
    checks: String,
    count: usize,
    mutations: usize,
    mutation_data: BufWriter<fs::File>,
}
impl Fixture {
    fn add_boxed<T: Wire + serde::de::DeserializeOwned>(&mut self, value: T, javascript: &str) {
        let boxed = Box::new(value);
        assert_eq!(
            encode_request::<Contract<T>>(&boxed).unwrap().as_bytes(),
            encode_request::<Contract<Box<T>>>(&boxed)
                .unwrap()
                .as_bytes(),
        );
        assert_eq!(
            ic_wire::typescript::generate::<Contract<T>>().unwrap(),
            ic_wire::typescript::generate::<Contract<Box<T>>>().unwrap(),
        );
        self.add(boxed, javascript);
    }
    fn add<T: Wire + serde::de::DeserializeOwned>(&mut self, value: T, javascript: &str) {
        let name = format!("m{}", self.count);
        fs::write(
            self.directory.join(format!("{name}.ts")),
            ic_wire::typescript::generate::<Contract<T>>().unwrap(),
        )
        .unwrap();
        let request = encode_request::<Contract<T>>(&value).unwrap();
        let response = encode_response::<Contract<T>>(&value).unwrap();
        writeln!(self.checks, "import * as {name} from './compiled/{name}.js';\ncheck({name}, {javascript}, {:?}, {:?});", request.as_bytes(), response.as_bytes()).unwrap();
        assert_eq!(
            ic_wire::encoded_request_len::<Contract<T>>(&value).unwrap(),
            request.as_bytes().len()
        );
        assert_eq!(
            ic_wire::encoded_response_len::<Contract<T>>(&value).unwrap(),
            response.as_bytes().len()
        );
        for (direction, original) in [(0, request.as_bytes()), (1, response.as_bytes())] {
            let mut candidates = Vec::new();
            for offset in 0..original.len() {
                for bit in [1, 128] {
                    let mut bytes = original.to_vec();
                    bytes[offset] ^= bit;
                    candidates.push(bytes);
                }
            }
            // Fixed xorshift stream: reproducible mutation testing, not coverage-guided fuzzing.
            let mut seed = 0x92d6_8ca2_u32;
            for length in 0..64 {
                let mut bytes = original[..11].to_vec();
                for _ in 0..length {
                    seed ^= seed << 13;
                    seed ^= seed >> 17;
                    seed ^= seed << 5;
                    bytes.push(seed as u8);
                }
                candidates.push(bytes);
            }
            for bytes in candidates {
                let accepted = if direction == 0 {
                    ic_wire::decode_request::<Contract<T>>(&bytes).map(|v| {
                        ic_wire::encode_request::<Contract<T>>(&v)
                            .unwrap()
                            .into_bytes()
                    })
                } else {
                    ic_wire::decode_response::<Contract<T>>(&bytes).map(|v| {
                        ic_wire::encode_response::<Contract<T>>(&v)
                            .unwrap()
                            .into_bytes()
                    })
                };
                if let Ok(encoded) = &accepted {
                    assert_eq!(encoded, &bytes);
                }
                serde_json::to_writer(
                    &mut self.mutation_data,
                    &serde_json::json!({
                        "codec": name,
                        "direction": direction,
                        "bytes": bytes,
                        "accepted": accepted.is_ok(),
                    }),
                )
                .unwrap();
                writeln!(self.mutation_data).unwrap();
                self.mutations += 1;
            }
        }
        self.count += 1;
    }
}
fn main() {
    let mut arguments = clap::Command::new("typescript-conformance")
        .arg(
            clap::Arg::new("directory")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .get_matches();
    let directory = arguments.remove_one::<PathBuf>("directory").unwrap();
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("wire-runtime.ts"),
        ic_wire::typescript::RUNTIME,
    )
    .unwrap();
    fs::write(directory.join("package.json"), "{\"type\":\"module\"}").unwrap();
    let mut types = std::collections::BTreeMap::from([(
        "Node".into(),
        serde_json::json!({
            "k":"enum","variants":[
                {"name":"Leaf","id":0,"value":{"k":"tuple","items":[{"k":"text"},{"k":"newtype","item":{"k":"u","bits":64}}]}},
                {"name":"Branch","id":1,"value":{"k":"list","item":{"k":"ref","name":"Node"}}}
            ]
        }),
    )]);
    types.insert("__proto__".into(), serde_json::json!({"k":"unit"}));
    let method = serde_json::json!({"name":"nativeTree","mode":"query","id":"1",
        "request":{"k":"ref","name":"Node"},"response":{"k":"ref","name":"Node"},
        "requestBytes":128,"responseBytes":128,"requestNodes":32,"responseNodes":32,
        "requestDepth":8,"responseDepth":8});
    let mut proto = method.clone();
    proto["name"] = serde_json::json!("prototypeName");
    proto["request"] = serde_json::json!({"k":"ref","name":"__proto__"});
    proto["response"] = proto["request"].clone();
    fs::write(
        directory.join("native.ts"),
        ic_wire::typescript::generate_registry(&types, &[method, proto], "./wire-runtime.js")
            .unwrap(),
    )
    .unwrap();
    let mut f = Fixture {
        mutation_data: BufWriter::new(fs::File::create(directory.join("mutations.jsonl")).unwrap()),
        directory,
        checks: format!(
            "const nativePayload = {:?};\n{}",
            postcard::to_allocvec(&NativeNode::Branch(vec![NativeNode::Leaf(
                "é".into(),
                NativeCount(128)
            )]))
            .unwrap(),
            include_str!("checks.mjs")
        ),
        count: 0,
        mutations: 0,
    };
    f.add((), "null");
    f.add(false, "false");
    f.add(true, "true");
    macro_rules! integers {
        ($($ty:ty => $suffix:literal),*) => {$({
            for value in [<$ty>::MIN, 0, 1, <$ty>::MAX] {
                f.add(value, &format!("{value}{}", $suffix));
            }
        })*};
    }
    integers!(u8 => "", i8 => "", u16 => "", i16 => "", u32 => "", i32 => "", u64 => "n", i64 => "n", u128 => "n", i128 => "n");
    for value in [127_u64, 128, 16383, 16384, 1 << 53] {
        f.add(value, &format!("{value}n"));
    }
    for value in [
        0.0_f32,
        -0.0,
        f32::from_bits(1),
        f32::MIN_POSITIVE,
        f32::MIN,
        f32::MAX,
        0.1,
        -2.5,
    ] {
        f.add(
            ic_wire::FiniteF32::new(value).unwrap(),
            &format!("{:?}", f64::from(value)),
        );
    }
    for value in [
        0.0_f64,
        -0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::MIN,
        f64::MAX,
        0.1,
        -2.5,
    ] {
        f.add(
            ic_wire::FiniteF64::new(value).unwrap(),
            &format!("{value:?}"),
        );
    }
    f.add(None::<Option<()>>, "null");
    f.add(Some(None::<()>), "{some:null}");
    f.add(Some(Some(())), "{some:{some:null}}");
    f.add(Ok::<_, Text<8>>(42_u64), "{ok:42n}");
    f.add(
        Err::<u64, _>(Text::<8>::new("oops".into()).unwrap()),
        "{err:'oops'}",
    );
    f.add([i16::MIN, 0, i16::MAX], "[-32768,0,32767]");
    f.add([0_u8; 0], "[]");
    f.add(
        List::<(), 4>::new(vec![(); 4]).unwrap(),
        "[null,null,null,null]",
    );
    f.add(List::<u64, 4>::new(vec![]).unwrap(), "[]");
    f.add(
        List::<u64, 4>::new(vec![0, 128, u64::MAX]).unwrap(),
        "[0n,128n,18446744073709551615n]",
    );
    f.add(
        Bytes::<8>::new(vec![0, 128, 255]).unwrap(),
        "new Uint8Array([0,128,255])",
    );
    // Retain the retired screen's principal-byte nesting and maximum length.
    // These are bytes, not a replacement for consumer principal validation.
    f.add(
        List::<Bytes<29>, 8>::new(vec![
            Bytes::new(vec![]).unwrap(),
            Bytes::new(vec![2; 10]).unwrap(),
            Bytes::new(vec![3; 29]).unwrap(),
        ])
        .unwrap(),
        "[new Uint8Array([]),new Uint8Array(10).fill(2),new Uint8Array(29).fill(3)]",
    );
    for text in ["", "é🦀", "\u{feff}text", "12345678"] {
        f.add(
            Text::<16>::new(text.into()).unwrap(),
            &serde_json::to_string(text).unwrap(),
        );
    }
    f.add(SqlPage {
        columns: List::new(["null", "signed", "unsigned", "real", "single", "text", "blob", "bool"].into_iter().map(|s| Text::new(s.into()).unwrap()).collect()).unwrap(),
        rows: List::new(vec![List::new(vec![SqlCell::Null, SqlCell::Signed(i64::MIN), SqlCell::Unsigned(u64::MAX), SqlCell::Real(ic_wire::FiniteF64::new(-0.0).unwrap()), SqlCell::Single(ic_wire::FiniteF32::new(0.1).unwrap()), SqlCell::Text(Text::new("é🦀".into()).unwrap()), SqlCell::Blob(Bytes::new(vec![0,255]).unwrap()), SqlCell::Boolean(true)]).unwrap()]).unwrap(),
        next: Some(Bytes::new(vec![1,2]).unwrap()),
    }, "{columns:['null','signed','unsigned','real','single','text','blob','bool'],rows:[[{tag:'Null',value:null},{tag:'Signed',value:-9223372036854775808n},{tag:'Unsigned',value:18446744073709551615n},{tag:'Real',value:-0},{tag:'Single',value:Math.fround(0.1)},{tag:'Text',value:'é🦀'},{tag:'Blob',value:new Uint8Array([0,255])},{tag:'Boolean',value:true}]],next:{some:new Uint8Array([1,2])}}");
    f.add(
        SqlPage {
            columns: List::new(vec![]).unwrap(),
            rows: List::new(vec![]).unwrap(),
            next: None,
        },
        "{columns:[],rows:[],next:null}",
    );
    f.add(Choice::Empty, "{tag:'Empty',value:null}");
    f.add(Choice::Payload(Record { r#type: Text::new("hello".into()).unwrap(), __proto__: Bytes::new(vec![1,2]).unwrap(), optional: Some(None) }), "{tag:'Payload',value:{['__proto__']:new Uint8Array([1,2]),optional:{some:null},type:'hello'}}");
    f.add_boxed((), "null");
    f.add_boxed(42_u64, "42n");
    f.add_boxed(Box::new(42_u64), "42n");
    f.add_boxed(
        Record {
            r#type: Text::new("hello".into()).unwrap(),
            __proto__: Bytes::new(vec![1, 2]).unwrap(),
            optional: Some(None),
        },
        "{['__proto__']:new Uint8Array([1,2]),optional:{some:null},type:'hello'}",
    );
    f.add_boxed(Choice::Empty, "{tag:'Empty',value:null}");
    f.add(BoxedChoice::Payload(Box::new(Record { r#type: Text::new("hello".into()).unwrap(), __proto__: Bytes::new(vec![1,2]).unwrap(), optional: Some(None) })), "{tag:'Payload',value:{['__proto__']:new Uint8Array([1,2]),optional:{some:null},type:'hello'}}");
    f.mutation_data.flush().unwrap();
    let codecs = (0..f.count)
        .map(|index| format!("m{index}"))
        .collect::<Vec<_>>()
        .join(",");
    writeln!(
        f.checks,
        "await checkMutations({{{codecs}}}, {});",
        f.mutations
    )
    .unwrap();
    writeln!(f.checks, "console.log('Rust/TypeScript conformance: {} vectors, both directions, truncation and identity rejection passed');", f.count).unwrap();
    writeln!(f.checks, "console.log('Differential mutations: {} Rust/TypeScript acceptance and canonical re-encode checks passed');", f.mutations).unwrap();
    f.checks.push_str(&large_bytes::generate(&f.directory));
    fs::write(f.directory.join("checks.mjs"), f.checks).unwrap();
}
