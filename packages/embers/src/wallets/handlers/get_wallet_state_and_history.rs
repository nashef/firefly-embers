use firefly_client::models::{Either, Uri, WalletAddress};
use firefly_client::rendering::Render;

use crate::common::tracing::record_trace;
use crate::wallets::blockchain::dtos;
use crate::wallets::handlers::WalletsService;
use crate::wallets::models::WalletStateAndHistory;

#[derive(Debug, Clone, Render)]
#[template(path = "wallets/get_balance_and_history.rho")]
struct GetBalanceAndHistory {
    env_uri: Uri,
    wallet_address: WalletAddress,
}

impl WalletsService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(address),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn get_wallet_state_and_history(
        &self,
        address: WalletAddress,
    ) -> anyhow::Result<WalletStateAndHistory> {
        record_trace!(address);

        let contract = GetBalanceAndHistory {
            env_uri: self.uri.clone(),
            wallet_address: address.clone(),
        }
        .render()?;

        let state = self
            .read_client
            .get_data::<Either<String, dtos::BalanceAndHistory>>(contract)
            .await?
            .to_result()
            .map_err(|err| anyhow::anyhow!("error from contract: {err}"))?;

        Ok(WalletStateAndHistory {
            balance: state.balance,
            transfers: state
                .transfers
                .into_iter()
                .flat_map(TryFrom::try_from)
                .collect(),
            boosts: state
                .boosts
                .into_iter()
                .flat_map(TryFrom::try_from)
                .collect(),
            exchanges: vec![],
            requests: vec![],
        })
    }
}
