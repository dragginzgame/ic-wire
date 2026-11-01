//! Small codec cost controls, retaining empty/singleton and ordinary-list cases.
use super::*;

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
    for value in [0_u64, 127, 128, u64::MAX] {
        push!(case(&format!("scalar-u64-{value}"), value));
    }
    for n in [0, 1, 4096, 1_048_576] {
        push!(case(&format!("blob-{n}"), Blob(data(n))));
    }
    for n in [0, 1, 4096] {
        push!(case(&format!("unit-list-{n}"), vec![(); n]));
    }
    push!(case(
        "empty-record-list-4096",
        (0..4096).map(|_| EmptyRecord {}).collect::<Vec<_>>()
    ));
    push!(case(
        "optional-unit-list-4096",
        (0..4096)
            .map(|i| (i % 2 == 0).then_some(()))
            .collect::<Vec<_>>()
    ));
    push!(case(
        "empty-inner-lists-4096",
        vec![Vec::<u64>::new(); 4096]
    ));
    push!(case("ordinary-i16-wide-16384", vec![i16::MAX; 16384]));
    push!(case("message-128", message(128)));
    for i in 0..4 {
        push!(byte_keys::cases_selected(Some(i)).remove(0));
    }
    assert!(selected.is_none_or(|s| s < index), "unknown cost control");
    out
}
