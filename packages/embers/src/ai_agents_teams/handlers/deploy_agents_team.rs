use anyhow::Context;
use chrono::{DateTime, Utc};
use firefly_client::models::{DeployId, Uri};
use firefly_client::rendering::Render;

use crate::ai_agents_teams::compilation::{parse, render_agent_team};
use crate::ai_agents_teams::handlers::AgentsTeamsService;
use crate::ai_agents_teams::models::{
    DeployAgentsTeamReq,
    DeployAgentsTeamResp,
    DeploySignedAgentsTeamtReq,
};
use crate::common::prepare_for_signing;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents_teams/record_deploy.rho")]
struct RecordDeploy {
    env_uri: Uri,
    id: String,
    last_deploy: DateTime<Utc>,
    uri: Uri,
}

impl AgentsTeamsService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(request),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn prepare_deploy_agents_team_contract(
        &self,
        request: DeployAgentsTeamReq,
    ) -> anyhow::Result<DeployAgentsTeamResp> {
        record_trace!(request);

        let valid_after = self.write_client.clone().get_head_block_index().await?;
        let (graph, phlo_limit, deploy, system) = match request {
            DeployAgentsTeamReq::AgentsTeam {
                id,
                version,
                address,
                phlo_limit,
                deploy,
            } => {
                let graph = self
                    .get_agents_team(address, id.clone(), version.clone())
                    .await?
                    .context("agents team not found")?
                    .graph
                    .context("agents team has no graph")?;

                let system_code = RecordDeploy {
                    env_uri: self.uri.clone(),
                    id,
                    last_deploy: Utc::now(),
                    uri: deploy.uri_pub_key.into(),
                }
                .render()?;

                (
                    graph,
                    phlo_limit,
                    deploy,
                    Some(
                        prepare_for_signing()
                            .code(system_code)
                            .valid_after_block_number(valid_after)
                            .call(),
                    ),
                )
            }
            DeployAgentsTeamReq::Graph {
                graph,
                phlo_limit,
                deploy,
            } => (graph, phlo_limit, deploy, None),
        };

        let timestamp = deploy.timestamp;

        let code = parse(&graph)?;
        let code = render_agent_team(code, deploy)?;

        Ok(DeployAgentsTeamResp {
            contract: prepare_for_signing()
                .code(code)
                .valid_after_block_number(valid_after)
                .timestamp(timestamp)
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
    pub async fn deploy_signed_deploy_agents_team(
        &self,
        request: DeploySignedAgentsTeamtReq,
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
