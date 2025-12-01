use firefly_client::models::{Uri, WalletAddress};
use firefly_client::rendering::Render;

use crate::ai_agents::blockchain::dtos;
use crate::ai_agents::handlers::AgentsService;
use crate::ai_agents::models::Agent;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents/get_agent.rho")]
struct GetAgent {
    env_uri: Uri,
    address: WalletAddress,
    id: String,
    version: String,
}

impl AgentsService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(address, id, version),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn get_agent(
        &self,
        address: WalletAddress,
        id: String,
        version: String,
    ) -> anyhow::Result<Option<Agent>> {
        record_trace!(address, id, version);

        let code = GetAgent {
            env_uri: self.uri.clone(),
            address,
            id,
            version,
        }
        .render()?;

        let agent: Option<dtos::Agent> = self.read_client.get_data(code).await?;
        Ok(agent.map(Into::into))
    }
}
