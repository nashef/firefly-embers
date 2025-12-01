use firefly_client::models::ParseWalletAddressError;
use serde::Deserialize;
use thiserror::Error;

use crate::common::blockchain;
use crate::common::models::PositiveNonZeroParsingError;
use crate::wallets::models::{Boost, Transfer};

#[derive(Debug, Clone, Deserialize)]
pub struct TransferRecord {
    pub id: String,
    pub timestamp: blockchain::dtos::DateTime,
    pub from: String,
    pub to: String,
    pub amount: i64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoostRecord {
    pub id: String,
    pub timestamp: blockchain::dtos::DateTime,
    pub from: String,
    pub to: String,
    pub amount: i64,
    pub description: Option<String>,
    pub post_author_did: String,
    pub post_id: Option<String>,
}

#[derive(Debug, Clone, Error)]
pub enum HistoryValidationError {
    #[error("description format error: {0}")]
    AmountError(#[from] PositiveNonZeroParsingError),
    #[error("receiver wallet adress has wrong format: {0}")]
    WrongReceiverAddressFormat(ParseWalletAddressError),
    #[error("sender wallet adress has wrong format: {0}")]
    WrongSenderAddressFormat(ParseWalletAddressError),
}

impl TryFrom<TransferRecord> for Transfer {
    type Error = HistoryValidationError;

    fn try_from(record: TransferRecord) -> Result<Self, Self::Error> {
        let from = record
            .from
            .try_into()
            .map_err(Self::Error::WrongSenderAddressFormat)?;
        let to = record
            .to
            .try_into()
            .map_err(Self::Error::WrongReceiverAddressFormat)?;

        let amount = record.amount.try_into()?;

        Ok(Self {
            id: record.id,
            timestamp: record.timestamp.into(),
            from,
            to,
            amount,
            description: record.description,
        })
    }
}

impl TryFrom<BoostRecord> for Boost {
    type Error = HistoryValidationError;

    fn try_from(record: BoostRecord) -> Result<Self, Self::Error> {
        let from = record
            .from
            .try_into()
            .map_err(Self::Error::WrongSenderAddressFormat)?;
        let to = record
            .to
            .try_into()
            .map_err(Self::Error::WrongReceiverAddressFormat)?;

        let amount = record.amount.try_into()?;

        Ok(Self {
            id: record.id,
            timestamp: record.timestamp.into(),
            from,
            to,
            amount,
            description: record.description,
            post_author_did: record.post_author_did,
            post_id: record.post_id,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BalanceAndHistory {
    pub balance: u64,
    pub transfers: Vec<TransferRecord>,
    pub boosts: Vec<BoostRecord>,
}
