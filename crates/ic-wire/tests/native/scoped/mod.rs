use ic_wire::{Error, Limits, native};
use serde::Serialize;
use std::cell::Cell;

const LIMITS: Limits = Limits {
    bytes: 65536,
    allocation: 4 * 1024 * 1024,
    nodes: 65536,
    depth: 64,
};

#[test]
fn scoped_output_matches_owned_bytes_and_selects_storage_once() {
    fn compare<T: Serialize>(value: T) {
        // Match the root type too: this call lends &T to with_encoded.
        let expected = native::encode(&&value, LIMITS).unwrap();
        for capacity in [0, expected.len().saturating_sub(1), expected.len(), 65536] {
            let mut scratch = vec![0xa5; capacity];
            let address = scratch.as_ptr();
            let length = native::with_encoded(&value, LIMITS, &mut scratch, |bytes| {
                assert_eq!(bytes, expected);
                if !expected.is_empty() {
                    assert_eq!(bytes.as_ptr() == address, capacity >= expected.len());
                }
                bytes.len()
            })
            .unwrap();
            assert_eq!(length, expected.len());
            let untouched = if capacity >= length { length } else { 0 };
            assert!(scratch[untouched..].iter().all(|&byte| byte == 0xa5));
        }
    }
    compare(());
    compare(7_u64);
    compare(vec![u32::MAX; 4096]);
    compare(ic_wire::ByteBuf::from(vec![0xa5; 4096]));
    compare(("é🦀", Some(vec![i16::MIN, 0, i16::MAX])));
}

#[test]
fn scoped_output_keeps_exact_admission_and_never_consumes_refused_values() {
    let exact = Limits {
        bytes: 1,
        allocation: 129,
        nodes: 4,
        depth: 3,
    };
    for capacity in [0, 16] {
        let mut scratch = vec![0xa5; capacity];
        assert_eq!(
            native::with_encoded(7_u64, exact, &mut scratch, |b| b[0]),
            Ok(7)
        );
        for short in [
            Limits { bytes: 0, ..exact },
            Limits {
                allocation: 128,
                ..exact
            },
            Limits { nodes: 3, ..exact },
            Limits { depth: 2, ..exact },
        ] {
            scratch.fill(0xa5);
            assert_eq!(native::encode(&7_u64, short), Err(Error::Limit));
            assert_eq!(
                native::with_encoded(7_u64, short, &mut scratch, |_| panic!("refused output")),
                Err::<(), _>(Error::Limit)
            );
            assert!(scratch.iter().all(|&b| b == 0xa5));
        }
    }
}

#[test]
fn scoped_output_drops_owned_input_before_callback_and_serializes_twice() {
    struct Tracked<'a> {
        calls: &'a Cell<usize>,
        dropped: &'a Cell<bool>,
    }
    impl Serialize for Tracked<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.calls.set(self.calls.get() + 1);
            serializer.serialize_u8(7)
        }
    }
    impl Drop for Tracked<'_> {
        fn drop(&mut self) {
            self.dropped.set(true);
        }
    }
    for capacity in [0, 16] {
        let calls = Cell::new(0);
        let dropped = Cell::new(false);
        let value = Tracked {
            calls: &calls,
            dropped: &dropped,
        };
        native::with_encoded(value, LIMITS, &mut vec![0; capacity], |bytes| {
            assert!(dropped.get());
            assert_eq!(calls.get(), 2);
            assert_eq!(bytes, [7]);
        })
        .unwrap();
    }
}

#[test]
fn scoped_output_never_consumes_failed_second_pass_on_either_destination() {
    struct Changed {
        calls: Cell<usize>,
        error: bool,
    }
    impl Serialize for Changed {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let pass = self.calls.get();
            self.calls.set(pass + 1);
            if pass == 0 {
                serializer.serialize_u8(7)
            } else if self.error {
                Err(serde::ser::Error::custom("second-pass failure"))
            } else {
                serializer.serialize_bytes(&[1, 2, 3])
            }
        }
    }
    for capacity in [0, 32] {
        for error in [false, true] {
            let value = Changed {
                calls: Cell::new(0),
                error,
            };
            assert_eq!(
                native::with_encoded(value, LIMITS, &mut vec![0; capacity], |_| panic!(
                    "partial output"
                )),
                Err::<(), _>(Error::Malformed)
            );
        }
    }
}
