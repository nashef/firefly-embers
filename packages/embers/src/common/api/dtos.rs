use std::borrow::Cow;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use derive_more::From;
use firefly_client::helpers::ShortHex;
use firefly_client::models::{DeployId, Uri, WalletAddress};
use poem_openapi::payload::Json;
use poem_openapi::registry::{MetaSchema, MetaSchemaRef, Registry};
use poem_openapi::types::{
    Base64,
    ParseError,
    ParseFromJSON,
    ParseFromParameter,
    ParseResult,
    ToJSON,
    Type,
};
use poem_openapi::{ApiResponse, NewType, Object, Tags};
use secp256k1::PublicKey;

use crate::ai_agents_teams::models::Graph;
use crate::common::models;

impl<T> Type for models::PositiveNonZero<T>
where
    T: Format + Send + Sync,
    T::Alias: Type,
{
    const IS_REQUIRED: bool = <T as Format>::Alias::IS_REQUIRED;

    type RawValueType = T;
    type RawElementValueType = T;

    fn name() -> Cow<'static, str> {
        format!("PositiveNonZero_{}", <T::Alias as Type>::name()).into()
    }

    fn schema_ref() -> MetaSchemaRef {
        <T::Alias as Type>::schema_ref()
    }

    fn register(registry: &mut Registry) {
        <T::Alias as Type>::register(registry);
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.0)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        Box::new(self.as_raw_value().into_iter())
    }
}

/// Transforms T to [`String`] before serialization/deserialization
/// and keeps original format in `OpenApi` model.
#[derive(Debug, Clone, From)]
pub struct Stringified<T>(pub T);

impl<T> Type for Stringified<T>
where
    T: Format + Send + Sync,
    T::Alias: Type,
{
    const IS_REQUIRED: bool = <T as Format>::Alias::IS_REQUIRED;

    type RawValueType = T;
    type RawElementValueType = T;

    fn name() -> Cow<'static, str> {
        format!("Stringified_{}", <T::Alias as Type>::name()).into()
    }

    fn schema_ref() -> MetaSchemaRef {
        MetaSchemaRef::Inline(Box::new(MetaSchema::new_with_format("string", T::format())))
    }

    fn register(registry: &mut Registry) {
        <T::Alias as Type>::register(registry);
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(&self.0)
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        Box::new(self.as_raw_value().into_iter())
    }
}

trait Format {
    type Alias;
    fn format() -> &'static str;
}

impl Format for DateTime<Utc> {
    type Alias = Self;
    fn format() -> &'static str {
        "timestamp-millis"
    }
}

impl ParseFromJSON for Stringified<DateTime<Utc>> {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        let value = String::parse_from_json(value).map_err(ParseError::propagate)?;
        let millis = value.parse::<i64>().map_err(ParseError::custom)?;
        let datetime = DateTime::<Utc>::from_timestamp_millis(millis)
            .ok_or_else(|| ParseError::custom("invalid timestamp"))?;

        Ok(Self(datetime))
    }
}

impl ToJSON for Stringified<DateTime<Utc>> {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.0.timestamp_millis().to_string().to_json()
    }
}

impl From<Stringified<Self>> for DateTime<Utc> {
    fn from(value: Stringified<Self>) -> Self {
        value.0
    }
}

impl Format for u64 {
    type Alias = Self;
    fn format() -> &'static str {
        "uint64"
    }
}

impl ParseFromJSON for Stringified<u64> {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        let value = String::parse_from_json(value).map_err(ParseError::propagate)?;
        value.parse::<u64>().map(Self).map_err(ParseError::custom)
    }
}

impl ToJSON for Stringified<u64> {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.0.to_string().to_json()
    }
}

impl From<Stringified<Self>> for u64 {
    fn from(value: Stringified<Self>) -> Self {
        value.0
    }
}

impl Format for i64 {
    type Alias = Self;
    fn format() -> &'static str {
        "int64"
    }
}

impl ParseFromJSON for Stringified<i64> {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        let value = String::parse_from_json(value).map_err(ParseError::propagate)?;
        value.parse::<i64>().map(Self).map_err(ParseError::custom)
    }
}

impl ToJSON for Stringified<i64> {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.0.to_string().to_json()
    }
}

impl From<Stringified<Self>> for i64 {
    fn from(value: Stringified<Self>) -> Self {
        value.0
    }
}

impl<T> Format for models::PositiveNonZero<T>
where
    T: Format,
{
    type Alias = T::Alias;
    fn format() -> &'static str {
        T::format()
    }
}

impl ParseFromJSON for Stringified<models::PositiveNonZero<i64>> {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        let value = String::parse_from_json(value).map_err(ParseError::propagate)?;
        let number = value.parse::<i64>().map_err(ParseError::custom)?;
        number.try_into().map(Self).map_err(ParseError::custom)
    }
}

impl ToJSON for Stringified<models::PositiveNonZero<i64>> {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.0.0.to_string().to_json()
    }
}

impl From<Stringified<Self>> for models::PositiveNonZero<i64> {
    fn from(value: Stringified<Self>) -> Self {
        value.0
    }
}

impl Format for WalletAddress {
    type Alias = String;
    fn format() -> &'static str {
        "blockchain-address"
    }
}

impl ParseFromParameter for Stringified<WalletAddress> {
    fn parse_from_parameter(value: &str) -> ParseResult<Self> {
        value.to_owned().try_into().map(Self).map_err(Into::into)
    }
}

impl ParseFromJSON for Stringified<WalletAddress> {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        let value = String::parse_from_json(value).map_err(ParseError::propagate)?;
        value.try_into().map(Self).map_err(Into::into)
    }
}

impl ToJSON for Stringified<WalletAddress> {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.0.as_ref().to_json()
    }
}

