use anyhow::Context;
use serde_json::Value;
use tracing::info;

use crate::errors::ReadNodeError;
use crate::models::ReadNodeExpr;

#[derive(Clone)]
pub struct ReadNodeClient {
    url: String,
    client: reqwest::Client,
}

impl ReadNodeClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: Default::default(),
        }
    }

    pub async fn get_data<T>(&self, rholang_code: String) -> Result<T, ReadNodeError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut response_json = self.explore_deploy(rholang_code).await?;

        info!(response_json = %response_json, "explore_deploy response");

        let data_value = response_json
            .pointer_mut("/expr/0")
            .map(Value::take)
            .ok_or(ReadNodeError::ReturnValueMissing)?;

        let intermediate: ReadNodeExpr = serde_json::from_value(data_value)
            .context("failed to deserialize intermediate model")
            .map_err(ReadNodeError::Deserialization)?;

        serde_json::from_value(intermediate.into())
            .context("failed to deserialize filed model")
            .map_err(ReadNodeError::Deserialization)
    }

    async fn explore_deploy(&self, rholang_code: String) -> Result<Value, ReadNodeError> {
        let request = self
            .client
            .post(format!("{}/api/explore-deploy", self.url))
            .body(rholang_code)
            .header("Content-Type", "text/plain")
            .send()
            .await?;

        if !request.status().is_success() {
            let status = request.status();
            let body = request.text().await?;
            return Err(ReadNodeError::Api(status, body));
        }

        request.json().await.map_err(Into::into)
    }
}
