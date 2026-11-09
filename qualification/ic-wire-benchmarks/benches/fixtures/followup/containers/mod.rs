//! Application container compositions and standard-library admission controls.
use super::*;
use std::collections::BinaryHeap;
#[cfg(feature = "wire")]
use std::num::NonZeroI64;

#[cfg(any(feature = "packed-containers", test))]
mod transport;

fn nonzero_case(n: usize) -> Case {
    let values = (0..n)
        .map(|i| [i64::MIN, -1, i64::MAX][i % 3])
        .collect::<Vec<_>>();
    let input_json = serde_json::to_vec(&values).unwrap();
    Case {
        name: format!("nonzero-i64-list-{n}"),
        input_json,
        lanes: vec![
            #[cfg(feature = "wire")]
            measured_lane(
                "wire-native",
                {
                    let values = values
                        .iter()
                        .map(|&n| NonZeroI64::new(n).unwrap())
                        .collect::<Vec<_>>();
                    #[cfg(feature = "packed-containers")]
                    {
                        transport::adapters::NonzeroValue(values)
                    }
                    #[cfg(not(feature = "packed-containers"))]
                    {
                        values
                    }
                },
                |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
                |b| ic_wire::native::decode_request(METHOD, b, LIMITS).ok(),
                PartialEq::eq,
            ),
            #[cfg(feature = "candid")]
            measured_lane(
                "candid",
                values,
                |v| candid::encode_one(v).unwrap(),
                |b| crate::fixtures::decode_candid::<Vec<i64>>(b).ok(),
                PartialEq::eq,
            ),
        ],
    }
}

fn heap_case(n: usize) -> Case {
    let heap = (0..n)
        .map(|i| {
            if i % 2 == 0 {
                u64::MAX - i as u64
            } else {
                i as u64
            }
        })
        .collect::<BinaryHeap<_>>();
    // Preserve the actual Serde iteration order in the external Candid vector.
    // Heap normalization is consumer-owned; compare contents outside counters.
    let values = heap.iter().copied().collect::<Vec<_>>();
    let input_json = serde_json::to_vec(&values).unwrap();
    #[cfg(not(feature = "wire"))]
    let _ = heap;
    Case {
        name: format!("binary-heap-u64-{n}"),
        input_json,
        lanes: vec![
            #[cfg(feature = "wire")]
            measured_lane(
                "wire-native",
                {
                    #[cfg(feature = "packed-containers")]
                    {
                        transport::adapters::HeapValue(heap)
                    }
                    #[cfg(not(feature = "packed-containers"))]
                    {
                        heap
                    }
                },
                |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
                |b| ic_wire::native::decode_request(METHOD, b, LIMITS).ok(),
                |a, b| {
                    #[cfg(feature = "packed-containers")]
                    {
                        a.0.clone().into_sorted_vec() == b.0.clone().into_sorted_vec()
                    }
                    #[cfg(not(feature = "packed-containers"))]
                    {
                        a.clone().into_sorted_vec() == b.clone().into_sorted_vec()
                    }
                },
            ),
            #[cfg(feature = "candid")]
            measured_lane(
                "candid",
                values,
                |v| candid::encode_one(v).unwrap(),
                |b| crate::fixtures::decode_candid::<Vec<u64>>(b).ok(),
                PartialEq::eq,
            ),
        ],
    }
}

pub(super) fn cases_selected(selected: Option<usize>) -> Vec<Case> {
    let mut out = Vec::new();
    let mut index = 0;
    macro_rules! push {
        ($case:expr) => {{
            if selected.is_none_or(|s| s == index) {
                out.push($case);
            }
            index += 1;
        }};
    }
    for n in [0, 1, 128, 4096] {
        push!({
            let values = (0..n).map(|i| u64::MAX - i as u64).collect::<Vec<_>>();
            let input = serde_json::to_vec(&values).unwrap();
            paired!(
                format!("result-packed-ok-{n}"),
                Ok::<_, ic_wire::ByteBuf>(ic_wire::Packed::new(values.clone())),
                Ok::<_, Blob>(values),
                input
            )
        });
        push!({
            let bytes = data(n);
            let input = serde_json::to_vec(&bytes).unwrap();
            paired!(
                format!("result-blob-err-{n}"),
                Err::<ic_wire::Packed<u64>, _>(ic_wire::ByteBuf::from(bytes.clone())),
                Err::<Vec<u64>, _>(Blob(bytes)),
                input
            )
        });
        push!({
            let bytes = data(n);
            let input = serde_json::to_vec(&bytes).unwrap();
            paired!(
                format!("nested-option-bytes-some-{n}"),
                Some(Some(ic_wire::ByteBuf::from(bytes.clone()))),
                Some(Some(Blob(bytes))),
                input
            )
        });
        let hashes = (0..n)
            .map(|i| {
                let mut hash = [0x95; 32];
                hash[..8].copy_from_slice(&(i as u64).to_le_bytes());
                hash
            })
            .collect::<Vec<_>>();
        let input = serde_json::to_vec(&hashes).unwrap();
        push!(paired!(
            format!("byte-array-hashes-{n}"),
            hashes
                .iter()
                .copied()
                .map(ic_wire::ByteArray::new)
                .collect::<Vec<_>>(),
            hashes
                .iter()
                .copied()
                .map(ExternalBytes32)
                .collect::<Vec<_>>(),
            input.clone()
        ));
        push!(paired!(
            format!("packed-hashes-{n}"),
            ic_wire::Packed::new(hashes.clone()),
            hashes
                .iter()
                .copied()
                .map(ExternalBytes32)
                .collect::<Vec<_>>(),
            input
        ));
        push!(nonzero_case(n));
        push!(heap_case(n));
    }
    push!(paired!(
        "nested-option-bytes-outer-none".into(),
        None::<Option<ic_wire::ByteBuf>>,
        None::<Option<Blob>>,
        b"null".to_vec()
    ));
    push!(paired!(
        "nested-option-bytes-inner-none".into(),
        Some(None::<ic_wire::ByteBuf>),
        Some(None::<Blob>),
        b"[null]".to_vec()
    ));
    assert!(selected.is_none_or(|s| s < index), "unknown container case");
    out
}
