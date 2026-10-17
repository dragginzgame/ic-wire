//! Source-matched IcyDB success projections. See docs/consumer-fixtures.md.
use super::{Contract, Fixture};
use ic_wire::{List, Text, Wire};
use serde::Serialize;

#[derive(Wire)]
struct Count {
    #[wire(id = 1)]
    entity: Text<64>,
    #[wire(id = 2)]
    row_count: u32,
}
#[derive(Wire)]
struct Grouped {
    #[wire(id = 1)]
    entity: Text<64>,
    #[wire(id = 2)]
    columns: List<Text<32>, 4>,
    #[wire(id = 3)]
    rows: List<List<Text<64>, 4>, 4>,
    #[wire(id = 4)]
    row_count: u32,
    #[wire(id = 5)]
    next_cursor: Option<Text<128>>,
}
#[derive(Wire)]
enum Reply {
    #[wire(id = 0)]
    Count(Count),
    #[wire(id = 2)]
    Grouped(Grouped),
}
#[derive(Wire)]
struct Fact {
    #[wire(id = 1)]
    tag: u8,
    #[wire(id = 2)]
    value: u64,
}
#[derive(Wire)]
struct QueryField {
    #[wire(id = 1)]
    role: u8,
    #[wire(id = 2)]
    field: Text<64>,
}
#[derive(Wire)]
struct PublicError {
    #[wire(id = 1)]
    code: u16,
    #[wire(id = 2)]
    class: u8,
    #[wire(id = 3)]
    origin: u8,
    #[wire(id = 4)]
    facts: List<Fact, 4>,
    #[wire(id = 5)]
    query_field: Option<QueryField>,
}
// Retain native declaration order for the independent Postcard byte oracle.
#[derive(Serialize)]
struct NativeGrouped {
    entity: String,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    row_count: u32,
    next_cursor: Option<String>,
}
#[derive(Serialize)]
enum NativeReply {
    Count { entity: String, row_count: u32 },
    // The real enum's Projection occupies ordinal 1. This oracle never emits it.
    Projection(()),
    Grouped(NativeGrouped),
}
impl TryFrom<NativeGrouped> for Grouped {
    type Error = String;
    fn try_from(value: NativeGrouped) -> Result<Self, Self::Error> {
        // Check outer dimensions before creating replacement collections.
        if value.columns.len() > 4 {
            return Err("columns: exceeds 4".into());
        }
        if value.rows.len() > 4 {
            return Err("rows: exceeds 4; paginate before projection".into());
        }
        let entity = Text::new(value.entity).map_err(|_| "entity: exceeds 64 UTF-8 bytes")?;
        let columns = value
            .columns
            .into_iter()
            .enumerate()
            .map(|(i, s)| Text::new(s).map_err(|_| format!("columns[{i}]: exceeds 32 UTF-8 bytes")))
            .collect::<Result<Vec<_>, _>>()?;
        let rows = value
            .rows
            .into_iter()
            .enumerate()
            .map(|(r, row)| {
                if row.len() > 4 {
                    return Err(format!("rows[{r}]: exceeds 4 cells"));
                }
                let cells = row
                    .into_iter()
                    .enumerate()
                    .map(|(c, s)| {
                        Text::new(s).map_err(|_| format!("rows[{r}][{c}]: exceeds 64 UTF-8 bytes"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(List::new(cells).unwrap())
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            entity,
            columns: List::new(columns).unwrap(),
            rows: List::new(rows).unwrap(),
            row_count: value.row_count,
            next_cursor: value
                .next_cursor
                .map(Text::new)
                .transpose()
                .map_err(|_| "next_cursor: exceeds 128 UTF-8 bytes")?,
        })
    }
}
fn native_grouped() -> NativeGrouped {
    NativeGrouped {
        entity: "Entry".into(),
        columns: vec!["value".into(), "count".into()],
        rows: vec![
            vec!["NULL".into(), "7".into()],
            vec!["\"NULL\"".into(), "2".into()],
        ],
        row_count: 2,
        next_cursor: Some("opaque-cursor".into()),
    }
}
fn payload_matches(projected: &Result<Reply, PublicError>, native: &NativeReply) {
    let bytes =
        ic_wire::encode_response::<Contract<Result<Reply, PublicError>>>(projected).unwrap();
    assert_eq!(
        &bytes.as_bytes()[11..],
        postcard::to_allocvec(&Ok::<_, ()>(native)).unwrap()
    );
}
pub(super) fn check(f: &mut Fixture) {
    // Construct the reserved variant once to ensure the native oracle's ordinal is real.
    assert_eq!(
        postcard::to_allocvec(&NativeReply::Projection(())).unwrap(),
        [1]
    );
    let count = Ok(Reply::Count(Count {
        entity: Text::new("Entry".into()).unwrap(),
        row_count: 7,
    }));
    payload_matches(
        &count,
        &NativeReply::Count {
            entity: "Entry".into(),
            row_count: 7,
        },
    );
    f.add(
        count,
        "{ok:{tag:'Count',value:{entity:'Entry',row_count:7}}}",
    );
    let grouped = Ok(Reply::Grouped(Grouped::try_from(native_grouped()).unwrap()));
    payload_matches(&grouped, &NativeReply::Grouped(native_grouped()));
    f.add(grouped, "{ok:{tag:'Grouped',value:{entity:'Entry',columns:['value','count'],rows:[['NULL','7'],['\"NULL\"','2']],row_count:2,next_cursor:{some:'opaque-cursor'}}}}");
    // Raw numeric leaves exercise the error layout, not diagnostic-code semantics.
    let error: Result<Reply, PublicError> = Err(PublicError {
        code: u16::MAX,
        class: 1,
        origin: 7,
        facts: List::new(vec![Fact {
            tag: 255,
            value: u64::MAX,
        }])
        .unwrap(),
        query_field: Some(QueryField {
            role: 255,
            field: Text::new("missing.field".into()).unwrap(),
        }),
    });
    let native = Err::<(), _>((
        u16::MAX,
        1u8,
        7u8,
        vec![(255u8, u64::MAX)],
        Some((255u8, "missing.field")),
    ));
    let bytes = ic_wire::encode_response::<Contract<Result<Reply, PublicError>>>(&error).unwrap();
    assert_eq!(
        &bytes.as_bytes()[11..],
        postcard::to_allocvec(&native).unwrap()
    );
    f.add(error, "{err:{code:65535,class:1,origin:7,facts:[{tag:255,value:18446744073709551615n}],query_field:{some:{role:255,field:'missing.field'}}}}");
    let mut bad = native_grouped();
    bad.rows[0][1] = "é".repeat(33);
    assert_eq!(
        Grouped::try_from(bad).err().unwrap(),
        "rows[0][1]: exceeds 64 UTF-8 bytes"
    );
    let mut bad = native_grouped();
    bad.rows = vec![vec![]; 5];
    assert_eq!(
        Grouped::try_from(bad).err().unwrap(),
        "rows: exceeds 4; paginate before projection"
    );
    // Ordinary native SQL text has identical payload bytes to bounded Text.
    let sql = "SELECT value, COUNT(*) FROM Entry GROUP BY value";
    let text = Text::<128>::new(sql.into()).unwrap();
    let bytes = ic_wire::encode_request::<Contract<Text<128>>>(&text).unwrap();
    assert_eq!(&bytes.as_bytes()[11..], postcard::to_allocvec(sql).unwrap());
    f.add(text, &format!("{sql:?}"));
}
