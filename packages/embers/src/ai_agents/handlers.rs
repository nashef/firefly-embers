use firefly_client::models::Uri;
use firefly_client::{ReadNodeClient, WriteNodeClient};

mod create_agent;
mod delete_agent;
mod deploy_agent;
mod get_agent;
mod list_agent_versions;
mod list_agents;
mod save_agent;

#[derive(Clone)]
pub struct AgentsService {
    pub uri: Uri,
    pub write_client: WriteNodeClient,
    pub read_client: ReadNodeClient,
}
