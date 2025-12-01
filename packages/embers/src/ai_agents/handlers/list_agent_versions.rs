use firefly_client::models::{Uri, WalletAddress};
use firefly_client::rendering::Render;

use crate::ai_agents::blockchain::dtos;
use crate::ai_agents::handlers::AgentsService;
use crate::ai_agents::models::Agents;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents/list_agent_versions.rho")]
struct ListAgentVersions {
    env_uri: Uri,
    address: WalletAddress,
    id: String,
}

impl AgentsService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(address, id),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn list_agent_versions(
        &self,
        address: WalletAddress,
        id: String,
    ) -> anyhow::Result<Option<Agents>> {
        record_trace!(address, id);

        let code = ListAgentVersions {
            env_uri: self.uri.clone(),
            address,
            id,
        }
        .render()?;

        let agents: Option<Vec<dtos::AgentHeader>> = self.read_client.get_data(code).await?;
        Ok(agents.map(|mut agents| {
            agents.sort_by(|l, r| l.version.cmp(&r.version));
            Agents {
                agents: agents.into_iter().map(Into::into).collect(),
            }
        }))
    }
}
