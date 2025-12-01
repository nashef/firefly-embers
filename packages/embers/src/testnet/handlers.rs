use firefly_client::models::Uri;
use firefly_client::{NodeEvents, ReadNodeClient, WriteNodeClient};
use secp256k1::SecretKey;

mod create_test_wallet;
mod deploy_test;

#[derive(Clone)]
pub struct TestnetService {
    pub uri: Uri,
    pub service_key: SecretKey,
    pub write_client: WriteNodeClient,
    pub read_client: ReadNodeClient,
    pub observer_node_events: NodeEvents,
}
