use firefly_client::models::{Uri, WalletAddress};
use firefly_client::rendering::Render;

use crate::ai_agents::blockchain::dtos;
use crate::ai_agents::handlers::AgentsService;
use crate::ai_agents::models::Agents;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents/list_agents.rho")]
struct ListAgents {
    env_uri: Uri,
    address: WalletAddress,
}

impl AgentsService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(address),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn list_agents(&self, address: WalletAddress) -> anyhow::Result<Agents> {
        record_trace!(address);

        let code = ListAgents {
            env_uri: self.uri.clone(),
            address,
        }
        .render()?;
        self.read_client
            .get_data(code)
            .await
            .map(|agents: Vec<dtos::AgentHeader>| Agents {
                agents: agents.into_iter().map(Into::into).collect(),
            })
            .map_err(Into::into)
    }
}
