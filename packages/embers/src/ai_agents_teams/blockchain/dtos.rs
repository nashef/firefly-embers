use derive_more::Into;
use serde::{Deserialize, de};
use structural_convert::StructuralConvert;

use crate::ai_agents_teams::models;
use crate::common::blockchain;

#[derive(Debug, Clone, StructuralConvert, Deserialize)]
#[convert(into(models::AgentsTeams))]
pub struct AgentsTeams {
    pub agents_teams: Vec<AgentsTeamHeader>,
}

#[derive(Debug, Clone, StructuralConvert, Deserialize)]
#[convert(into(models::AgentsTeamHeader))]
pub struct AgentsTeamHeader {
    pub id: String,
    pub version: String,
    pub created_at: blockchain::dtos::DateTime,
    pub last_deploy: Option<blockchain::dtos::DateTime>,
    pub name: String,
    pub description: Option<String>,
    pub shard: Option<String>,
    pub logo: Option<String>,
}

#[derive(Debug, Clone, Into)]
pub struct Graph(models::Graph);

impl<'de> Deserialize<'de> for Graph {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let graphl = String::deserialize(deserializer)?;
        models::Graph::new(graphl)
            .map(Self)
            .map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, StructuralConvert, Deserialize)]
#[convert(into(models::AgentsTeam))]
pub struct AgentsTeam {
    pub id: String,
    pub version: String,
    pub created_at: blockchain::dtos::DateTime,
    pub last_deploy: Option<blockchain::dtos::DateTime>,
    pub uri: Option<blockchain::dtos::Uri>,
    pub name: String,
    pub description: Option<String>,
    pub shard: Option<String>,
    pub logo: Option<String>,
    pub graph: Option<Graph>,
}
