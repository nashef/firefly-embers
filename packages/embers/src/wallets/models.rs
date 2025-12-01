use chrono::{DateTime, Utc};
use firefly_client::models::{DeployId, WalletAddress};

use crate::common::models::PositiveNonZero;

pub type Amount = PositiveNonZero<i64>;

#[derive(Debug, Clone)]
pub struct Transfer {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub from: WalletAddress,
    pub to: WalletAddress,
    pub amount: Amount,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WalletStateAndHistory {
    pub balance: u64,
    pub requests: Vec<Request>,
    pub exchanges: Vec<Exchange>,
    pub boosts: Vec<Boost>,
    pub transfers: Vec<Transfer>,
}

#[derive(Debug, Clone)]
pub struct Boost {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub from: WalletAddress,
    pub to: WalletAddress,
    pub amount: Amount,
    pub description: Option<String>,
    pub post_author_did: String,
    pub post_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub amount: Amount,
    pub status: RequestStatus,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RequestStatus {
    Done,
    Ongoing,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Exchange {}

#[derive(Debug, Clone)]
pub struct TransferReq {
    pub from: WalletAddress,
    pub to: WalletAddress,
    pub amount: Amount,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BoostReq {
    pub from: WalletAddress,
    pub to: WalletAddress,
    pub amount: Amount,
    pub description: Option<String>,
    pub post_author_did: String,
    pub post_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Validator,
    Observer,
}

#[derive(Debug, Clone)]
pub struct DeployDescription {
    pub deploy_id: DeployId,
    pub cost: u64,
    pub errored: bool,
    pub node_type: NodeType,
}

#[derive(Debug, Clone)]
pub enum DeployEvent {
    Finalized(DeployDescription),
}
