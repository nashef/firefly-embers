use std::collections::HashMap;
use std::marker::PhantomData;

use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use chrono::{DateTime, Utc};
use crc::Crc;
use derive_more::{AsRef, Display, From, Into};
use digest::OutputSizeUser;
use digest::typenum::Unsigned;
use secp256k1::PublicKey;
use serde::{Deserialize, Deserializer, Serialize, de};
use thiserror::Error;

use crate::helpers::ShortHex;
use crate::rendering::{IntoValue, Value};

pub mod servicemodelapi {
    #![allow(warnings)]
    #![allow(clippy::all)]
    #![allow(clippy::pedantic)]
    #![allow(clippy::nursery)]
    tonic::include_proto!("servicemodelapi");
}

pub mod rhoapi {
    #![allow(warnings)]
    #![allow(clippy::all)]
    #![allow(clippy::pedantic)]
    #![allow(clippy::nursery)]
    tonic::include_proto!("rhoapi");
}

pub mod casper {
    #![allow(warnings)]
    #![allow(clippy::all)]
    #![allow(clippy::pedantic)]
    #![allow(clippy::nursery)]
    tonic::include_proto!("casper");

    pub mod v1 {
        tonic::include_proto!("casper.v1");
    }
}

#[derive(
    Debug,
    Clone,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsRef,
    Into,
    From,
    Serialize,
    Deserialize,
)]
pub struct BlockId(String);

impl IntoValue for BlockId {
    fn into_value(self) -> Value {
        self.0.into_value()
    }
}

#[derive(
    Debug,
    Clone,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsRef,
    Into,
    From,
    Serialize,
    Deserialize,
)]
pub struct DeployId(String);

impl IntoValue for DeployId {
    fn into_value(self) -> Value {
        self.0.into_value()
    }
}

#[derive(derive_more::Debug, Clone)]
pub struct SignedCode {
    #[debug("{:?}", contract.short_hex(32))]
    pub contract: Vec<u8>,
    #[debug("{:?}", hex::encode(sig))]
    pub sig: Vec<u8>,
    pub sig_algorithm: String,
    #[debug("{:?}", hex::encode(deployer))]
    pub deployer: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ReadNodeExprUnforg {
    UnforgPrivate { data: String },
    UnforgDeploy { data: String },
    UnforgDeployer { data: String },
}

impl From<ReadNodeExprUnforg> for serde_json::Value {
    fn from(value: ReadNodeExprUnforg) -> Self {
        match value {
            ReadNodeExprUnforg::UnforgPrivate { data } => Self::String(data),
            ReadNodeExprUnforg::UnforgDeploy { data } => Self::String(data),
            ReadNodeExprUnforg::UnforgDeployer { data } => Self::String(data),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum ReadNodeExpr {
    ExprTuple { data: Vec<Self> },
    ExprList { data: Vec<Self> },
    ExprSet { data: Vec<Self> },
    ExprMap { data: HashMap<String, Self> },

    ExprNil {},
    ExprBool { data: bool },
    ExprInt { data: serde_json::Number },
    ExprString { data: String },
    ExprBytes { data: String },
    ExprUri { data: String },
    ExprUnforg { data: ReadNodeExprUnforg },
}

impl From<ReadNodeExpr> for serde_json::Value {
    fn from(value: ReadNodeExpr) -> Self {
        match value {
            ReadNodeExpr::ExprTuple { data } => {
                Self::Array(data.into_iter().map(Into::into).collect())
            }
            ReadNodeExpr::ExprList { data } => {
                Self::Array(data.into_iter().map(Into::into).collect())
            }
            ReadNodeExpr::ExprSet { data } => {
                Self::Array(data.into_iter().map(Into::into).collect())
            }
            ReadNodeExpr::ExprMap { data } => {
                Self::Object(data.into_iter().map(|(k, v)| (k, v.into())).collect())
            }
            ReadNodeExpr::ExprNil {} => Self::Null,
            ReadNodeExpr::ExprBool { data } => Self::Bool(data),
            ReadNodeExpr::ExprInt { data } => Self::Number(data),
            ReadNodeExpr::ExprString { data } => Self::String(data),
            ReadNodeExpr::ExprBytes { data } => Self::String(data),
            ReadNodeExpr::ExprUri { data } => Self::String(data),
            ReadNodeExpr::ExprUnforg { data } => data.into(),
        }
    }
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn to_result(self) -> Result<R, L> {
        self.into()
    }
}

impl<L, R> From<Either<L, R>> for Result<R, L> {
    fn from(value: Either<L, R>) -> Self {
        match value {
            Either::Left(err) => Err(err),
            Either::Right(v) => Ok(v),
        }
    }
}

impl<'de, L, R> Deserialize<'de> for Either<L, R>
where
    L: Deserialize<'de>,
    R: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(
            2,
            EitherVisitor {
                phantom: PhantomData,
            },
        )
    }
}

struct EitherVisitor<L, R> {
    phantom: PhantomData<(L, R)>,
}

impl<'de, L, R> de::Visitor<'de> for EitherVisitor<L, R>
where
    L: Deserialize<'de>,
    R: Deserialize<'de>,
{
    type Value = Either<L, R>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence of (bool, value)")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let valid: bool = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;

        if valid {
            seq.next_element()?.map(Either::Right)
        } else {
            seq.next_element()?.map(Either::Left)
        }
        .ok_or_else(|| de::Error::invalid_length(1, &self))
    }
}

#[derive(Debug, Clone)]
pub enum ValidAfter {
    Head,
    Index(u64),
}

#[derive(Debug, Clone, bon::Builder)]
pub struct DeployData {
    #[builder(start_fn)]
    pub term: String,