impl From<Stringified<Self>> for WalletAddress {
    fn from(value: Stringified<Self>) -> Self {
        value.0
    }
}

impl Format for Uri {
    type Alias = String;
    fn format() -> &'static str {
        "blockchain-uri"
    }
}

impl ParseFromParameter for Stringified<Uri> {
    fn parse_from_parameter(value: &str) -> ParseResult<Self> {
        value.to_owned().try_into().map(Self).map_err(Into::into)
    }
}

impl ParseFromJSON for Stringified<Uri> {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        let value = String::parse_from_json(value).map_err(ParseError::propagate)?;
        value.try_into().map(Self).map_err(Into::into)
    }
}

impl ToJSON for Stringified<Uri> {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.0.as_ref().to_json()
    }
}

impl From<Stringified<Self>> for Uri {
    fn from(value: Stringified<Self>) -> Self {
        value.0
    }
}

impl Format for Graph {
    type Alias = String;
    fn format() -> &'static str {
        "graphl"
    }
}

impl ParseFromParameter for Stringified<Graph> {
    fn parse_from_parameter(value: &str) -> ParseResult<Self> {
        Graph::new(value.to_owned()).map(Self).map_err(Into::into)
    }
}

impl ParseFromJSON for Stringified<Graph> {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        let value = String::parse_from_json(value).map_err(ParseError::propagate)?;
        Graph::new(value).map(Self).map_err(Into::into)
    }
}

impl ToJSON for Stringified<Graph> {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.0.clone().graphl().to_json()
    }
}

impl From<Stringified<Self>> for Graph {
    fn from(value: Stringified<Self>) -> Self {
        value.0
    }
}

impl Format for PublicKey {
    type Alias = String;
    fn format() -> &'static str {
        "public-key"
    }
}

impl ParseFromJSON for Stringified<PublicKey> {
    fn parse_from_json(value: Option<serde_json::Value>) -> ParseResult<Self> {
        let value = String::parse_from_json(value).map_err(ParseError::propagate)?;
        PublicKey::from_str(&value).map(Self).map_err(Into::into)
    }
}

impl ToJSON for Stringified<PublicKey> {
    fn to_json(&self) -> Option<serde_json::Value> {
        self.0.to_string().to_json()
    }
}

impl From<Stringified<Self>> for PublicKey {
    fn from(value: Stringified<Self>) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Tags)]
pub enum ApiTags {
    Testnet,
    Wallets,
    AIAgents,
    AIAgentsTeams,
    Service,
}

#[derive(Debug, Clone, Object)]
pub struct InternalError {
    description: String,
}

impl From<anyhow::Error> for InternalError {
    fn from(err: anyhow::Error) -> Self {
        Self {
            description: format!("{err:?}"),
        }
    }
}

#[derive(Debug, Clone, ApiResponse)]
pub enum MaybeNotFound<T>
where
    T: Type + ToJSON + Send + Sync,
{
    #[oai(status = 200)]
    Ok(Json<T>),
    #[oai(status = 404)]
    NotFound,
    #[oai(status = 500)]
    InternalError(Json<InternalError>),
}

impl<T, K> From<Option<T>> for MaybeNotFound<K>
where
    K: Type + ToJSON + Send + Sync + From<T>,
{
    fn from(value: Option<T>) -> Self {
        value.map_or_else(|| Self::NotFound, |value| Self::Ok(Json(value.into())))
    }
}

impl<T, K> From<anyhow::Result<Option<T>>> for MaybeNotFound<K>
where
    K: Type + ToJSON + Send + Sync + From<T>,
{
    fn from(value: anyhow::Result<Option<T>>) -> Self {
        match value {
            Ok(opt) => opt.into(),
            Err(err) => Self::InternalError(Json(err.into())),
        }
    }
}

#[derive(derive_more::Debug, Clone, NewType)]
#[oai(to_header = false, from_multipart = false)]
#[debug("{:?}", _0.0.short_hex(32))]
pub struct PreparedContract(pub Base64<Vec<u8>>);

impl From<models::PreparedContract> for PreparedContract {
    fn from(value: models::PreparedContract) -> Self {
        Self(Base64(value.0))
    }
}

#[derive(derive_more::Debug, Clone, Object)]
pub struct SignedContract {
    #[debug("{:?}", contract.0.short_hex(32))]
    pub contract: Base64<Vec<u8>>,
    #[debug("{:?}", hex::encode(&sig.0))]
    pub sig: Base64<Vec<u8>>,
    pub sig_algorithm: String,
    #[debug("{:?}", hex::encode(&deployer.0))]
    pub deployer: Base64<Vec<u8>>,
}

impl From<SignedContract> for firefly_client::models::SignedCode {
    fn from(value: SignedContract) -> Self {
        Self {
            contract: value.contract.0,
            sig: value.sig.0,
            sig_algorithm: value.sig_algorithm,
            deployer: value.deployer.0,
        }
    }
}

#[derive(Debug, Clone, Object)]
pub struct RegistryDeploy {
    pub timestamp: Stringified<DateTime<Utc>>,
    pub version: Stringified<i64>,
    pub uri_pub_key: Stringified<PublicKey>,
    pub signature: Base64<Vec<u8>>,
}

impl From<RegistryDeploy> for models::RegistryDeploy {
    fn from(value: RegistryDeploy) -> Self {
        Self {
            timestamp: value.timestamp.into(),
            version: value.version.0,
            uri_pub_key: value.uri_pub_key.into(),
            signature: value.signature.0,
        }
    }
}

#[derive(Debug, Clone, Object)]
pub struct SendResp {
    pub deploy_id: String,
}

impl From<DeployId> for SendResp {
    fn from(value: DeployId) -> Self {
        Self {
            deploy_id: value.into(),
        }
    }
}
