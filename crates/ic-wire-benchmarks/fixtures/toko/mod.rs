//! Toko-shaped inputs, copied from Toko 44d4e2c6d; no application dependency.
//! TokenCard is a synthetic projection, not Toko's complete TokenSummary.
use super::*;
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct Funding {
    estimated_monthly_ic_burn: u64,
    estimated_monthly_total_burn: u64,
    instance_cycle_balance: u64,
    ledger_cycle_balance: Option<u64>,
    latest_cycle_tracker_timestamp_secs: Option<u64>,
}
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct Billing {
    days_requested: u32,
    days_with_data: u32,
    avg_daily_total_burn: u64,
    avg_daily_ic_burn: u64,
    avg_daily_blob_burn: u64,
    estimated_monthly_total_burn: u64,
    estimated_monthly_ic_burn: u64,
    estimated_monthly_blob_burn: u64,
    latest_instance_balance: u64,
    latest_ledger_balance: Option<u64>,
    latest_blob_cashier_balance: Option<u64>,
    latest_remote_media_bytes_total: u64,
}
// Match IcyDB v0.258.0's non-human-readable ULID blob encoding and
// fixed-width sequence admission. JSON identity uses byte arrays, not ULID text.
#[derive(Debug, PartialEq)]
struct Id([u8; 16]);
impl Serialize for Id {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(&self.0)
    }
}
impl<'de> serde::Deserialize<'de> for Id {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Id;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("16 ULID bytes")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut a: A) -> Result<Id, A::Error> {
                let mut bytes = [0; 16];
                for b in &mut bytes {
                    *b = a
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::custom("short ULID"))?;
                }
                if a.next_element::<u8>()?.is_some() {
                    return Err(serde::de::Error::custom("long ULID"));
                }
                Ok(Id(bytes))
            }
        }
        d.deserialize_seq(Visitor)
    }
}
#[cfg(feature = "candid")]
impl CandidType for Id {
    fn _ty() -> ic_platform::candid::contract::types::Type {
        <Vec<u8> as CandidType>::ty()
    }
    fn idl_serialize<S: ic_platform::candid::contract::types::Serializer>(
        &self,
        s: S,
    ) -> Result<(), S::Error> {
        s.serialize_blob(&self.0)
    }
}
use serde::Serializer;
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct Filters {
    token_ids: Vec<Id>,
    tags: Vec<String>,
    tokens_missing_required_attributes: Vec<Id>,
    draft_missing_required_attributes: Vec<Id>,
    draft_missing_tier: Vec<Id>,
    draft_missing_token_type: Vec<Id>,
    draft_missing_contract: Vec<Id>,
}
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct VariantInput {
    key: String,
    label: String,
    description: Option<String>,
    asset_id: Option<Id>,
    display_order: u16,
}
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct TokenCard {
    id: Id,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    asset_ids: Vec<Id>,
    minted_quantity: u32,
    final_score: Option<String>,
    score_stale: bool,
}
#[derive(Debug, PartialEq, serde::Deserialize, Serialize)]
#[cfg_attr(feature = "candid", derive(CandidType))]
#[cfg_attr(feature = "candid", candid_path("ic_platform::candid::contract"))]
struct Page {
    items: Vec<TokenCard>,
    next_offset: Option<u32>,
    has_more: bool,
    total: u32,
}
fn id(i: usize) -> Id {
    Id((0x0190_0000_0000_7000_8000_0000_0000_0000_u128 + i as u128).to_be_bytes())
}
pub(super) fn cases_selected(selected: Option<usize>) -> Vec<Case> {
    let mut out = Vec::new();
    let mut index = 0;
    macro_rules! push {
        ($name:expr, $value:expr) => {{
            if selected.is_none_or(|s| s == index) {
                out.push(case($name, $value));
            }
            index += 1;
        }};
    }
    push!(
        "toko-funding",
        Funding {
            estimated_monthly_ic_burn: 3_000_000_000_000,
            estimated_monthly_total_burn: 5_000_000_000_000,
            instance_cycle_balance: 20_000_000_000_000,
            ledger_cycle_balance: Some(10_000_000_000_000),
            latest_cycle_tracker_timestamp_secs: Some(1_792_627_200)
        }
    );
    push!(
        "toko-billing",
        Billing {
            days_requested: 30,
            days_with_data: 28,
            avg_daily_total_burn: 170_000_000_000,
            avg_daily_ic_burn: 100_000_000_000,
            avg_daily_blob_burn: 70_000_000_000,
            estimated_monthly_total_burn: 5_100_000_000_000,
            estimated_monthly_ic_burn: 3_000_000_000_000,
            estimated_monthly_blob_burn: 2_100_000_000_000,
            latest_instance_balance: 20_000_000_000_000,
            latest_ledger_balance: Some(10_000_000_000_000),
            latest_blob_cashier_balance: Some(4_000_000_000_000),
            latest_remote_media_bytes_total: 1_073_741_824
        }
    );
    push!(
        "toko-create-variant",
        VariantInput {
            key: "gold".into(),
            label: "Gold edition".into(),
            description: Some("A limited gold edition with alternate artwork.".into()),
            asset_id: Some(id(1)),
            display_order: 2
        }
    );
    for n in [0, 50, 500, 1000] {
        push!(
            &format!("toko-filter-ids-{n}"),
            Filters {
                token_ids: (0..n).map(id).collect(),
                tags: (0..8).map(|i| format!("tag-{i}")).collect(),
                tokens_missing_required_attributes: (0..n / 10).map(id).collect(),
                draft_missing_required_attributes: (0..n / 20).map(id).collect(),
                draft_missing_tier: vec![],
                draft_missing_token_type: vec![],
                draft_missing_contract: vec![]
            }
        );
    }
    for n in [0, 1, 50] {
        push!(
            &format!("toko-token-card-projection-{n}"),
            Page {
                items: (0..n)
                    .map(|i| TokenCard {
                        id: id(i),
                        name: format!("Token {i:04}"),
                        description: Some(
                            "An example collectible with a deterministic description.".repeat(2)
                        ),
                        tags: vec!["collectible".into(), "edition-one".into()],
                        asset_ids: vec![id(i * 2 + 10_000), id(i * 2 + 10_001)],
                        minted_quantity: i as u32 * 7,
                        final_score: Some("1234.5678".into()),
                        score_stale: false
                    })
                    .collect(),
                next_offset: Some(n as u32),
                has_more: true,
                total: 1000
            }
        );
    }
    assert!(selected.is_none_or(|s| s < index), "unknown Toko case");
    out
}
