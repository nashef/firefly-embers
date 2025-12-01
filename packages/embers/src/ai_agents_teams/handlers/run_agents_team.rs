use std::time::Duration;

use anyhow::anyhow;
use firefly_client::models::{DeployId, SignedCode, Uri};
use firefly_client::rendering::Render;

use crate::ai_agents_teams::handlers::AgentsTeamsService;
use crate::ai_agents_teams::models::{RunAgentsTeamReq, RunAgentsTeamResp};
use crate::common::prepare_for_signing;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents_teams/run_agents_team.rho")]
struct RunAgentsTeam {
    agents_team: Uri,
    prompt: String,
}

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents_teams/get_run_agents_team_result.rho")]
struct GetAgentsTeamResult {
    deploy_id: DeployId,
}

impl AgentsTeamsService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(request),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn prepare_run_agents_team_contract(
        &self,
        request: RunAgentsTeamReq,
    ) -> anyhow::Result<RunAgentsTeamResp> {
        record_trace!(request);

        let contract = RunAgentsTeam {
            agents_team: request.agents_team,
            prompt: request.prompt,
        }
        .render()?;

        let valid_after = self.write_client.clone().get_head_block_index().await?;
        Ok(RunAgentsTeamResp {
            contract: prepare_for_signing()
                .code(contract)
                .phlo_limit(request.phlo_limit)
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
    pub async fn deploy_signed_run_agents_team(
        &self,
        contract: SignedCode,
    ) -> anyhow::Result<serde_json::Value> {
        record_trace!(contract);

        let mut write_client = self.write_client.clone();

        let deploy_id = write_client.deploy_signed_contract(contract).await?;

        let deploy_waiter = self
            .observer_node_events
            .wait_for_deploy(&deploy_id, Duration::from_secs(60));
        let (_, finalized) =
            tokio::try_join!(write_client.propose(), async { Ok(deploy_waiter.await) })?;

        if !finalized {
            return Err(anyhow!("block is not finalized"));
        }

        let code = GetAgentsTeamResult { deploy_id }.render()?;
        self.read_client.get_data(code).await.map_err(Into::into)
    }
}
