use chrono::{DateTime, Utc};
use firefly_client::models::{DeployId, SignedCode, Uri, WalletAddress};
use firefly_client::rendering::Render;

use crate::common::models::PreparedContract;
use crate::common::prepare_for_signing;
use crate::common::tracing::record_trace;
use crate::wallets::handlers::WalletsService;
use crate::wallets::models::BoostReq;

#[derive(Debug, Clone, Render)]
#[template(path = "wallets/boost.rho")]
struct BoostContract {
    env_uri: Uri,
    timestamp: DateTime<Utc>,
    wallet_address_from: WalletAddress,
    wallet_address_to: WalletAddress,
    amount: i64,
    description: Option<String>,
    post_author_did: String,
    post_id: Option<String>,
}

impl WalletsService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(request),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn prepare_boost_contract(
        &self,
        request: BoostReq,
    ) -> anyhow::Result<PreparedContract> {
        record_trace!(request);

        let contract = BoostContract {
            env_uri: self.uri.clone(),
            timestamp: Utc::now(),
            wallet_address_from: request.from,
            wallet_address_to: request.to,
            amount: request.amount.0,
            description: request.description,
            post_author_did: request.post_author_did,
            post_id: request.post_id,
        }
        .render()?;

        let valid_after = self.write_client.clone().get_head_block_index().await?;
        Ok(prepare_for_signing()
            .code(contract)
            .valid_after_block_number(valid_after)
            .call())
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(contract),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn deploy_boost_transfer(&self, contract: SignedCode) -> anyhow::Result<DeployId> {
        record_trace!(contract);

        let mut write_client = self.write_client.clone();

        let deploy_id = write_client.deploy_signed_contract(contract).await?;
        write_client.propose().await?;
        Ok(deploy_id)
    }
}
