use firefly_client::models::{DeployId, SignedCode, Uri};
use firefly_client::rendering::Render;

use crate::ai_agents::handlers::AgentsService;
use crate::ai_agents::models::DeleteAgentResp;
use crate::common::prepare_for_signing;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents/delete_agent.rho")]
struct DeleteAgent {
    env_uri: Uri,
    id: String,
}

impl AgentsService {
    #[tracing::instrument(level = "info", skip(self), err(Debug), ret(Debug, level = "trace"))]
    pub async fn prepare_delete_agent_contract(
        &self,
        id: String,
    ) -> anyhow::Result<DeleteAgentResp> {
        let contract = DeleteAgent {
            env_uri: self.uri.clone(),
            id,
        }
        .render()?;

        let valid_after = self.write_client.clone().get_head_block_index().await?;
        Ok(DeleteAgentResp {
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
    pub async fn deploy_signed_delete_agent(
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
