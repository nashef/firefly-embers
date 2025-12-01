use firefly_client::models::{DeployId, SignedCode, Uri};
use firefly_client::rendering::Render;

use crate::ai_agents_teams::handlers::AgentsTeamsService;
use crate::ai_agents_teams::models::DeleteAgentsTeamResp;
use crate::common::prepare_for_signing;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents_teams/delete_agents_team.rho")]
struct DeleteAgentsTeam {
    env_uri: Uri,
    id: String,
}

impl AgentsTeamsService {
    #[tracing::instrument(level = "info", skip(self), err(Debug), ret(Debug, level = "trace"))]
    pub async fn prepare_delete_agents_team_contract(
        &self,
        id: String,
    ) -> anyhow::Result<DeleteAgentsTeamResp> {
        let contract = DeleteAgentsTeam {
            env_uri: self.uri.clone(),
            id,
        }
        .render()?;

        let valid_after = self.write_client.clone().get_head_block_index().await?;
        Ok(DeleteAgentsTeamResp {
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
    pub async fn deploy_signed_delete_agents_team(
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
