use firefly_client::models::{Uri, WalletAddress};
use firefly_client::rendering::Render;

use crate::ai_agents_teams::blockchain::dtos;
use crate::ai_agents_teams::handlers::AgentsTeamsService;
use crate::ai_agents_teams::models::AgentsTeam;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents_teams/get_agents_team.rho")]
struct GetAgentsTeam {
    env_uri: Uri,
    address: WalletAddress,
    id: String,
    version: String,
}

impl AgentsTeamsService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(address, id, version),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn get_agents_team(
        &self,
        address: WalletAddress,
        id: String,
        version: String,
    ) -> anyhow::Result<Option<AgentsTeam>> {
        record_trace!(address, id, version);

        let code = GetAgentsTeam {
            env_uri: self.uri.clone(),
            address,
            id,
            version,
        }
        .render()?;

        let agents_team: Option<dtos::AgentsTeam> = self.read_client.get_data(code).await?;
        Ok(agents_team.map(Into::into))
    }
}
