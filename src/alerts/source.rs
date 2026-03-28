use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{RwLock, mpsc};

use crate::alerts::error::AlertError;
use crate::alerts::model::MarketTick;
use crate::polymarket::gamma::GammaClient;

#[async_trait]
pub trait MarketUpdateSource: Send + Sync {
    async fn subscribe(&self) -> Result<mpsc::Receiver<MarketTick>, AlertError>;
    async fn set_tracked_slugs(&self, slugs: HashSet<String>) -> Result<(), AlertError>;
}

#[derive(Clone)]
pub struct PollingMarketUpdateSource {
    gamma: Arc<GammaClient>,
    interval_seconds: u64,
    tracked_slugs: Arc<RwLock<HashSet<String>>>,
}

impl PollingMarketUpdateSource {
    pub fn new(gamma: Arc<GammaClient>, interval_seconds: u64) -> Self {
        Self {
            gamma,
            interval_seconds,
            tracked_slugs: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

#[async_trait]
impl MarketUpdateSource for PollingMarketUpdateSource {
    async fn subscribe(&self) -> Result<mpsc::Receiver<MarketTick>, AlertError> {
        let (tx, rx) = mpsc::channel(256);
        let gamma = self.gamma.clone();
        let interval_seconds = self.interval_seconds.max(3);
        let tracked_slugs = self.tracked_slugs.clone();

        tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(std::time::Duration::from_secs(interval_seconds));
            loop {
                ticker.tick().await;
                let slugs: Vec<String> = tracked_slugs.read().await.iter().cloned().collect();
                for slug in &slugs {
                    match gamma.get_market_by_slug(slug).await {
                        Ok(market) => {
                            if let Some(prices) = &market.outcome_prices {
                                if let Some(price) = prices.first() {
                                    let price = price.to_string().parse::<f64>().unwrap_or(0.0);
                                    if tx
                                        .send(MarketTick {
                                            slug: slug.clone(),
                                            price,
                                            seen_at: chrono::Utc::now(),
                                        })
                                        .await
                                        .is_err()
                                    {
                                        return;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            tracing::warn!(slug = %slug, error = %err, "Polling tick fetch failed");
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn set_tracked_slugs(&self, slugs: HashSet<String>) -> Result<(), AlertError> {
        let mut guard = self.tracked_slugs.write().await;
        *guard = slugs;
        Ok(())
    }
}

#[derive(Clone)]
pub struct WsMarketUpdateSource {
    fallback: PollingMarketUpdateSource,
}

impl WsMarketUpdateSource {
    pub fn new(fallback: PollingMarketUpdateSource) -> Self {
        Self { fallback }
    }
}

#[async_trait]
impl MarketUpdateSource for WsMarketUpdateSource {
    async fn subscribe(&self) -> Result<mpsc::Receiver<MarketTick>, AlertError> {
        tracing::info!("WebSocket source unavailable in current SDK path; using polling fallback");
        self.fallback.subscribe().await
    }

    async fn set_tracked_slugs(&self, slugs: HashSet<String>) -> Result<(), AlertError> {
        self.fallback.set_tracked_slugs(slugs).await
    }
}
