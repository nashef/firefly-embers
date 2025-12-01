use firefly_client::models::WalletAddress;
use futures::future;
use futures::sink::SinkExt;
use poem::web::{Data, websocket};
use poem_openapi::OpenApi;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::types::ToJSON;
use tracing::error;

use crate::common::api::dtos::{ApiTags, SendResp, SignedContract, Stringified};
use crate::wallets::api::dtos::{DeployEvent, TransferReq, TransferResp, WalletStateAndHistory};
use crate::wallets::handlers::WalletsService;

mod dtos;

#[derive(Debug, Clone)]
pub struct WalletsApi;

#[OpenApi(prefix_path = "/wallets", tag = ApiTags::Wallets)]
impl WalletsApi {
    #[oai(path = "/:address/state", method = "get")]
    async fn wallet_state_and_history(
        &self,
        Path(address): Path<Stringified<WalletAddress>>,
        Data(wallets): Data<&WalletsService>,
    ) -> poem::Result<Json<WalletStateAndHistory>> {
        let wallet_state_and_history = wallets
            .get_wallet_state_and_history(address.0.clone())
            .await
            .map_err(|err| {
                error!(
                    address = ?address.0,
                    error = %err,
                    error_debug = ?err,
                    backtrace = %std::backtrace::Backtrace::force_capture(),
                    "Failed to get wallet state and history"
                );
                err
            })
            .map(Into::into)?;

        Ok(Json(wallet_state_and_history))
    }

    #[oai(path = "/transfer/prepare", method = "post")]
    async fn prepare_transfer(
        &self,
        Json(body): Json<TransferReq>,
        Data(wallets): Data<&WalletsService>,
    ) -> poem::Result<Json<TransferResp>> {
        let input = body.try_into()?;
        let result = wallets
            .prepare_transfer_contract(input)
            .await
            .map_err(|err| {
                error!(
                    error = %err,
                    error_debug = ?err,
                    backtrace = %std::backtrace::Backtrace::force_capture(),
                    "Failed to prepare transfer contract"
                );
                err
            })?;

        Ok(Json(TransferResp {
            contract: result.into(),
        }))
    }

    #[oai(path = "/transfer/send", method = "post")]
    async fn transfer(
        &self,
        Json(body): Json<SignedContract>,
        Data(wallets): Data<&WalletsService>,
    ) -> poem::Result<Json<SendResp>> {
        let deploy_id = wallets
            .deploy_signed_transfer(body.into())
            .await
            .map_err(|err| {
                error!(
                    error = %err,
                    error_debug = ?err,
                    backtrace = %std::backtrace::Backtrace::force_capture(),
                    "Failed to deploy signed transfer"
                );
                err
            })?;
        Ok(Json(deploy_id.into()))
    }

    #[allow(clippy::unused_async)]
    #[oai(path = "/:address/deploys", method = "get")]
    async fn deploys(
        &self,
        Path(address): Path<Stringified<WalletAddress>>,
        Data(wallets): Data<&WalletsService>,
        ws: websocket::WebSocket,
    ) -> websocket::BoxWebSocketUpgraded {
        let wallets = wallets.clone();

        ws.on_upgrade(move |socket| {
            let sink = socket.with(|msg| {
                let msg = DeployEvent::from(msg).to_json_string();
                future::ok(websocket::Message::Text(msg))
            });
            wallets.subscribe_to_deploys(address.0, sink);
            future::ready(())
        })
        .boxed()
    }
}
