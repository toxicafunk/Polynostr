use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryChannel {
    Nip17,
    Nip04,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeliveryTarget {
    pub pubkey: String,
    pub channel: DeliveryChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertRule {
    Above { threshold: f64 },
    Below { threshold: f64 },
    PercentMove { percent: f64 },
}

impl AlertRule {
    pub fn describe(&self) -> String {
        match self {
            AlertRule::Above { threshold } => format!("above {:.2}¢", threshold * 100.0),
            AlertRule::Below { threshold } => format!("below {:.2}¢", threshold * 100.0),
            AlertRule::PercentMove { percent } => format!("moves by {:.2}%", percent),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    Active,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerState {
    pub last_seen_price: Option<f64>,
    pub last_triggered_price: Option<f64>,
    pub last_triggered_at: Option<DateTime<Utc>>,
}

impl TriggerState {
    pub fn new() -> Self {
        Self {
            last_seen_price: None,
            last_triggered_price: None,
            last_triggered_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSubscription {
    pub id: String,
    pub slug: String,
    pub rule: AlertRule,
    pub status: AlertStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub delivery: DeliveryTarget,
    pub trigger_state: TriggerState,
}

impl AlertSubscription {
    pub fn new(slug: String, rule: AlertRule, delivery: DeliveryTarget) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            slug,
            rule,
            status: AlertStatus::Active,
            created_at: now,
            updated_at: now,
            delivery,
            trigger_state: TriggerState::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AlertEvent {
    pub alert_id: String,
    pub slug: String,
    pub price: f64,
    pub triggered_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MarketTick {
    pub slug: String,
    pub price: f64,
    pub seen_at: DateTime<Utc>,
}
