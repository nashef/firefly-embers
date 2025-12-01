use firefly_client::models::WalletAddress;
use poem::web::Data;
use poem_openapi::OpenApi;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;

use crate::ai_agents_teams::api::dtos::{
    AgentsTeam,
    AgentsTeams,
    CreateAgentsTeamReq,
    CreateAgentsTeamResp,
    DeleteAgentsTeamResp,
    DeployAgentsTeamReq,
    DeployAgentsTeamResp,
    DeploySignedAgentsTeamtReq,
    PublishAgentsTeamToFireskyReq,
    PublishAgentsTeamToFireskyResp,
    RunAgentsTeamReq,
    RunAgentsTeamResp,
    SaveAgentsTeamReq,
    SaveAgentsTeamResp,
};
use crate::ai_agents_teams::handlers::AgentsTeamsService;
use crate::common::api::dtos::{ApiTags, MaybeNotFound, SendResp, SignedContract, Stringified};

mod dtos;

#[derive(Debug, Clone)]
pub struct AIAgentsTeams;

#[OpenApi(prefix_path = "/ai-agents-teams", tag = ApiTags::AIAgentsTeams)]
impl AIAgentsTeams {
    #[oai(path = "/:address", method = "get")]
    async fn list(
        &self,
        Path(address): Path<Stringified<WalletAddress>>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<AgentsTeams>> {
        let agents_teams = agents_teams.list_agents_teams(address.0).await?;
        Ok(Json(agents_teams.into()))
    }

    #[oai(path = "/:address/:id/versions", method = "get")]
    async fn list_versions(
        &self,
        Path(address): Path<Stringified<WalletAddress>>,
        Path(id): Path<String>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> MaybeNotFound<AgentsTeams> {
        agents_teams
            .list_agents_team_versions(address.0, id)
            .await
            .into()
    }

    #[oai(path = "/:address/:id/versions/:version", method = "get")]
    async fn get(
        &self,
        Path(address): Path<Stringified<WalletAddress>>,
        Path(id): Path<String>,
        Path(version): Path<String>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> MaybeNotFound<AgentsTeam> {
        agents_teams
            .get_agents_team(address.0, id, version)
            .await
            .into()
    }

    #[oai(path = "/create/prepare", method = "post")]
    async fn prepare_create(
        &self,
        Json(body): Json<CreateAgentsTeamReq>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<CreateAgentsTeamResp>> {
        let contract = agents_teams
            .prepare_create_agents_team_contract(body.into())
            .await?;
        Ok(Json(contract.into()))
    }

    #[oai(path = "/create/send", method = "post")]
    async fn create(
        &self,
        Json(body): Json<SignedContract>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<SendResp>> {
        let deploy_id = agents_teams
            .deploy_signed_create_agents_team(body.into())
            .await?;
        Ok(Json(deploy_id.into()))
    }

    #[oai(path = "/deploy/prepare", method = "post")]
    async fn prepare_deploy_agents_team(
        &self,
        Json(body): Json<DeployAgentsTeamReq>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<DeployAgentsTeamResp>> {
        let contract = agents_teams
            .prepare_deploy_agents_team_contract(body.into())
            .await?;
        Ok(Json(contract.into()))
    }

    #[oai(path = "/deploy/send", method = "post")]
    async fn deploy_agents_team(
        &self,
        Json(body): Json<DeploySignedAgentsTeamtReq>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<SendResp>> {
        let deploy_id = agents_teams
            .deploy_signed_deploy_agents_team(body.into())
            .await?;
        Ok(Json(deploy_id.into()))
    }

    #[oai(path = "/run/prepare", method = "post")]
    async fn prepare_run_agents_team(
        &self,
        Json(body): Json<RunAgentsTeamReq>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<RunAgentsTeamResp>> {
        let contract = agents_teams
            .prepare_run_agents_team_contract(body.into())
            .await?;
        Ok(Json(contract.into()))
    }

    #[oai(path = "/run/send", method = "post")]
    async fn run_agents_team(
        &self,
        Json(body): Json<SignedContract>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<serde_json::Value>> {
        agents_teams
            .deploy_signed_run_agents_team(body.into())
            .await
            .map(Json)
            .map_err(Into::into)
    }

    #[oai(path = "/:id/save/prepare", method = "post")]
    async fn prepare_save(
        &self,
        Path(id): Path<String>,
        Json(body): Json<SaveAgentsTeamReq>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<SaveAgentsTeamResp>> {
        let contract = agents_teams
            .prepare_save_agents_team_contract(id, body.into())
            .await?;
        Ok(Json(contract.into()))
    }

    #[oai(path = "/:id/save/send", method = "post")]
    async fn save(
        &self,
        #[allow(unused_variables)] Path(id): Path<String>,
        Json(body): Json<SignedContract>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<SendResp>> {
        let deploy_id = agents_teams
            .deploy_signed_save_agents_team(body.into())
            .await?;
        Ok(Json(deploy_id.into()))
    }

    #[oai(path = "/:id/delete/prepare", method = "post")]
    async fn prepare_delete(
        &self,
        Path(id): Path<String>,
        Data(agents): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<DeleteAgentsTeamResp>> {
        let contract = agents.prepare_delete_agents_team_contract(id).await?;
        Ok(Json(contract.into()))
    }

    #[oai(path = "/:id/delete/send", method = "post")]
    async fn delete(
        &self,
        #[allow(unused_variables)] Path(id): Path<String>,
        Json(body): Json<SignedContract>,
        Data(agents): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<SendResp>> {
        let deploy_id = agents.deploy_signed_delete_agents_team(body.into()).await?;
        Ok(Json(deploy_id.into()))
    }

    #[oai(path = "/:address/:id/publish-to-firesky/prepare", method = "post")]
    async fn prepare_publish_agents_team_to_firesky(
        &self,
        Path(address): Path<Stringified<WalletAddress>>,
        Path(id): Path<String>,
        Json(body): Json<PublishAgentsTeamToFireskyReq>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<PublishAgentsTeamToFireskyResp>> {
        let contract = agents_teams
            .prepare_publish_agents_team_to_firesky_contract(address.into(), id, body.into())
            .await?;
        Ok(Json(contract.into()))
    }

    #[oai(path = "/:address/:id/publish-to-firesky/send", method = "post")]
    async fn publish_agents_team_to_firesky(
        &self,
        #[allow(unused_variables)] Path(address): Path<Stringified<WalletAddress>>,
        #[allow(unused_variables)] Path(id): Path<String>,
        Json(body): Json<SignedContract>,
        Data(agents_teams): Data<&AgentsTeamsService>,
    ) -> poem::Result<Json<SendResp>> {
        let deploy_id = agents_teams
            .deploy_signed_publish_agents_team_to_firesky(body.into())
            .await?;
        Ok(Json(deploy_id.into()))
    }
}
