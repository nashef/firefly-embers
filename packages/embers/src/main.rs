use anyhow::Context;
use firefly_client::{NodeEvents, ReadNodeClient, WriteNodeClient};
use poem::listener::TcpListener;
use poem::middleware::{Compression, Cors, NormalizePath, RequestId, Tracing, TrailingSlash};
use poem::{EndpointExt, Route, Server};
use poem_openapi::OpenApiService;
use tokio::try_join;

use crate::ai_agents::api::AIAgents;
use crate::ai_agents::handlers::AgentsService;
use crate::ai_agents_teams::api::AIAgentsTeams;
use crate::ai_agents_teams::handlers::AgentsTeamsService;
use crate::common::api::Service;
use crate::configuration::collect_config;
use crate::testnet::api::Testnet;
use crate::testnet::handlers::TestnetService;
use crate::wallets::api::WalletsApi;
use crate::wallets::handlers::WalletsService;

mod ai_agents;
mod ai_agents_teams;
mod bootstrap;
mod common;
mod configuration;
mod testnet;
mod wallets;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = collect_config().context("failed to read configuration")?;

    let env_filter = tracing_subscriber::EnvFilter::try_new(config.log_level)
        .context("failed to init log filter")?;

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .init();

    let read_client = ReadNodeClient::new(config.mainnet.observer_url);
    let validator_node_events = NodeEvents::new(&config.mainnet.validator_ws_api_url);
    let observer_node_events = NodeEvents::new(&config.mainnet.observer_ws_api_url);

    let testnet_read_client = ReadNodeClient::new(config.testnet.observer_url);
    let _testnet_validator_node_events = NodeEvents::new(&config.testnet.validator_ws_api_url);
    let testnet_observer_node_events = NodeEvents::new(&config.testnet.observer_ws_api_url);

    let ((agents_service, agents_teams_service, wallets_service), testnet_service) = try_join!(
        async {
            let mut write_client = WriteNodeClient::new(
                config.mainnet.deploy_service_url,
                config.mainnet.propose_service_url,
            )
            .await?;

            let agents_service = AgentsService::bootstrap(
                write_client.clone(),
                read_client.clone(),
                &config.mainnet.service_key,
                &config.mainnet.agents_env_key,
            )
            .await?;

            let agents_teams_service = AgentsTeamsService::bootstrap(
                write_client.clone(),
                read_client.clone(),
                observer_node_events.clone(),
                &config.mainnet.service_key,
                &config.mainnet.agents_teams_env_key,
            )
            .await?;

            let wallets_service = WalletsService::bootstrap(
                write_client.clone(),
                read_client,
                validator_node_events,
                observer_node_events,
                &config.mainnet.service_key,
                &config.mainnet.wallets_env_key,
            )
            .await?;

            // Try to propose, but don't fail if validator is busy
            let _ = write_client.propose().await;

            anyhow::Ok((agents_service, agents_teams_service, wallets_service))
        },
        async {
            let mut testnet_write_client = WriteNodeClient::new(
                config.testnet.deploy_service_url,
                config.testnet.propose_service_url,
            )
            .await?;

            let testnet_service = TestnetService::bootstrap(
                testnet_write_client.clone(),
                testnet_read_client,
                testnet_observer_node_events,
                config.testnet.service_key,
                &config.testnet.env_key,
            )
            .await?;

            // Try to propose, but don't fail if validator is busy
            let _ = testnet_write_client.propose().await;

            anyhow::Ok(testnet_service)
        },
    )?;

    let api = OpenApiService::new(
        (Service, Testnet, WalletsApi, AIAgents, AIAgentsTeams),
        "Embers API",
        "0.1.0",
    )
    .url_prefix("/api");

    let ui = api.swagger_ui();
    let spec = api.spec_endpoint();
    let spec_yaml = api.spec_endpoint_yaml();

    let routes = Route::new()
        .nest("/api", api)
        .nest("/swagger-ui/index.html", ui)
        .nest("/swagger-ui/openapi.json", spec)
        .nest("/swagger-ui/openapi.yaml", spec_yaml)
        .data(agents_service)
        .data(agents_teams_service)
        .data(wallets_service)
        .data(testnet_service)
        .with(Cors::new().allow_origin_regex("*"))
        .with(RequestId::default())
        .with(Tracing)
        .with(Compression::default())
        .with(NormalizePath::new(TrailingSlash::Trim));

    Server::new(TcpListener::bind((config.address, config.port)))
        .run_with_graceful_shutdown(
            routes,
            async move {
                let _ = tokio::signal::ctrl_c()
                    .await
                    .inspect_err(|err| tracing::warn!("ctrl_c error: {err:?}"));
            },
            None,
        )
        .await?;

    Ok(())
}
