//! Allocation counts and semantic parity for the nested evolution workload.
mod control;
#[allow(dead_code, reason = "The control uses the benchmark fixture directly")]
mod fixture;

use control::{Batch, CASES, LIMITS, REQUEST, RowA, RowB, RowC, StateA, StateC};
use fixture::{BatchView, GroupView, State, View};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, Stats, StatsAlloc};
use std::{alloc::System, hint::black_box};

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn state_a<'a>(state: &StateA<'a>) -> State<'a> {
    match state {
        StateA::Idle => State::Idle(()),
        StateA::Detail(message) => State::Detail {
            message: message.as_str(),
            audit: None,
        },
    }
}

fn state_c<'a>(state: &StateC<'a>) -> State<'a> {
    match state {
        StateC::Idle => State::Idle(()),
        StateC::Detail { message, audit } => State::Detail {
            message: message.as_str(),
            audit: audit.map(|value| value.as_slice()),
        },
        StateC::Archived(message) => State::Archived(message.as_str()),
    }
}

fn row_a(row: RowA<'_>) -> View<'_> {
    View {
        id: row.id,
        state: row.state.as_ref().map(state_a),
        words: row.words.into_vec(),
        note: None,
        marker: false,
        epoch: None,
    }
}

fn row_b(row: RowB<'_>) -> View<'_> {
    View {
        id: row.id,
        state: row.state.as_ref().map(state_a),
        words: row.words.into_vec(),
        note: row.note.map(|value| value.as_str()),
        marker: row.marker.is_some(),
        epoch: None,
    }
}

fn row_c(row: RowC<'_>) -> View<'_> {
    View {
        id: row.id,
        state: row.state.as_ref().map(state_c),
        words: row.words.into_vec(),
        note: row.note.map(|value| value.as_str()),
        marker: row.marker.is_some(),
        epoch: Some(row.epoch),
    }
}

fn project<'a, R>(batch: Batch<'a, R>, row_view: impl Fn(R) -> View<'a>) -> BatchView<'a> {
    BatchView {
        groups: batch
            .groups
            .into_vec()
            .into_iter()
            .map(|group| GroupView {
                channel: group.channel,
                rows: group.rows.into_vec().into_iter().map(&row_view).collect(),
                sequence: group.sequence,
            })
            .collect(),
        suffix: batch.suffix.as_str(),
    }
}

fn display(case: &str, format: &str, operation: &str, bytes: usize, stats: Stats) {
    println!(
        "{case},{format},{operation},{bytes},{},{},{},{},{}",
        stats.allocations,
        stats.reallocations,
        stats.deallocations,
        stats.bytes_allocated,
        stats.bytes_reallocated
    );
}

fn main() -> Result<(), wire::Error> {
    println!(
        "case,format,operation,frame_bytes,allocations,reallocations,deallocations,bytes_allocated,bytes_reallocated"
    );
    for (case, revision, count, dense) in CASES {
        macro_rules! measure {
            ($rev:literal, $build:ident, $type:ident, $project:ident) => {{
                #[cfg(feature = "evolution-caller-buffer")]
                let mut sparse = vec![0; fixture::LIMITS.bytes];
                #[cfg(not(feature = "evolution-caller-buffer"))]
                let mut sparse = Vec::new();
                let encode = Region::new(ALLOCATOR);
                #[cfg(feature = "evolution-caller-buffer")]
                let sparse = fixture::encode_batch_to_slice::<$rev>(
                    count,
                    dense,
                    fixture::LIMITS,
                    &mut sparse,
                )?;
                #[cfg(not(feature = "evolution-caller-buffer"))]
                let sparse = {
                    fixture::encode_batch::<$rev>(count, dense, fixture::LIMITS, &mut sparse)?;
                    sparse.as_slice()
                };
                let encode_sparse = encode.change();

                #[cfg(feature = "evolution-caller-buffer")]
                let mut positional = vec![0; LIMITS.bytes];
                #[cfg(not(feature = "evolution-caller-buffer"))]
                let mut positional = Vec::new();
                let encode = Region::new(ALLOCATOR);
                let batch = control::build(count, dense, control::$build)?;
                #[cfg(feature = "evolution-caller-buffer")]
                let positional = wire::encode_to_slice(&batch, REQUEST, LIMITS, &mut positional)?;
                #[cfg(not(feature = "evolution-caller-buffer"))]
                let positional = wire::encode_into(&batch, REQUEST, LIMITS, &mut positional)?;
                drop(batch);
                let encode_positional = encode.change();

                let decode = Region::new(ALLOCATOR);
                let value_sparse = black_box(fixture::decode::<$rev>(sparse, fixture::LIMITS)?);
                let decode_sparse = decode.change();

                let decode = Region::new(ALLOCATOR);
                let value_positional: Batch<'_, $type<'_>> =
                    black_box(wire::decode(positional, REQUEST, LIMITS)?);
                let decode_positional = decode.change();
                let value_sparse = value_sparse.into_view();
                assert_eq!(value_sparse, project(value_positional, $project));

                display(case, "sparse", "encode", sparse.len(), encode_sparse);
                display(case, "sparse", "decode", sparse.len(), decode_sparse);
                display(
                    case,
                    "positional",
                    "encode",
                    positional.len(),
                    encode_positional,
                );
                display(
                    case,
                    "positional",
                    "decode",
                    positional.len(),
                    decode_positional,
                );
                if $rev != 0 {
                    let decode = Region::new(ALLOCATOR);
                    let old = black_box(fixture::decode::<0>(sparse, fixture::LIMITS)?);
                    let decode_old = decode.change();
                    assert_eq!(old.suffix.as_str(), value_sparse.suffix);
                    display(case, "sparse", "old_decode", sparse.len(), decode_old);
                }
            }};
        }
        match revision {
            0 => measure!(0, row_a, RowA, row_a),
            1 => measure!(1, row_b, RowB, row_b),
            _ => measure!(2, row_c, RowC, row_c),
        }
    }
    Ok(())
}
