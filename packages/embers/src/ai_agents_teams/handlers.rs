use aes_gcm::{Aes256Gcm, Key};
use firefly_client::models::Uri;
use firefly_client::{NodeEvents, ReadNodeClient, WriteNodeClient};

mod create_agents_team;
mod delete_agents_team;
mod deploy_agents_team;
mod get_agents_team;
mod list_agents_team_versions;
mod list_agents_teams;
mod publish_agents_team_to_firesky;
mod run_agents_team;
mod save_agents_team;

#[derive(Clone)]
pub struct AgentsTeamsService {
    pub uri: Uri,
    pub write_client: WriteNodeClient,
    pub read_client: ReadNodeClient,
    pub observer_node_events: NodeEvents,
    pub aes_encryption_key: Key<Aes256Gcm>,
}
