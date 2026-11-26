//! Stock Serde projections used by the default `Wire` derive.

use super::{ReadContext, WriteContext, decode_projected, encode_projected_into};
use crate::{Encoding, Error, Limits};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

mod containers;

/// A leaf that uses its ordinary Serde representation inside evolving values.
///
/// Derives and the built-in fields supply this marker. The complete Serde value
/// is a fixed contract, including any nested contents it serializes itself.
pub trait Fixed {}

/// Compose a typed value with the Serde projections of its children.
///
/// The derive builds a transport view without replacing the application's own
/// Serde implementations. Borrowed reader and writer views retain their storage.
pub trait Inline<'de>: Sized {
    /// The staged Serde value and its borrowed application data or scratch.
    type Encoded<'s>: Serialize
    where
        Self: 's;

    /// The raw Serde value to project with the enclosing read context.
    type Decoded: Deserialize<'de>;

    /// Stage this value and return the unused suffix of the supplied scratch.
    ///
    /// # Errors
    /// Propagates resource, scratch and nested value serialization failures.
    fn prepare<'s>(
        &'s self,
        context: &WriteContext<'_>,
        scratch: &'s mut [u8],
    ) -> Result<(Self::Encoded<'s>, &'s mut [u8]), Error>;

    /// Validate nested envelopes and construct the application value.
    ///
    /// # Errors
    /// Propagates structural, compatibility and resource failures.
    fn project(input: Self::Decoded, context: &ReadContext<'_>) -> Result<Self, Error>;
}

impl<'de, T: Fixed + Serialize + Deserialize<'de>> Inline<'de> for T {
    type Encoded<'s>
        = &'s T
    where
        Self: 's;
    type Decoded = T;

    fn prepare<'s>(
        &'s self,
        _: &WriteContext<'_>,
        scratch: &'s mut [u8],
    ) -> Result<(Self::Encoded<'s>, &'s mut [u8]), Error> {
        Ok((self, scratch))
    }

    fn project(input: Self::Decoded, _: &ReadContext<'_>) -> Result<Self, Error> {
        Ok(input)
    }
}

/// Record an incompatibility so known siblings still receive validation.
///
/// # Errors
/// Propagates hard failures immediately.
pub fn preserve_incompatible<T>(
    result: Result<T, Error>,
    incompatible: &mut bool,
) -> Result<Option<T>, Error> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(Error::Incompatible) => {
            *incompatible = true;
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

/// Stage and encode an inline value into the existing projected frame path.
///
/// # Errors
/// Returns the first projection, encoding or admission error. Output is empty
/// on failure.
pub fn encode<'de, T: Inline<'de>>(
    value: &T,
    output: &mut Vec<u8>,
    encoding: Encoding,
    limits: Limits,
    scratch: &mut [u8],
) -> Result<(), Error> {
    encode_projected_into(output, encoding, limits, |context| {
        let (value, _) = value.prepare(context, scratch)?;
        Ok(value)
    })
    .map(|_| ())
}

/// Decode and project an inline value with one shared resource meter.
///
/// # Errors
/// Returns a frame, known value, payload or admission error.
pub fn decode<'de, T: Inline<'de>>(
    input: &'de [u8],
    encoding: Encoding,
    limits: Limits,
) -> Result<T, Error> {
    decode_projected(input, encoding, limits, T::project)
}

#[cfg(test)]
mod tests {
    use super::super::{Extensions, ExtensionsWriter};
    use super::*;
    use crate::LimitKind;

    #[derive(Debug, PartialEq, Eq)]
    struct Nested {
        fixed: u8,
        added: u8,
    }

    impl<'de> Inline<'de> for Nested {
        type Encoded<'s> = (u8, Extensions<'s>);
        type Decoded = (u8, Extensions<'de>);

        fn prepare<'s>(
            &'s self,
            context: &WriteContext<'_>,
            scratch: &'s mut [u8],
        ) -> Result<(Self::Encoded<'s>, &'s mut [u8]), Error> {
            context.admit_preparation()?;
            let mut block = ExtensionsWriter::new(scratch);
            block.push(context, 2, &self.added)?;
            let (block, tail) = block.finish_and_tail();
            Ok(((self.fixed, block), tail))
        }

        fn project(
            (fixed, block): Self::Decoded,
            context: &ReadContext<'_>,
        ) -> Result<Self, Error> {
            let mut added = None;
            block.visit(context, |id, value| {
                if id == 2 {
                    added = Some(value.decode()?);
                }
                Ok(())
            })?;
            Ok(Self {
                fixed,
                added: added.ok_or(Error::Incompatible)?,
            })
        }
    }

    #[test]
    fn staged_child_bytes_are_reused_by_following_extension() {
        const LIMITS: Limits = Limits {
            bytes: 32,
            allocation: 2048,
            nodes: 64,
            depth: 8,
        };
        let child = Nested {
            fixed: 42,
            added: 17,
        };
        let mut output = Vec::new();
        let mut scratch = [0; 10];
        fn stage<'out>(
            output: &'out mut Vec<u8>,
            child: &Nested,
            scratch: &mut [u8],
        ) -> Result<&'out [u8], Error> {
            encode_projected_into(output, Encoding::Payload, LIMITS, |context| {
                let mut block = ExtensionsWriter::new(scratch);
                block.push_projected(context, 1, child)?;
                block.push(context, 5, &9_u8)?;
                Ok(block.finish())
            })
        }
        assert_eq!(
            stage(&mut output, &child, &mut scratch),
            Ok([10, 1, 5, 42, 3, 2, 1, 17, 5, 1, 9].as_slice()),
        );
        let actual = decode_projected(
            &output,
            Encoding::Payload,
            LIMITS,
            |block: Extensions<'_>, context| {
                let mut child = None;
                let mut sibling = None;
                block.visit(context, |id, field| {
                    match id {
                        1 => child = Some(field.project_inline::<Nested>()?),
                        5 => sibling = Some(field.decode::<u8>()?),
                        _ => (),
                    }
                    Ok(())
                })?;
                Ok((child, sibling))
            },
        );
        assert_eq!(
            actual,
            Ok((
                Some(Nested {
                    fixed: 42,
                    added: 17
                }),
                Some(9)
            ))
        );
        assert_eq!(
            stage(&mut output, &child, &mut scratch[..9]),
            Err(Error::Limit(LimitKind::Output)),
        );
        assert!(output.is_empty());
    }
}
