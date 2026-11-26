//! Same-revision positional control for the nested evolution workload.
use serde::{Deserialize, Serialize};
use wire::{BytesRef, Direction, Encoding, Error, Limits, List, Packed, TextRef};

pub(super) const CASES: [(&str, u8, usize, bool); 8] = [
    ("empty_a", 0, 0, false),
    ("single_a", 0, 1, false),
    ("unchanged16_a", 0, 16, false),
    ("sparse128_b", 1, 128, false),
    ("dense128_b", 1, 128, true),
    ("sparse1024_b", 1, 1_024, false),
    ("dense1024_b", 1, 1_024, true),
    ("required128_c", 2, 128, false),
];

pub(super) const LIMITS: Limits = Limits {
    bytes: 65_536,
    allocation: 16_777_216,
    nodes: 200_000,
    depth: 32,
};

pub(super) const REQUEST: Encoding = Encoding::Frame {
    method: 7,
    direction: Direction::Request,
};

#[derive(Serialize, Deserialize)]
pub(super) struct Batch<'a, R> {
    pub(super) groups: List<Group<R>, 4>,
    #[serde(borrow)]
    pub(super) suffix: TextRef<'a, 32>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct Group<R> {
    pub(super) channel: u16,
    pub(super) rows: List<R, 512>,
    pub(super) sequence: u64,
}

#[derive(Serialize, Deserialize)]
pub(super) enum StateA<'a> {
    Idle,
    Detail(#[serde(borrow)] TextRef<'a, 32>),
}

#[derive(Serialize, Deserialize)]
pub(super) enum StateC<'a> {
    Idle,
    Detail {
        #[serde(borrow)]
        message: TextRef<'a, 32>,
        #[serde(borrow)]
        audit: Option<BytesRef<'a, 16>>,
    },
    Archived(#[serde(borrow)] TextRef<'a, 32>),
}

#[derive(Serialize, Deserialize)]
pub(super) struct RowA<'a> {
    pub(super) id: u32,
    #[serde(borrow)]
    pub(super) state: Option<StateA<'a>>,
    pub(super) words: Packed<i16, 8>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct RowB<'a> {
    pub(super) id: u32,
    #[serde(borrow)]
    pub(super) state: Option<StateA<'a>>,
    pub(super) words: Packed<i16, 8>,
    #[serde(borrow)]
    pub(super) note: Option<TextRef<'a, 16>>,
    pub(super) marker: Option<()>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct RowC<'a> {
    pub(super) id: u32,
    #[serde(borrow)]
    pub(super) state: Option<StateC<'a>>,
    pub(super) words: Packed<i16, 8>,
    pub(super) epoch: u32,
    #[serde(borrow)]
    pub(super) note: Option<TextRef<'a, 16>>,
    pub(super) marker: Option<()>,
}

pub(super) fn build<R>(
    count: usize,
    dense: bool,
    make_row: impl Fn(usize, bool) -> Result<R, Error>,
) -> Result<Batch<'static, R>, Error> {
    let boundary = count.div_ceil(2);
    let group = |channel, indices: std::ops::Range<usize>, sequence| -> Result<Group<R>, Error> {
        let rows = indices
            .map(|index| make_row(index, dense))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Group {
            channel,
            rows: List::new(rows)?,
            sequence,
        })
    };
    Ok(Batch {
        groups: List::new(vec![
            group(17, 0..boundary, 1_u64 << 60)?,
            group(33, boundary..count, 257)?,
        ])?,
        suffix: TextRef::new("tail ✅")?,
    })
}

fn state_a(index: usize) -> Result<Option<StateA<'static>>, Error> {
    match index % 4 {
        1 => Ok(Some(StateA::Detail(TextRef::new("steady Δ")?))),
        2 => Ok(Some(StateA::Idle)),
        _ => Ok(None),
    }
}

fn note(index: usize, dense: bool) -> Result<Option<TextRef<'static, 16>>, Error> {
    let present = if dense {
        index % 4 < 2
    } else {
        index.is_multiple_of(16)
    };
    present
        .then(|| {
            TextRef::new(if index.is_multiple_of(4) {
                ""
            } else {
                "low Δ"
            })
        })
        .transpose()
}

fn marker(index: usize, dense: bool) -> Option<()> {
    let present = if dense {
        index % 4 == 2
    } else {
        index % 64 == 2
    };
    present.then_some(())
}

fn words() -> Result<Packed<i16, 8>, Error> {
    Packed::new(vec![i16::MIN, -1, i16::MAX])
}

pub(super) fn row_a(index: usize, _dense: bool) -> Result<RowA<'static>, Error> {
    Ok(RowA {
        id: 120 + index as u32,
        state: state_a(index)?,
        words: words()?,
    })
}

pub(super) fn row_b(index: usize, dense: bool) -> Result<RowB<'static>, Error> {
    Ok(RowB {
        id: 120 + index as u32,
        state: state_a(index)?,
        words: words()?,
        note: note(index, dense)?,
        marker: marker(index, dense),
    })
}

pub(super) fn row_c(index: usize, dense: bool) -> Result<RowC<'static>, Error> {
    let id = 120 + index as u32;
    let state = match index % 4 {
        0 => Some(StateC::Archived(TextRef::new("stopped Δ")?)),
        1 => Some(StateC::Detail {
            message: TextRef::new("steady Δ")?,
            audit: Some(BytesRef::new(&[0, 255, 9, 1])?),
        }),
        2 => Some(StateC::Idle),
        _ => None,
    };
    Ok(RowC {
        id,
        state,
        words: words()?,
        epoch: 16_384 + id,
        note: note(index, dense)?,
        marker: marker(index, dense),
    })
}
