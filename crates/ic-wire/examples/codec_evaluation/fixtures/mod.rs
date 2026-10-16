//! Synthetic, consumer-shaped fixtures. These are not upstream contracts.

use candid::CandidType;
use ic_principal::Principal;
use serde::{Deserialize, Serialize};

use crate::{
    Error, Row,
    bounded::{Bytes, List, Text},
    measure,
};

macro_rules! record {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CandidType)]
        #[serde(deny_unknown_fields)]
        pub(super) struct $name { $(pub(super) $field: $ty),* }
    };
}

record!(Request {
    account: u64,
    include_pending: bool
});
record!(Response {
    balance: u64,
    revision: u64
});
record!(Scope { target: Principal, methods: List<Text, 8> });
record!(Authority { subject: Principal, expires_at: u64, scopes: List<Scope, 4> });
record!(BlobMessage {
    id: u64,
    payload: Bytes
});
record!(Delivery {
    recipient: Principal,
    payload: Bytes
});
record!(Batch { sequence: u64, messages: List<Delivery, 8> });
record!(Ack { sequence: u64, accepted: List<u64, 8>, retry: bool });
record!(Field {
    id: u16,
    name: Text,
    kind: u8,
    nullable: bool
});
record!(Schema { revision: u32, entity: Text, fields: List<Field, 32> });

pub(super) fn corpus() -> Result<Vec<Row>, Error> {
    let mut rows = Vec::new();
    measure(
        "small-request",
        &Request {
            account: 7,
            include_pending: true,
        },
        &mut rows,
    )?;
    measure(
        "small-response",
        &Response {
            balance: 1_000_000,
            revision: 42,
        },
        &mut rows,
    )?;
    measure(
        "nested-authority",
        &Authority {
            subject: Principal::from_slice(&[1; 10]),
            expires_at: 1_800_000_000,
            scopes: List::new(vec![
                Scope {
                    target: Principal::from_slice(&[2; 10]),
                    methods: List::new(vec![Text::new("read")?, Text::new("write")?])?,
                },
                Scope {
                    target: Principal::from_slice(&[3; 29]),
                    methods: List::new(vec![Text::new("status")?])?,
                },
            ])?,
        },
        &mut rows,
    )?;
    let blob: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();
    measure(
        "blob-4096",
        &BlobMessage {
            id: 300,
            payload: Bytes::new(&blob)?,
        },
        &mut rows,
    )?;
    measure(
        "relay-batch",
        &Batch {
            sequence: 300,
            messages: List::new(vec![
                Delivery {
                    recipient: Principal::from_slice(&[2; 10]),
                    payload: Bytes::new(&[])?,
                },
                Delivery {
                    recipient: Principal::from_slice(&[3; 29]),
                    payload: Bytes::new(&blob[..128])?,
                },
                Delivery {
                    recipient: Principal::anonymous(),
                    payload: Bytes::new(&blob[..1024])?,
                },
            ])?,
        },
        &mut rows,
    )?;
    measure(
        "relay-ack",
        &Ack {
            sequence: 300,
            accepted: List::new(vec![0, 1, 2])?,
            retry: false,
        },
        &mut rows,
    )?;
    measure(
        "schema-artifact",
        &Schema {
            revision: 3,
            entity: Text::new("accounts")?,
            fields: List::new(vec![
                Field {
                    id: 10,
                    name: Text::new("owner")?,
                    kind: 1,
                    nullable: false,
                },
                Field {
                    id: 20,
                    name: Text::new("balance")?,
                    kind: 2,
                    nullable: false,
                },
                Field {
                    id: 30,
                    name: Text::new("memo")?,
                    kind: 3,
                    nullable: true,
                },
            ])?,
        },
        &mut rows,
    )?;
    Ok(rows)
}
