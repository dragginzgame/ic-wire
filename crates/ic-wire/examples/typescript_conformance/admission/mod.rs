//! Cross-runtime budgets are independently qualified, not assumed equivalent.
use ic_wire::{
    Limits, Wire, native,
    typescript::{self, SchemaRegistry},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fs, path::Path};

const LIMITS: Limits = Limits {
    bytes: 4096,
    allocation: 1 << 20,
    nodes: 4096,
    depth: 128,
};
#[derive(Serialize, Deserialize, Wire)]
struct Record {
    number: u8,
    optional: Option<u8>,
}
#[derive(Serialize, Deserialize, Wire)]
struct Newtype(u8);
#[derive(Serialize, Deserialize, Wire)]
#[serde(transparent)]
struct Transparent(u8);
#[derive(Serialize, Deserialize, Wire)]
enum Tree {
    Empty,
    Children(Vec<Tree>),
    Optional(Option<Box<Tree>>),
}
#[derive(Serialize, Deserialize, Wire)]
struct Left {
    right: Option<Box<Right>>,
}
#[derive(Serialize, Deserialize, Wire)]
struct Right {
    children: Vec<Left>,
}
#[derive(Serialize, Deserialize, Wire)]
struct Link {
    next: Option<Box<Link>>,
}

fn case<T: typescript::Wire + for<'de> Deserialize<'de>>(name: &str, value: T, js: Value) -> Value {
    let mut registry = SchemaRegistry::new();
    let root = registry.register::<T>("Root").unwrap();
    let types = registry.finish().unwrap();
    let frames = [
        native::encode_request(17, &value, LIMITS).unwrap(),
        native::encode_response(17, &value, LIMITS).unwrap(),
    ];
    assert!(
        native::encode_request(
            17,
            &value,
            Limits {
                allocation: 0,
                ..LIMITS
            }
        )
        .is_err()
    );
    assert!(
        native::decode_request::<T>(
            17,
            &frames[0],
            Limits {
                allocation: 0,
                ..LIMITS
            }
        )
        .is_err()
    );
    let mut rows = Vec::new();
    // Independent budgets in each direction, including exact byte boundaries.
    for direction in 0..2 {
        for (axis, max) in [
            ("nodes", 64),
            ("depth", 32),
            ("bytes", frames[direction].len() + 1),
        ] {
            for limit in if axis == "bytes" {
                frames[direction].len().saturating_sub(1).max(11)..=max
            } else {
                1..=max
            } {
                let mut limits = LIMITS;
                match axis {
                    "nodes" => limits.nodes = limit,
                    "depth" => limits.depth = limit,
                    _ => limits.bytes = limit,
                }
                let budgets = if direction == 0 {
                    typescript::method_limits(limits, LIMITS)
                } else {
                    typescript::method_limits(LIMITS, limits)
                }
                .unwrap();
                let (encoded, decoded) = if direction == 0 {
                    (
                        native::encode_request(17, &value, limits).is_ok(),
                        native::decode_request::<T>(17, &frames[0], limits).is_ok(),
                    )
                } else {
                    (
                        native::encode_response(17, &value, limits).is_ok(),
                        native::decode_response::<T>(17, &frames[1], limits).is_ok(),
                    )
                };
                rows.push(json!({"direction":direction,"axis":axis,"limit":limit,"budgets":budgets,"rustEncode":encoded,"rustDecode":decoded}));
            }
        }
    }
    let mut malformed = Vec::new();
    let mut mutations = Vec::new();
    for (direction, frame) in frames.iter().enumerate() {
        let accepts = |bytes: &[u8]| {
            if direction == 0 {
                native::decode_request::<T>(17, bytes, LIMITS).is_ok()
            } else {
                native::decode_response::<T>(17, bytes, LIMITS).is_ok()
            }
        };
        let mut inputs = Vec::new();
        for end in 0..frame.len() {
            inputs.push(frame[..end].to_vec());
        }
        let mut trailing = frame.clone();
        trailing.push(0);
        inputs.push(trailing);
        let mut header = frame.clone();
        header[0] ^= 1;
        inputs.push(header);
        assert!(inputs.iter().all(|bytes| !accepts(bytes)));
        malformed.push(inputs);
        let mut changed = Vec::new();
        for index in 0..frame.len() {
            for mask in [1, 128] {
                let mut bytes = frame.clone();
                bytes[index] ^= mask;
                changed.push(json!({"accepted":accepts(&bytes),"bytes":bytes}));
            }
        }
        mutations.push(changed);
    }
    json!({"name":name,"root":root,"types":types,"value":js,"frames":frames,"rows":rows,"malformed":malformed,"mutations":mutations})
}

