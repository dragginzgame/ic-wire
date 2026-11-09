//! Private research fields: byte-buffer admission through the production codec.
//! No published Packed implementation, schema word or consumer rewrite is added.
use super::*;

pub(super) trait Word:
    bytemuck::NoUninit + Copy + Debug + Serialize + DeserializeOwned + Compared
{
    const WIDTH: usize;
    fn read(bytes: &[u8]) -> Self;
}
macro_rules! float {
    ($ty:ty, $width:expr) => {
        impl Word for $ty {
            const WIDTH: usize = $width;
            fn read(bytes: &[u8]) -> Self {
                Self::from_le_bytes(bytes.try_into().unwrap())
            }
        }
    };
}
float!(f32, 4);
float!(f64, 8);
#[derive(Debug)]
pub(super) struct Bulk<T>(pub Vec<T>);
impl<T: Word> Serialize for Bulk<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // RESEARCH ONLY: float lists change element-count prefixes to byte counts:
        // [1_f32] 01 00 00 80 3f -> 04 00 00 80 3f.
        // Any production adoption needs word/schema/client qualification and
        // consumer review of layout, limits and retained retry/hash bytes.
        // The prototype qualifies the supported little-endian hosts/Wasm only.
        #[cfg(target_endian = "little")]
        {
            serializer.serialize_bytes(bytemuck::must_cast_slice(&self.0))
        }
        #[cfg(target_endian = "big")]
        {
            let _ = serializer;
            Err(serde::ser::Error::custom("LE-only research field"))
        }
    }
}
impl<'de, T: Word> serde::Deserialize<'de> for Bulk<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<T>(std::marker::PhantomData<T>);
        impl<T: Word> serde::de::Visitor<'_> for Visitor<T> {
            type Value = Bulk<T>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("research fixed-width words")
            }
            fn visit_bytes<E: serde::de::Error>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                // Validate width before allocating an output.
                // Native byte-buffer admission has already checked frame/limits.
                if !bytes.len().is_multiple_of(T::WIDTH) {
                    return Err(E::custom("invalid research words"));
                }
                let mut values = Vec::new();
                values
                    .try_reserve_exact(bytes.len() / T::WIDTH)
                    .map_err(|_| E::custom("allocation"))?;
                values.extend(bytes.chunks_exact(T::WIDTH).map(T::read));
                Ok(Bulk(values))
            }
        }
        deserializer.deserialize_bytes(Visitor(std::marker::PhantomData))
    }
}
pub(super) fn same<T: Word>(a: &[T], b: &[T]) -> bool {
    bytemuck::must_cast_slice::<T, u8>(a) == bytemuck::must_cast_slice::<T, u8>(b)
}

#[derive(Debug, PartialEq, Serialize, serde::Deserialize)]
#[serde(transparent)]
pub(super) struct BoolValue(#[serde(with = "ic_wire::serde_packed::bools")] pub Vec<bool>);

pub(super) fn bool_case(name: String, values: Vec<bool>) -> Case {
    let input = serde_json::to_vec(bytemuck::must_cast_slice::<bool, u8>(&values)).unwrap();
    #[cfg(not(feature = "research-bulk"))]
    let wire = values.clone();
    #[cfg(feature = "research-bulk")]
    let wire = BoolValue(values.clone());
    paired_values(
        name,
        wire,
        #[cfg(feature = "candid")]
        values,
        input,
    )
}

pub(super) fn case<T: Word>(name: String, values: Vec<T>) -> Case {
    // IEEE bits, rather than JSON nulls or PartialEq, identify non-finite inputs.
    let input_json = serde_json::to_vec(bytemuck::must_cast_slice::<T, u8>(&values)).unwrap();
    #[cfg(all(feature = "wire", not(feature = "research-bulk")))]
    let wire = values.clone();
    #[cfg(all(feature = "wire", feature = "research-bulk"))]
    let wire = Bulk(values.clone());
    Case {
        name,
        input_json,
        lanes: vec![
            #[cfg(feature = "wire")]
            measured_lane(
                "wire-native",
                wire,
                |v| ic_wire::native::encode_request(METHOD, v, LIMITS).unwrap(),
                |b| ic_wire::native::decode_request(METHOD, b, LIMITS).ok(),
                |a, b| {
                    #[cfg(not(feature = "research-bulk"))]
                    {
                        same(a, b)
                    }
                    #[cfg(feature = "research-bulk")]
                    {
                        same(&a.0, &b.0)
                    }
                },
            ),
            #[cfg(feature = "candid")]
            measured_lane(
                "candid",
                values,
                |v| candid::encode_one(v).unwrap(),
                |b| crate::fixtures::decode_candid(b).ok(),
                |a, b| same(a, b),
            ),
        ],
    }
}

macro_rules! finite_field {
    ($name:ident, $case:ident, $ty:ty, $adapter:literal) => {
        #[derive(Debug, Serialize, serde::Deserialize)]
        #[serde(transparent)]
        struct $name(#[serde(with = $adapter)] Vec<$ty>);
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                same(&self.0, &other.0)
            }
        }
        pub(super) fn $case(name: String, values: Vec<$ty>) -> Case {
            let input = serde_json::to_vec(bytemuck::must_cast_slice::<$ty, u8>(&values)).unwrap();
            paired_values(
                name,
                $name(values.clone()),
                #[cfg(feature = "candid")]
                values,
                input,
            )
        }
    };
}
finite_field!(
    FiniteF32Values,
    finite_f32_case,
    f32,
    "ic_wire::serde_packed::finite_f32"
);
finite_field!(
    FiniteF64Values,
    finite_f64_case,
    f64,
    "ic_wire::serde_packed::finite_f64"
);
