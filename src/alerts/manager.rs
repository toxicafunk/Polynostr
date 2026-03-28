use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::alerts::error::AlertError;
use crate::alerts::evaluator::{AlertEvaluator, EvaluatorConfig};
use crate::alerts::model::{
    AlertRule, AlertStatus, AlertSubscription, DeliveryChannel, DeliveryTarget,
};
use crate::alerts::notifier::AlertDelivery;
use crate::alerts::repository::AlertRepository;
use crate::alerts::source::MarketUpdateSource;

#[derive(Debug, Clone)]
pub struct AlertManagerConfig {
    pub max_alerts_per_user: usize,
    pub cooldown_seconds: u64,
    pub hysteresis_bps: u32,
    pub notifications_per_minute: usize,
    pub refresh_seconds: u64,
    pub stream_enabled: bool,
    pub reconnect_backoff_initial_seconds: u64,
    pub reconnect_backoff_max_seconds: u64,
}

#[derive(Clone)]
pub struct AlertManager {
    repo: Arc<dyn AlertRepository>,
    source: Arc<dyn MarketUpdateSource>,
    notifier: Arc<dyn AlertDelivery>,
    evaluator: Arc<AlertEvaluator>,
    config: AlertManagerConfig,
    subscribed_slugs: Arc<RwLock<HashSet<String>>>,
    notif_window: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
}

