//! Explicit synthetic layout: big-endian integers and u32 length prefixes.
//! Independently authored comparator; not IcyDB's storage format.
use crate::{
    Error,
    bounded::{Bytes, List, MAX_BLOB, MAX_TEXT, Text},
    fixtures::*,
};
use ic_principal::Principal;

pub(crate) trait Fixed: Sized {
    fn write(&self, out: &mut Vec<u8>);
    fn read(input: &mut &[u8]) -> Result<Self, Error>;
}
fn take<'a>(input: &mut &'a [u8], n: usize) -> Result<&'a [u8], Error> {
    let value = input.get(..n).ok_or(Error::Decode)?;
    *input = &input[n..];
    Ok(value)
}
macro_rules! integer {
    ($($ty:ty),*) => { $(impl Fixed for $ty {
        fn write(&self, out: &mut Vec<u8>) { out.extend(self.to_be_bytes()); }
        fn read(input: &mut &[u8]) -> Result<Self, Error> {
            Ok(Self::from_be_bytes(take(input, size_of::<Self>())?.try_into().map_err(|_| Error::Decode)?))
        }
    })* };
}
integer!(u8, u16, u32, u64);
impl Fixed for bool {
    fn write(&self, out: &mut Vec<u8>) {
        out.push(u8::from(*self));
    }
    fn read(input: &mut &[u8]) -> Result<Self, Error> {
        match u8::read(input)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Decode),
        }
    }
}
fn write_bytes(bytes: &[u8], out: &mut Vec<u8>) {
    (bytes.len() as u32).write(out);
    out.extend(bytes);
}
fn read_bytes<'a>(input: &mut &'a [u8], max: usize) -> Result<&'a [u8], Error> {
    let n = u32::read(input)? as usize;
    if n > max {
        return Err(Error::Limit);
    }
    take(input, n)
}
impl Fixed for Text {
    fn write(&self, out: &mut Vec<u8>) {
        write_bytes(self.as_str().as_bytes(), out);
    }
    fn read(input: &mut &[u8]) -> Result<Self, Error> {
        Self::new(std::str::from_utf8(read_bytes(input, MAX_TEXT)?).map_err(|_| Error::Decode)?)
    }
}
impl Fixed for Bytes {
    fn write(&self, out: &mut Vec<u8>) {
        write_bytes(self.as_slice(), out);
    }
    fn read(input: &mut &[u8]) -> Result<Self, Error> {
        Self::new(read_bytes(input, MAX_BLOB)?)
    }
}
impl Fixed for Principal {
    fn write(&self, out: &mut Vec<u8>) {
        write_bytes(self.as_slice(), out);
    }
    fn read(input: &mut &[u8]) -> Result<Self, Error> {
        Self::try_from_slice(read_bytes(input, 29)?).map_err(|_| Error::Decode)
    }
}
impl<T: Fixed, const N: usize> Fixed for List<T, N> {
    fn write(&self, out: &mut Vec<u8>) {
        (self.as_slice().len() as u32).write(out);
        for v in self.as_slice() {
            v.write(out);
        }
    }
    fn read(input: &mut &[u8]) -> Result<Self, Error> {
        let n = u32::read(input)? as usize;
        if n > N {
            return Err(Error::Limit);
        }
        let mut values = Vec::new();
        for _ in 0..n {
            values.push(T::read(input)?);
        }
        Self::new(values)
    }
}
macro_rules! record {
    ($name:ty {$($field:ident),*}) => { impl Fixed for $name {
        fn write(&self, out: &mut Vec<u8>) { $(self.$field.write(out);)* }
        fn read(input: &mut &[u8]) -> Result<Self, Error> { Ok(Self { $($field: Fixed::read(input)?),* }) }
    } };
}
record!(Request {
    account,
    include_pending
});
record!(Response { balance, revision });
record!(Scope { target, methods });
record!(Authority {
    subject,
    expires_at,
    scopes
});
record!(BlobMessage { id, payload });
record!(Delivery { recipient, payload });
record!(Batch { sequence, messages });
record!(Ack {
    sequence,
    accepted,
    retry
});
record!(Field {
    id,
    name,
    kind,
    nullable
});
record!(Schema {
    revision,
    entity,
    fields
});