pub(super) fn generate(directory: &Path) -> String {
    benchmark_limits(directory);
    let cases = vec![
        case(
            "recursive-record",
            Link {
                next: Some(Box::new(Link { next: None })),
            },
            json!({"next":{"some":{"next":null}}}),
        ),
        case("scalar", 7_u8, json!(7)),
        case(
            "record",
            Record {
                number: 7,
                optional: Some(9),
            },
            json!({"number":7,"optional":{"some":9}}),
        ),
        case("option", Some(7_u8), json!({"some":7})),
        case("newtype", Newtype(7), json!(7)),
        case("transparent", Transparent(7), json!(7)),
        case(
            "enum-empty",
            Tree::Empty,
            json!({"tag":"Empty","value":null}),
        ),
        case(
            "recursive-empty",
            Tree::Children(vec![]),
            json!({"tag":"Children","value":[]}),
        ),
        case(
            "recursive-list",
            Tree::Children(vec![Tree::Empty]),
            json!({"tag":"Children","value":[{"tag":"Empty","value":null}]}),
        ),
        case(
            "recursive-option",
            Tree::Optional(Some(Box::new(Tree::Empty))),
            json!({"tag":"Optional","value":{"some":{"tag":"Empty","value":null}}}),
        ),
        case(
            "mutual",
            Left {
                right: Some(Box::new(Right {
                    children: vec![Left { right: None }],
                })),
            },
            json!({"right":{"some":{"children":[{"right":null}]}}}),
        ),
        case(
            "bytes",
            ic_wire::ByteBuf::from(vec![0, 127, 255]),
            json!([0, 127, 255]),
        ),
        case(
            "packed",
            ic_wire::Packed::new(vec![i16::MIN, i16::MAX]),
            json!([-32768, 32767]),
        ),
    ];
    fs::write(
        directory.join("admission.json"),
        serde_json::to_vec_pretty(&cases).unwrap(),
    )
    .unwrap();
    // Type-check generated recursive modules as well as exercising raw metadata.
    for (index, case) in cases.iter().enumerate() {
        let mut method = typescript::method_limits(LIMITS, LIMITS).unwrap();
        method.as_object_mut().unwrap().extend(json!({"name":"codec","mode":"query","id":"17","request":case["root"],"response":case["root"]}).as_object().unwrap().clone());
        let types = serde_json::from_value(case["types"].clone()).unwrap();
        fs::write(
            directory.join(format!("admission{index}.ts")),
            typescript::generate_registry(&types, &[method], "./wire-runtime.js").unwrap(),
        )
        .unwrap();
    }
    include_str!("checks.mjs").into()
}

fn benchmark_limits(directory: &Path) {
    use std::{hint::black_box, time::Instant};
    let project = |typed| {
        let request = black_box(LIMITS);
        let response = black_box(LIMITS);
        if typed {
            typescript::method_limits(request, response).unwrap()
        } else {
            json!({"requestBytes":request.bytes,"responseBytes":response.bytes,
            "requestNodes":request.nodes,"responseNodes":response.nodes,
            "requestDepth":request.depth,"responseDepth":response.depth})
        }
    };
    assert_eq!(project(false), project(true));
    let mut rows = Vec::new();
    for sample in 0..7 {
        for typed in if sample % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            black_box(project(typed));
            let start = Instant::now();
            for _ in 0..1000 {
                black_box(project(typed));
            }
            rows.push(json!({"typed":typed,"sample":sample,"iterations":1000,"ns_per_iteration":start.elapsed().as_nanos()/1000}));
        }
    }
    fs::write(
        directory.join("method-limits-host-benchmark.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
