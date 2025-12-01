use chrono::{DateTime, Utc};
use firefly_client::models::{Uri, WalletAddress};
use poem_openapi::{Object, Union};
use structural_convert::StructuralConvert;

use crate::ai_agents_teams::models;
use crate::common::api::dtos::{PreparedContract, RegistryDeploy, SignedContract, Stringified};
use crate::common::models::PositiveNonZero;

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(from(models::AgentsTeams))]
pub struct AgentsTeams {
    pub agents_teams: Vec<AgentsTeamHeader>,
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(from(models::AgentsTeamHeader))]
pub struct AgentsTeamHeader {
    pub id: String,
    pub version: String,
    pub created_at: Stringified<DateTime<Utc>>,
    pub last_deploy: Option<Stringified<DateTime<Utc>>>,
    pub name: String,
    pub description: Option<String>,
    pub shard: Option<String>,
    pub logo: Option<String>,
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(into(models::CreateAgentsTeamReq))]
pub struct CreateAgentsTeamReq {
    pub name: String,
    pub description: Option<String>,
    pub shard: Option<String>,
    pub logo: Option<String>,
    pub graph: Option<Stringified<models::Graph>>,
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(from(models::AgentsTeam))]
pub struct AgentsTeam {
    pub id: String,
    pub version: String,
    pub created_at: Stringified<DateTime<Utc>>,
    pub last_deploy: Option<Stringified<DateTime<Utc>>>,
    pub uri: Option<Stringified<Uri>>,
    pub name: String,
    pub description: Option<String>,
    pub shard: Option<String>,
    pub logo: Option<String>,
    pub graph: Option<Stringified<models::Graph>>,
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(from(models::CreateAgentsTeamResp))]
pub struct CreateAgentsTeamResp {
    pub id: String,
    pub version: String,
    pub contract: PreparedContract,
}

pub type SaveAgentsTeamReq = CreateAgentsTeamReq;

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(from(models::SaveAgentsTeamResp))]
pub struct SaveAgentsTeamResp {
    pub version: String,
    pub contract: PreparedContract,
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(from(models::DeleteAgentsTeamResp))]
pub struct DeleteAgentsTeamResp {
    pub contract: PreparedContract,
}

#[derive(Debug, Clone, Object)]
pub struct DeployAgentsTeam {
    pub id: String,
    pub version: String,
    pub address: Stringified<WalletAddress>,
    pub phlo_limit: Stringified<PositiveNonZero<i64>>,
    pub deploy: RegistryDeploy,
}

#[derive(Debug, Clone, Object)]
pub struct DeployGraph {
    pub graph: Stringified<models::Graph>,
    pub phlo_limit: Stringified<PositiveNonZero<i64>>,
    pub deploy: RegistryDeploy,
}

#[derive(Debug, Clone, Union)]
#[oai(one_of = true, discriminator_name = "type")]
pub enum DeployAgentsTeamReq {
    AgentsTeam(DeployAgentsTeam),
    Graph(DeployGraph),
}

impl From<DeployAgentsTeamReq> for models::DeployAgentsTeamReq {
    fn from(value: DeployAgentsTeamReq) -> Self {
        match value {
            DeployAgentsTeamReq::AgentsTeam(deploy) => Self::AgentsTeam {
                id: deploy.id,
                version: deploy.version,
                address: deploy.address.0,
                phlo_limit: deploy.phlo_limit.0,
                deploy: deploy.deploy.into(),
            },
            DeployAgentsTeamReq::Graph(deploy) => Self::Graph {
                graph: deploy.graph.0,
                phlo_limit: deploy.phlo_limit.0,
                deploy: deploy.deploy.into(),
            },
        }
    }
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(from(models::DeployAgentsTeamResp))]
pub struct DeployAgentsTeamResp {
    pub contract: PreparedContract,
    pub system: Option<PreparedContract>,
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(into(models::DeploySignedAgentsTeamtReq))]
pub struct DeploySignedAgentsTeamtReq {
    pub contract: SignedContract,
    pub system: Option<SignedContract>,
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(into(models::RunAgentsTeamReq))]
pub struct RunAgentsTeamReq {
    pub prompt: String,
    pub phlo_limit: Stringified<PositiveNonZero<i64>>,
    pub agents_team: Stringified<Uri>,
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(from(models::RunAgentsTeamResp))]
pub struct RunAgentsTeamResp {
    pub contract: PreparedContract,
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(into(models::PublishAgentsTeamToFireskyReq))]
pub struct PublishAgentsTeamToFireskyReq {
    pub pds_url: String,
    pub email: String,
    pub handle: String,
    pub password: String,
    pub invite_code: Option<String>,
}

#[derive(Debug, Clone, StructuralConvert, Object)]
#[convert(from(models::PublishAgentsTeamToFireskyResp))]
pub struct PublishAgentsTeamToFireskyResp {
    pub contract: PreparedContract,
}