impl AlertManager {
    pub fn new(
        repo: Arc<dyn AlertRepository>,
        source: Arc<dyn MarketUpdateSource>,
        notifier: Arc<dyn AlertDelivery>,
        config: AlertManagerConfig,
    ) -> Self {
        let evaluator = AlertEvaluator::new(EvaluatorConfig {
            cooldown_seconds: config.cooldown_seconds,
            hysteresis_bps: config.hysteresis_bps,
        });
        Self {
            repo,
            source,
            notifier,
            evaluator: Arc::new(evaluator),
            config,
            subscribed_slugs: Arc::new(RwLock::new(HashSet::new())),
            notif_window: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_alert(
        &self,
        owner_pubkey_hex: String,
        channel: DeliveryChannel,
        slug: String,
        rule: AlertRule,
    ) -> Result<AlertSubscription, AlertError> {
        let count = self.repo.count_by_pubkey(&owner_pubkey_hex).await?;
        if count >= self.config.max_alerts_per_user {
            return Err(AlertError::MaxAlertsReached(
                self.config.max_alerts_per_user,
            ));
        }

        let alert = AlertSubscription::new(
            slug.clone(),
            rule,
            DeliveryTarget {
                pubkey: owner_pubkey_hex,
                channel,
            },
        );
        let created = self.repo.create(alert).await?;

        let active = self.repo.list_active().await?;
        let slugs: HashSet<String> = active.iter().map(|a| a.slug.clone()).collect();
        self.source.set_tracked_slugs(slugs.clone()).await?;
        *self.subscribed_slugs.write().await = slugs;

        Ok(created)
    }

    pub async fn list_alerts(
        &self,
        owner_pubkey_hex: &str,
    ) -> Result<Vec<AlertSubscription>, AlertError> {
        self.repo.list_by_pubkey(owner_pubkey_hex).await
    }

    pub async fn remove_alert(
        &self,
        owner_pubkey_hex: &str,
        alert_id: &str,
    ) -> Result<(), AlertError> {
        self.repo
            .delete_by_owner(alert_id, owner_pubkey_hex)
            .await?;
        let active = self.repo.list_active().await?;
        let slugs: HashSet<String> = active.iter().map(|a| a.slug.clone()).collect();
        self.source.set_tracked_slugs(slugs.clone()).await?;
        *self.subscribed_slugs.write().await = slugs;
        Ok(())
    }

    pub async fn pause_alert(
        &self,
        owner_pubkey_hex: &str,
        alert_id: &str,
    ) -> Result<AlertSubscription, AlertError> {
        self.repo
            .set_status_by_owner(alert_id, owner_pubkey_hex, AlertStatus::Paused)
            .await
    }

    pub async fn resume_alert(
        &self,
        owner_pubkey_hex: &str,
        alert_id: &str,
    ) -> Result<AlertSubscription, AlertError> {
        self.repo
            .set_status_by_owner(alert_id, owner_pubkey_hex, AlertStatus::Active)
            .await
    }

    pub async fn send_test(
        &self,
        owner_pubkey_hex: &str,
        alert_id: &str,
    ) -> Result<(), AlertError> {
        let Some(alert) = self.repo.get(alert_id).await? else {
            return Err(AlertError::AlertNotFound(alert_id.to_owned()));
        };
        if alert.delivery.pubkey != owner_pubkey_hex {
            return Err(AlertError::Unauthorized);
        }
        self.notifier.send_test(&alert).await
    }

    pub async fn run(self: Arc<Self>) -> Result<(), AlertError> {
        let active = self.repo.list_active().await?;
        let slugs: HashSet<String> = active.iter().map(|a| a.slug.clone()).collect();

        if slugs.is_empty() {
            tracing::info!("Alert manager started with no active subscriptions");
        }

        self.source.set_tracked_slugs(slugs.clone()).await?;

        {
            let mut guard = self.subscribed_slugs.write().await;
            for slug in &slugs {
                guard.insert(slug.clone());
            }
        }

        let mut updates = self.source.subscribe().await?;
        tracing::info!(
            stream_enabled = self.config.stream_enabled,
            refresh_seconds = self.config.refresh_seconds,
            reconnect_initial_seconds = self.config.reconnect_backoff_initial_seconds,
            reconnect_max_seconds = self.config.reconnect_backoff_max_seconds,
            "Alert manager processing market updates"
        );

        while let Some(tick) = updates.recv().await {
            let alerts = self.repo.list_active().await?;
            for alert in alerts.into_iter().filter(|a| a.slug == tick.slug) {
                let (event, seen_price) = self.evaluator.evaluate(&alert, &tick);
                if let Some(triggered) = event {
                    if !self.allow_notification(&alert.delivery.pubkey).await {
                        tracing::warn!(
                            alert_id = %alert.id,
                            owner = %alert.delivery.pubkey,
                            "Alert notification dropped due to throughput limit"
                        );
                        continue;
                    }
                    tracing::info!(
                        alert_id = %alert.id,
                        slug = %alert.slug,
                        price = triggered.price,
                        "Alert triggered"
                    );
                    self.notifier.send_trigger(&alert, &triggered).await?;
                    self.repo
                        .update_trigger_state(
                            &alert.id,
                            seen_price,
                            Some(triggered.price),
                            Some(triggered.triggered_at),
                        )
                        .await?;
                } else {
                    self.repo
                        .update_trigger_state(&alert.id, seen_price, None, None)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn allow_notification(&self, owner_pubkey_hex: &str) -> bool {
        let mut guard = self.notif_window.write().await;
        let window = guard
            .entry(owner_pubkey_hex.to_owned())
            .or_insert_with(Vec::new);
        let now = Utc::now();
        window.retain(|ts| (now - *ts).num_seconds() < 60);
        if window.len() >= self.config.notifications_per_minute {
            return false;
        }
        window.push(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    use async_trait::async_trait;
    use tokio::sync::{RwLock, mpsc};

    use super::*;
    use crate::alerts::error::AlertError;
    use crate::alerts::model::{
        AlertEvent, AlertRule, AlertSubscription, DeliveryChannel, MarketTick,
    };
    use crate::alerts::repository::memory::InMemoryAlertRepository;
    use crate::alerts::source::MarketUpdateSource;

    #[derive(Clone, Default)]
    struct StubDelivery {
        sent: Arc<RwLock<Vec<String>>>,
    }

    #[async_trait]
    impl crate::alerts::notifier::AlertDelivery for StubDelivery {
        async fn send_trigger(
            &self,
            alert: &AlertSubscription,
            _event: &AlertEvent,
        ) -> Result<(), AlertError> {
            self.sent.write().await.push(alert.id.clone());
            Ok(())
        }

        async fn send_test(&self, _alert: &AlertSubscription) -> Result<(), AlertError> {
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct StubSource {
        tracked: Arc<RwLock<HashSet<String>>>,
        tx: Arc<RwLock<Option<mpsc::Sender<MarketTick>>>>,
    }

    #[async_trait]
    impl MarketUpdateSource for StubSource {
        async fn subscribe(&self) -> Result<mpsc::Receiver<MarketTick>, AlertError> {
            let (tx, rx) = mpsc::channel(32);
            *self.tx.write().await = Some(tx);
            Ok(rx)
        }

        async fn set_tracked_slugs(&self, slugs: HashSet<String>) -> Result<(), AlertError> {
            *self.tracked.write().await = slugs;
            Ok(())
        }
    }

    #[tokio::test]
    async fn add_alert_updates_tracked_slugs() {
        let repo = Arc::new(InMemoryAlertRepository::default());
        let source = Arc::new(StubSource::default());
        let notifier: Arc<dyn crate::alerts::notifier::AlertDelivery> =
            Arc::new(StubDelivery::default());
        let manager = AlertManager::new(
            repo,
            source.clone(),
            notifier,
            AlertManagerConfig {
                max_alerts_per_user: 10,
                cooldown_seconds: 10,
                hysteresis_bps: 25,
                notifications_per_minute: 5,
                refresh_seconds: 5,
                stream_enabled: true,
                reconnect_backoff_initial_seconds: 1,
                reconnect_backoff_max_seconds: 10,
            },
        );

        manager
            .add_alert(
                "abcd".to_owned(),
                DeliveryChannel::Nip17,
                "btc-100k".to_owned(),
                AlertRule::Above { threshold: 0.55 },
            )
            .await
            .expect("add alert");

        let tracked = source.tracked.read().await.clone();
        assert!(tracked.contains("btc-100k"));
    }

    #[tokio::test]
    async fn notification_window_caps_throughput() {
        let repo = Arc::new(InMemoryAlertRepository::default());
        let source = Arc::new(StubSource::default());
        let notifier: Arc<dyn crate::alerts::notifier::AlertDelivery> =
            Arc::new(StubDelivery::default());
        let manager = AlertManager {
            repo,
            source,
            notifier,
            evaluator: Arc::new(AlertEvaluator::new(EvaluatorConfig {
                cooldown_seconds: 1,
                hysteresis_bps: 10,
            })),
            config: AlertManagerConfig {
                max_alerts_per_user: 10,
                cooldown_seconds: 1,
                hysteresis_bps: 10,
                notifications_per_minute: 1,
                refresh_seconds: 5,
                stream_enabled: true,
                reconnect_backoff_initial_seconds: 1,
                reconnect_backoff_max_seconds: 5,
            },
            subscribed_slugs: Arc::new(RwLock::new(HashSet::new())),
            notif_window: Arc::new(RwLock::new(HashMap::new())),
        };

        assert!(manager.allow_notification("pk1").await);
        assert!(!manager.allow_notification("pk1").await);
    }

    #[tokio::test]
    async fn end_to_end_command_to_trigger_notification_flow() {
        let repo = Arc::new(InMemoryAlertRepository::default());
        let source = Arc::new(StubSource::default());
        let delivery = Arc::new(StubDelivery::default());
        let notifier: Arc<dyn crate::alerts::notifier::AlertDelivery> = delivery.clone();

        let manager = Arc::new(AlertManager::new(
            repo,
            source.clone(),
            notifier,
            AlertManagerConfig {
                max_alerts_per_user: 10,
                cooldown_seconds: 1,
                hysteresis_bps: 1,
                notifications_per_minute: 10,
                refresh_seconds: 1,
                stream_enabled: true,
                reconnect_backoff_initial_seconds: 1,
                reconnect_backoff_max_seconds: 5,
            },
        ));

        let add_response = crate::commands::alert_add::handle_add(
            manager.clone(),
            "pubkey-e2e".to_owned(),
            DeliveryChannel::Nip17,
            "btc-100k above 52",
        )
        .await;
        assert!(add_response.contains("Alert created"));

        let runner = {
            let manager = manager.clone();
            tokio::spawn(async move {
                let _ = manager.run().await;
            })
        };

        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

        let tx = source
            .tx
            .read()
            .await
            .clone()
            .expect("source sender initialized by run");

        tx.send(MarketTick {
            slug: "btc-100k".to_owned(),
            price: 0.50,
            seen_at: Utc::now(),
        })
        .await
        .expect("seed tick");

        tx.send(MarketTick {
            slug: "btc-100k".to_owned(),
            price: 0.53,
            seen_at: Utc::now(),
        })
        .await
        .expect("crossing tick");

        tokio::time::sleep(std::time::Duration::from_millis(80)).await;

        let sent = delivery.sent.read().await;
        assert_eq!(sent.len(), 1);

        runner.abort();
        let _ = runner.await;
    }
}
