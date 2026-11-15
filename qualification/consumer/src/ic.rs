//! Private IC counter wrapper; SDK is confined to this optional qualification guest.
#[cfg(target_family = "wasm")]
mod guest {
    use ic_platform_runtime::{cdk, ic0};
    use ic_wire_consumer::{
        Receipt, decode_receipt, decode_upload, encode_receipt, encode_upload, sample_upload,
    };
    use std::hint::black_box;
    const REPETITIONS: usize = 8;
    fn guard() -> Result<(), String> {
        if ic0::msg_arg_data_size() == 0 {
            Ok(())
        } else {
            Err("expected empty counter request".into())
        }
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "The runtime decode hook transfers ownership of the argument buffer"
    )]
    fn decode(bytes: Vec<u8>) {
        assert!(bytes.is_empty());
    }
    fn encode(bytes: Vec<u8>) -> Vec<u8> {
        bytes
    }
    #[cdk::query(
        name = "measure",
        guard = "guard",
        decode_with = "decode",
        encode_with = "encode"
    )]
    fn measure() -> Vec<u8> {
        let request = sample_upload().unwrap();
        let receipt = Receipt {
            revision: 129,
            accepted: true,
        };
        let request_bytes = encode_upload(&request).unwrap();
        let receipt_bytes = encode_receipt(&receipt).unwrap();
        let mut bad = request_bytes.clone();
        bad[0] ^= 1;
        let mut counts = [0_u64; 7];
        for (operation, count) in counts.iter_mut().enumerate() {
            let run = || match operation {
                0 => {
                    black_box(encode_upload(black_box(&request)).unwrap());
                }
                1 => {
                    black_box(decode_upload(black_box(&request_bytes)).unwrap());
                }
                2 => {
                    black_box(encode_receipt(black_box(&receipt)).unwrap());
                }
                3 => {
                    black_box(decode_receipt(black_box(&receipt_bytes)).unwrap());
                }
                4 => {
                    assert!(decode_upload(black_box(&bad)).is_err());
                }
                5 => {
                    assert!(
                        decode_upload(black_box(&request_bytes[..request_bytes.len() - 1]))
                            .is_err()
                    );
                }
                6 => {
                    black_box(&request_bytes);
                }
                _ => unreachable!(),
            };
            run();
            let start = ic0::performance_counter(0);
            for _ in 0..REPETITIONS {
                run();
            }
            *count = ic0::performance_counter(0) - start;
        }
        let mut reply: Vec<u8> = counts.into_iter().flat_map(u64::to_le_bytes).collect();
        reply.extend(request_bytes);
        reply.extend(receipt_bytes);
        reply
    }
}
