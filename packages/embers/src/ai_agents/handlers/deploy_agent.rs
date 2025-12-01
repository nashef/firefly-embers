use anyhow::Context;
use chrono::{DateTime, Utc};
use firefly_client::models::{DeployId, Uri};
use firefly_client::rendering::Render;

use crate::ai_agents::handlers::AgentsService;
use crate::ai_agents::models::{DeployAgentReq, DeployAgentResp, DeploySignedAgentReq};
use crate::common::prepare_for_signing;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents/record_deploy.rho")]
struct UpdateLastDeploy {
    env_uri: Uri,
    id: String,
    version: String,
    last_deploy: DateTime<Utc>,
}

impl AgentsService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(request),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn prepare_deploy_agent_contract(
        &self,
        request: DeployAgentReq,
    ) -> anyhow::Result<DeployAgentResp> {
        record_trace!(request);

        let valid_after = self.write_client.clone().get_head_block_index().await?;
        let (code, phlo_limit, system) = match request {
            DeployAgentReq::Agent {
                id,
                version,
                address,
                phlo_limit,
            } => {
                let code = self
                    .get_agent(address, id.clone(), version.clone())
                    .await?
                    .context("agent not found")?
                    .code
                    .context("agent has no code")?;

                let system_code = UpdateLastDeploy {
                    env_uri: self.uri.clone(),
                    id,
                    version,
                    last_deploy: Utc::now(),
                }
                .render()?;

                (
                    code,
                    phlo_limit,
                    Some(
                        prepare_for_signing()
                            .code(system_code)
                            .valid_after_block_number(valid_after)
                            .call(),
                    ),
                )
            }
            DeployAgentReq::Code { code, phlo_limit } => (code, phlo_limit, None),
        };

        Ok(DeployAgentResp {
            contract: prepare_for_signing()
                .code(code)
                .valid_after_block_number(valid_after)
                .phlo_limit(phlo_limit)
                .call(),
            system,
        })
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(request),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn deploy_signed_deploy_agent(
        &self,
        request: DeploySignedAgentReq,
    ) -> anyhow::Result<DeployId> {
        record_trace!(request);

        let mut write_client = self.write_client.clone();

        let deploy_id = write_client
            .deploy_signed_contract(request.contract)
            .await?;

        if let Some(system) = request.system {
            write_client.deploy_signed_contract(system).await?;
        }

        write_client.propose().await?;
        Ok(deploy_id)
    }
}
