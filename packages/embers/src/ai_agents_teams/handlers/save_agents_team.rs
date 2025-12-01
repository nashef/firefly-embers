use chrono::{DateTime, Utc};
use firefly_client::models::{DeployId, SignedCode, Uri};
use firefly_client::rendering::Render;
use uuid::Uuid;

use crate::ai_agents_teams::handlers::AgentsTeamsService;
use crate::ai_agents_teams::models::{Graph, SaveAgentsTeamReq, SaveAgentsTeamResp};
use crate::common::prepare_for_signing;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents_teams/save_agents_team.rho")]
struct SaveAgentsTeam {
    env_uri: Uri,
    id: String,
    version: Uuid,
    created_at: DateTime<Utc>,
    name: String,
    description: Option<String>,
    shard: Option<String>,
    logo: Option<String>,
    graph: Option<String>,
}

impl AgentsTeamsService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(id, request),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn prepare_save_agents_team_contract(
        &self,
        id: String,
        request: SaveAgentsTeamReq,
    ) -> anyhow::Result<SaveAgentsTeamResp> {
        record_trace!(id, request);

        let version = Uuid::now_v7();

        let contract = SaveAgentsTeam {
            env_uri: self.uri.clone(),
            id,
            version,
            created_at: Utc::now(),
            name: request.name,
            description: request.description,
            shard: request.shard,
            logo: request.logo,
            graph: request.graph.map(Graph::graphl),
        }
        .render()?;

        let valid_after = self.write_client.clone().get_head_block_index().await?;
        Ok(SaveAgentsTeamResp {
            version: version.into(),
            contract: prepare_for_signing()
                .code(contract)
                .valid_after_block_number(valid_after)
                .call(),
        })
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(contract),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn deploy_signed_save_agents_team(
        &self,
        contract: SignedCode,
    ) -> anyhow::Result<DeployId> {
        record_trace!(contract);

        let mut write_client = self.write_client.clone();

        let deploy_id = write_client.deploy_signed_contract(contract).await?;
        write_client.propose().await?;
        Ok(deploy_id)
    }
}
