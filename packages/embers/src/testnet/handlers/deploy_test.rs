use std::time::Duration;

use anyhow::anyhow;
use firefly_client::models::{DeployId, Uri};
use firefly_client::rendering::Render;

use crate::common::prepare_for_signing;
use crate::common::tracing::record_trace;
use crate::testnet::blockchain;
use crate::testnet::handlers::TestnetService;
use crate::testnet::models::{
    DeploySignedTestReq,
    DeploySignedTestResp,
    DeployTestReq,
    DeployTestResp,
};

#[derive(Debug, Clone, Render)]
#[template(path = "testnet/get_logs.rho")]
struct GetLogs {
    env_uri: Uri,
    deploy_id: DeployId,
}

impl TestnetService {
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(request),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn prepare_test_contract(
        &self,
        request: DeployTestReq,
    ) -> anyhow::Result<DeployTestResp> {
        record_trace!(request);

        let valid_after = self.write_client.clone().get_head_block_index().await?;
        Ok(DeployTestResp {
            env_contract: request.env.map(|env| {
                prepare_for_signing()
                    .code(env)
                    .valid_after_block_number(valid_after)
                    .call()
            }),
            test_contract: prepare_for_signing()
                .code(request.test)
                .valid_after_block_number(valid_after)
                .call(),
        })
    }

    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(request),
        err(Debug),
        ret(Debug, level = "trace")
    )]
    pub async fn deploy_test_contract(
        &self,
        request: DeploySignedTestReq,
    ) -> anyhow::Result<DeploySignedTestResp> {
        record_trace!(request);

        let mut write_client = self.write_client.clone();

        if let Some(contract) = request.env {
            let result = write_client.deploy_signed_contract(contract).await;
            if let Err(err) = result {
                return Ok(DeploySignedTestResp::EnvDeployFailed {
                    error: err.to_string(),
                });
            }

            write_client.propose().await?;
        }

        let result = write_client.deploy_signed_contract(request.test).await;
        let deploy_id = match result {
            Ok(deploy_id) => deploy_id,
            Err(err) => {
                return Ok(DeploySignedTestResp::TestDeployFailed {
                    error: err.to_string(),
                });
            }
        };

        let deploy_waiter = self
            .observer_node_events
            .wait_for_deploy(&deploy_id, Duration::from_secs(60));
        let (_, finalized) =
            tokio::try_join!(write_client.propose(), async { Ok(deploy_waiter.await) })?;

        if !finalized {
            return Err(anyhow!("block is not finalized"));
        }

        let code = GetLogs {
            deploy_id,
            env_uri: self.uri.clone(),
        }
        .render()?;

        let logs: Option<Vec<blockchain::dtos::Log>> = self.read_client.get_data(code).await?;

        Ok(DeploySignedTestResp::Ok {
            logs: logs
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
}
