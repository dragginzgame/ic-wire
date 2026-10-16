//! Generate independent Rust vectors and executable JavaScript conformance checks.
use ic_wire::{Bytes, Limits, List, Method, Text, Wire, encode_request, encode_response};
use std::{fmt::Write, fs, marker::PhantomData, path::PathBuf};

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
struct Fixture {
    directory: PathBuf,
    checks: String,
    count: usize,
}
impl Fixture {
    fn add<T: Wire>(&mut self, value: T, javascript: &str) {
        let name = format!("m{}", self.count);
        fs::write(
            self.directory.join(format!("{name}.ts")),
            ic_wire::typescript::generate::<Contract<T>>().unwrap(),
        )
        .unwrap();
        let request = encode_request::<Contract<T>>(&value).unwrap();
        let response = encode_response::<Contract<T>>(&value).unwrap();
        writeln!(self.checks, "import * as {name} from './compiled/{name}.js';\ncheck({name}, {javascript}, {:?}, {:?});", request.as_bytes(), response.as_bytes()).unwrap();
        self.count += 1;
    }
}
fn main() {
    let directory = PathBuf::from(std::env::args().nth(1).expect("output directory"));
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("wire-runtime.ts"),
        ic_wire::typescript::RUNTIME,
    )
    .unwrap();
    fs::write(directory.join("package.json"), "{\"type\":\"module\"}").unwrap();
    let mut f = Fixture {
        directory,
        checks: include_str!("checks.mjs").into(),
        count: 0,
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
    for text in ["", "é🦀", "\u{feff}text", "12345678"] {
        f.add(
            Text::<16>::new(text.into()).unwrap(),
            &serde_json::to_string(text).unwrap(),
        );
    }
    f.add(Choice::Empty, "{tag:'Empty',value:null}");
    f.add(Choice::Payload(Record { r#type: Text::new("hello".into()).unwrap(), __proto__: Bytes::new(vec![1,2]).unwrap(), optional: Some(None) }), "{tag:'Payload',value:{['__proto__']:new Uint8Array([1,2]),optional:{some:null},type:'hello'}}");
    writeln!(f.checks, "console.log('Rust/TypeScript conformance: {} vectors, both directions, truncation and identity rejection passed');", f.count).unwrap();
    fs::write(f.directory.join("checks.mjs"), f.checks).unwrap();
}
