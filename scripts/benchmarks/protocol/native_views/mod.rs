//! Native API projections plus processing cost; these are not consumer endpoint DTOs.
use super::sample;
use ic_wire::{Bytes, BytesRef, Limits, TextRef, native};
use std::hint::black_box;

const LIMITS: Limits = Limits {
    bytes: 2 * 1024 * 1024,
    // Native admission charges conservatively even when the output borrows.
    allocation: 64 * 1024 * 1024,
    nodes: 4096,
    depth: 32,
};

fn scan(bytes: &[u8]) -> usize {
    black_box(bytes).iter().map(|byte| usize::from(*byte)).sum()
}

pub(super) fn run(directory: &str) {
    for size in [0, 32, 4096, 262144, 1048576] {
        let owned = Bytes::<1048576>::new(vec![42; size]).unwrap();
        let frame = native::encode_request(11, &owned, LIMITS).unwrap();
        let view: BytesRef<'_, 1048576> = native::decode_request(11, &frame, LIMITS).unwrap();
        assert_eq!(view.as_slice(), owned.as_slice());
        assert_eq!(native::encode_request(11, &view, LIMITS).unwrap(), frame);
        if size != 0 {
            assert_eq!(
                view.as_slice().as_ptr(),
                frame[frame.len() - size..].as_ptr()
            );
        }
        std::fs::write(format!("{directory}/native_blob_{size}.bin"), &frame).unwrap();
        for consume in [false, true] {
            let stage = if consume { "decode_scan" } else { "decode" };
            sample(&format!("native_owned_{size}"), stage, frame.len(), || {
                let value: Bytes<1048576> =
                    native::decode_request(11, black_box(&frame), LIMITS).unwrap();
                if consume {
                    scan(value.as_slice())
                } else {
                    black_box(value).as_slice().len()
                }
            });
            sample(
                &format!("native_borrowed_{size}"),
                stage,
                frame.len(),
                || {
                    let value: BytesRef<'_, 1048576> =
                        native::decode_request(11, black_box(&frame), LIMITS).unwrap();
                    if consume {
                        scan(value.as_slice())
                    } else {
                        black_box(value).as_slice().len()
                    }
                },
            );
        }
    }
    for count in [1, 16] {
        let headers: Vec<_> = (0..count)
            .map(|index| (format!("header-{index}"), "v".repeat(128)))
            .collect();
        let frame = native::encode_request(12, &headers, LIMITS).unwrap();
        let view: Vec<(TextRef<'_, 4096>, TextRef<'_, 4096>)> =
            native::decode_request(12, &frame, LIMITS).unwrap();
        assert_eq!(native::encode_request(12, &view, LIMITS).unwrap(), frame);
        for ((name, value), (owned_name, owned_value)) in view.iter().zip(&headers) {
            assert_eq!(name.as_str(), owned_name);
            assert_eq!(value.as_str(), owned_value);
            for text in [name.as_str(), value.as_str()] {
                let offset = text.as_ptr() as usize - frame.as_ptr() as usize;
                assert!(offset + text.len() <= frame.len());
            }
        }
        std::fs::write(format!("{directory}/native_headers_{count}.bin"), &frame).unwrap();
        for consume in [false, true] {
            let stage = if consume { "decode_scan" } else { "decode" };
            sample(
                &format!("headers_owned_{count}"),
                stage,
                frame.len(),
                || {
                    let value: Vec<(String, String)> =
                        native::decode_request(12, black_box(&frame), LIMITS).unwrap();
                    if consume {
                        value
                            .iter()
                            .map(|(k, v)| scan(k.as_bytes()) + scan(v.as_bytes()))
                            .sum()
                    } else {
                        black_box(value).len()
                    }
                },
            );
            sample(
                &format!("headers_borrowed_{count}"),
                stage,
                frame.len(),
                || {
                    let value: Vec<(TextRef<'_, 4096>, TextRef<'_, 4096>)> =
                        native::decode_request(12, black_box(&frame), LIMITS).unwrap();
                    if consume {
                        value
                            .iter()
                            .map(|(k, v)| scan(k.as_str().as_bytes()) + scan(v.as_str().as_bytes()))
                            .sum()
                    } else {
                        black_box(value).len()
                    }
                },
            );
        }
    }
}
