use chrono::{DateTime, Utc};
use firefly_client::models::casper::DeployDataProto;
use prost::Message;

use crate::common::models::{PositiveNonZero, PreparedContract};

pub mod api;
pub mod blockchain;
pub mod models;
pub mod tracing;

#[bon::builder]
pub fn prepare_for_signing(
    code: String,
    valid_after_block_number: u64,
    phlo_limit: Option<PositiveNonZero<i64>>,
    timestamp: Option<DateTime<Utc>>,
) -> PreparedContract {
    let timestamp = timestamp
        .unwrap_or_else(chrono::Utc::now)
        .timestamp_millis();
    let contract = DeployDataProto {
        term: code,
        timestamp,
        phlo_price: 1,
        phlo_limit: phlo_limit.map_or(5_000_000, |v| v.0),
        valid_after_block_number: valid_after_block_number as _,
        shard_id: "root".into(),
        ..Default::default()
    }
    .encode_to_vec();

    PreparedContract(contract)
}
