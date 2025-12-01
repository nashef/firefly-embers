use std::sync::Arc;
use std::task::ready;
use std::time::Duration;

use backon::{ExponentialBuilder, Retryable};
use dashmap::DashMap;
use futures::{Stream, StreamExt};
use tokio::sync::{Notify, broadcast};
use tokio_stream::wrappers::BroadcastStream;
use tokio_tungstenite::tungstenite::Message;
use tracing::Instrument;
use uuid::Uuid;

use crate::models::{BlockEventDeploy, DeployId, NodeEvent, WalletAddress};

#[derive(Debug, Clone)]
pub enum DeployEvent {
    Finalized {
        id: DeployId,
        cost: u64,
        errored: bool,
    },
}

type DeploySubscriptions = Arc<DashMap<DeployId, DashMap<Uuid, Arc<Notify>>>>;
type WalletSubscriptions = Arc<DashMap<WalletAddress, broadcast::Sender<DeployEvent>>>;

#[derive(Clone)]
pub struct NodeEvents {
    deploy_subscriptions: DeploySubscriptions,
    wallet_subscriptions: WalletSubscriptions,
}

impl NodeEvents {
    pub fn new(url: &str) -> Self {
        let url = format!("{url}/ws/events");
        let tx = broadcast::Sender::<NodeEvent>::new(32);
        let deploy_subscriptions = DeploySubscriptions::default();
        let wallet_subscriptions = WalletSubscriptions::default();

        tokio::spawn({
            let tx = tx.clone();
            async move {
                loop {
                    let Ok((mut stream, _)) =
                        (|| async { tokio_tungstenite::connect_async(&url).await })
                            .retry(ExponentialBuilder::default().without_max_times())
                            .await
                    else {
                        return;
                    };

                    while let Some(msg) = stream.next().await {
                        let buff = match msg {
                            Ok(Message::Text(buff)) => buff,
                            Ok(event) => {
                                tracing::debug!("ignored ws event: {event:?}");
                                continue;
                            }
                            Err(err) => {
                                tracing::debug!("ws error: {err:?}");
                                continue;
                            }
                        };

                        let event = match serde_json::from_str(&buff) {
                            Ok(event) => event,
                            Err(err) => {
                                tracing::debug!("serde ws error: {err:?}");
                                continue;
                            }
                        };

                        let _ = tx.send(event);
                    }
                }
            }
            .in_current_span()
        });

        tokio::spawn({
            let mut rx = tx.subscribe();
            let deploy_subscriptions = deploy_subscriptions.clone();
            let wallet_subscriptions = wallet_subscriptions.clone();
            async move {
                loop {
                    let deploys = match rx.recv().await {
                        Ok(NodeEvent::Started) => continue,
                        Ok(NodeEvent::BlockAdded { .. }) => continue,
                        Ok(NodeEvent::BlockCreated { .. }) => continue,
                        Ok(NodeEvent::BlockFinalised { payload }) => payload.deploys,
                        Err(broadcast::error::RecvError::Closed) => return,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    };

                    for deploy in deploys {
                        deploy_subscriptions
                            .remove(&deploy.id)
                            .map(|(_, waiters)| waiters)
                            .into_iter()
                            .flatten()
                            .for_each(|(_, w)| w.notify_waiters());

                        if let Some(subscription) =
                            wallet_subscriptions.get(&deploy.deployer.into())
                        {
                            let _ = subscription.send(deploy.into());
                        }
                    }
                }
            }
            .in_current_span()
        });

        Self {
            deploy_subscriptions,
            wallet_subscriptions,
        }
    }

    pub fn wait_for_deploy(
        &self,
        deploy_id: &DeployId,
        max_wait: Duration,
    ) -> impl Future<Output = bool> {
        let id = Uuid::now_v7();

        let notify = Arc::<Notify>::default();
        let notified = notify.clone().notified_owned();

        self.deploy_subscriptions
            .entry(deploy_id.clone())
            .or_default()
            .insert(id, notify);

        let guard = scopeguard::guard(
            self.deploy_subscriptions.clone(),
            move |deploy_subscriptions| {
                deploy_subscriptions.remove_if(deploy_id, |_, submap| {
                    submap.remove(&id);
                    submap.is_empty()
                });
            },
        );

        async move {
            tokio::select! {
                _ = notified => {
                    scopeguard::ScopeGuard::into_inner(guard); // defuse
                    true
                },
                _ = tokio::time::sleep(max_wait) => false,
            }
        }
    }

    pub fn subscribe_for_deploys(&self, wallet_address: WalletAddress) -> WalletSubscription {
        let tx = self
            .wallet_subscriptions
            .entry(wallet_address.clone())
            .or_insert_with(|| broadcast::Sender::new(32));

        WalletSubscription {
            wallet_address,
            wallet_subscriptions: self.wallet_subscriptions.clone(),
            rx: BroadcastStream::new(tx.subscribe()),
        }
    }
}

impl From<BlockEventDeploy> for DeployEvent {
    fn from(value: BlockEventDeploy) -> Self {
        Self::Finalized {
            id: value.id,
            cost: value.cost,
            errored: value.errored,
        }
    }
}

pub struct WalletSubscription {
    wallet_address: WalletAddress,
    wallet_subscriptions: WalletSubscriptions,
    rx: BroadcastStream<DeployEvent>,
}

impl Stream for WalletSubscription {
    type Item = DeployEvent;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        ready!(self.rx.poll_next_unpin(cx))
            .transpose()
            .map(std::task::Poll::Ready)
            .unwrap_or(std::task::Poll::Pending)
    }
}

impl Drop for WalletSubscription {
    fn drop(&mut self) {
        self.wallet_subscriptions
            .remove_if(&self.wallet_address, |_, sender| {
                sender.receiver_count() == 0
            });
    }
}