    #[builder(default = 5_000_000)]
    pub phlo_limit: u64,

    #[builder(default = chrono::Utc::now())]
    pub timestamp: DateTime<Utc>,

    #[builder(default = ValidAfter::Head)]
    pub valid_after_block_number: ValidAfter,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
pub enum NodeEvent {
    Started,
    BlockAdded { payload: BlockEventPayload },
    BlockCreated { payload: BlockEventPayload },
    BlockFinalised { payload: BlockEventPayload },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BlockEventPayload {
    pub block_hash: BlockId,
    pub deploys: Vec<BlockEventDeploy>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlockEventDeploy {
    pub id: DeployId,
    pub cost: u64,
    pub deployer: PublicKey,
    pub errored: bool,
}

pub const FIRECAP_ID: [u8; 3] = [0, 0, 0];
pub const FIRECAP_VERSION: u8 = 0;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Into, AsRef)]
pub struct WalletAddress(String);

impl IntoValue for WalletAddress {
    fn into_value(self) -> Value {
        self.0.into_value()
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseWalletAddressError {
    #[error("internal encoder error: {0}")]
    EncoderError(bs58::decode::Error),

    #[error("invalid address size: {0}")]
    InvalidRevAddressSize(usize),

    #[error("invalid address format: {0}")]
    InvalidAddress(String),
}

impl TryFrom<String> for WalletAddress {
    type Error = ParseWalletAddressError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let decoded = bs58::decode(&value)
            .into_vec()
            .map_err(Self::Error::EncoderError)?;

        let (payload, checksum) = decoded
            .split_at_checked(decoded.len().wrapping_sub(4))
            .ok_or(ParseWalletAddressError::InvalidRevAddressSize(
                decoded.len(),
            ))?;

        let hash = Blake2b::<U32>::new().chain_update(payload).finalize();

        if checksum != &hash[..4] {
            return Err(ParseWalletAddressError::InvalidAddress(value));
        }

        Ok(Self(value))
    }
}

impl From<PublicKey> for WalletAddress {
    fn from(key: PublicKey) -> Self {
        let key_hash: [u8; 32] = sha3::Keccak256::new()
            .chain_update(&key.serialize_uncompressed()[1..])
            .finalize()
            .into();

        let eth_hash = sha3::Keccak256::new()
            .chain_update(&key_hash[key_hash.len() - 20..])
            .finalize();

        let checksum_hash: [u8; 32] = Blake2b::<U32>::new()
            .chain_update(FIRECAP_ID)
            .chain_update([FIRECAP_VERSION])
            .chain_update(eth_hash)
            .finalize()
            .into();

        let checksum = &checksum_hash[0..4];

        let address_bytes = [
            FIRECAP_ID.as_ref(),
            [FIRECAP_VERSION].as_ref(),
            eth_hash.as_ref(),
            checksum,
        ]
        .concat();

        Self(bs58::encode(address_bytes).into_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Into, AsRef)]
pub struct Uri(String);

const CRC14: crc::Algorithm<u16> = crc::Algorithm {
    width: 14,
    poly: 0x4805,
    init: 0x0000,
    refin: false,
    refout: false,
    xorout: 0x0000,
    check: 0,
    residue: 0x0000,
};

impl From<PublicKey> for Uri {
    fn from(value: PublicKey) -> Self {
        let hash = Blake2b::<U32>::new()
            .chain_update(value.serialize_uncompressed())
            .finalize();

        let crc = Crc::<u16>::new(&CRC14).checksum(&hash).to_ne_bytes();
        let full_key = [hash.as_ref(), [crc[0], crc[1] << 2].as_ref()].concat();
        let encoded = zbase32::encode(&full_key, 270);
        Self(format!("rho:id:{encoded}"))
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParseUriError {
    #[error("invalid uri prefix")]
    IvalidPrefix,

    #[error("invalid zbase32: {0}")]
    InvalidZBase32(&'static str),

    #[error("invalid decoded bytes length")]
    InvalidDecodedLength,

    #[error("checksum mistmatch")]
    ChecksumMistmatch,
}

impl TryFrom<String> for Uri {
    type Error = ParseUriError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        const HASH_SIZE: usize = <Blake2b<U32> as OutputSizeUser>::OutputSize::USIZE;

        let encoded = value
            .strip_prefix("rho:id:")
            .ok_or(Self::Error::IvalidPrefix)?;
        let decoded = zbase32::decode_str(encoded, 270).map_err(Self::Error::InvalidZBase32)?;
        let bytes: [u8; HASH_SIZE + 2] = decoded
            .try_into()
            .map_err(|_| Self::Error::InvalidDecodedLength)?;

        let (hash, crc_bytes) = bytes.split_at(HASH_SIZE);
        let crc = u16::from_ne_bytes([crc_bytes[0], crc_bytes[1] >> 2]);
        let expected = Crc::<u16>::new(&CRC14).checksum(hash);

        if expected != crc {
            return Err(Self::Error::ChecksumMistmatch);
        }

        Ok(Self(value))
    }
}

impl IntoValue for Uri {
    fn into_value(self) -> Value {
        Value::Uri(self.0)
    }
}
