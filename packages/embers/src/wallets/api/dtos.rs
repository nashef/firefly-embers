use chrono::{DateTime, Utc};
use firefly_client::models::WalletAddress;
use poem_openapi::{Enum, Object, Union};
use structural_convert::StructuralConvert;

use crate::common::api::dtos::{PreparedContract, Stringified};
use crate::common::models::PositiveNonZero;
use crate::wallets::models;

#[derive(Debug, Clone, Object, StructuralConvert)]
#[convert(from(models::Transfer))]
pub struct Transfer {
    pub id: String,
    pub timestamp: Stringified<DateTime<Utc>>,
    pub from: Stringified<WalletAddress>,
    pub to: Stringified<WalletAddress>,
    pub amount: Stringified<PositiveNonZero<i64>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Object, StructuralConvert)]
#[convert(from(models::Boost))]
pub struct Boost {
    pub id: String,
    pub timestamp: Stringified<DateTime<Utc>>,
    pub from: Stringified<WalletAddress>,
    pub to: Stringified<WalletAddress>,
    pub amount: Stringified<PositiveNonZero<i64>>,
    pub description: Option<String>,
    pub post_author_did: String,
    pub post_id: Option<String>,
}

#[derive(Debug, Clone, Object, StructuralConvert)]
#[convert(from(models::Exchange))]
pub struct Exchange {}

#[derive(Debug, Clone, Eq, PartialEq, StructuralConvert, Enum)]
#[convert(from(models::RequestStatus))]
#[oai(rename_all = "lowercase")]
pub enum RequestStatus {
    Done,
    Ongoing,
    Cancelled,
}

#[derive(Debug, Clone, Object, StructuralConvert)]
#[convert(from(models::Request))]
pub struct Request {
    pub id: String,
    pub timestamp: Stringified<DateTime<Utc>>,
    pub amount: Stringified<PositiveNonZero<i64>>,
    pub status: RequestStatus,
}

#[derive(Debug, Clone, Object, StructuralConvert)]
#[convert(from(models::WalletStateAndHistory))]
pub struct WalletStateAndHistory {
    pub balance: Stringified<u64>,
    pub requests: Vec<Request>,
    pub exchanges: Vec<Exchange>,
    pub boosts: Vec<Boost>,
    pub transfers: Vec<Transfer>,
}

#[derive(Debug, Clone, Object, StructuralConvert)]
#[convert(into(models::TransferReq))]
pub struct TransferReq {
    pub from: Stringified<WalletAddress>,
    pub to: Stringified<WalletAddress>,
    pub amount: Stringified<PositiveNonZero<i64>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Object)]
pub struct TransferResp {
    pub contract: PreparedContract,
}

#[derive(Debug, Clone, Object, StructuralConvert)]
#[convert(into(models::BoostReq))]
pub struct BoostReq {
    pub from: Stringified<WalletAddress>,
    pub to: Stringified<WalletAddress>,
    pub amount: Stringified<PositiveNonZero<i64>>,
    pub description: Option<String>,
    pub post_author_did: String,
    pub post_id: Option<String>,
}

#[derive(Debug, Clone, Object)]
pub struct BoostResp {
    pub contract: PreparedContract,
}

#[derive(Debug, Clone, Enum, StructuralConvert)]
#[convert(from(models::NodeType))]
pub enum NodeType {
    Validator,
    Observer,
}

#[derive(Debug, Clone, Object, StructuralConvert)]
#[convert(from(models::DeployDescription))]
pub struct DeployDescription {
    pub deploy_id: String,
    pub cost: Stringified<u64>,
    pub errored: bool,
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Union, StructuralConvert)]
#[oai(discriminator_name = "type")]
#[convert(from(models::DeployEvent))]
pub enum DeployEvent {
    Finalized(DeployDescription),
}
