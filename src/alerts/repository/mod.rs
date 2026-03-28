use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::alerts::error::AlertError;
use crate::alerts::model::{AlertStatus, AlertSubscription, DeliveryTarget};

#[async_trait]
pub trait AlertRepository: Send + Sync {
    async fn create(&self, alert: AlertSubscription) -> Result<AlertSubscription, AlertError>;
    async fn list_by_pubkey(&self, pubkey: &str) -> Result<Vec<AlertSubscription>, AlertError>;
    async fn list_active(&self) -> Result<Vec<AlertSubscription>, AlertError>;
    async fn get(&self, alert_id: &str) -> Result<Option<AlertSubscription>, AlertError>;
    async fn delete_by_owner(&self, alert_id: &str, owner_pubkey: &str) -> Result<(), AlertError>;
    async fn set_status_by_owner(
        &self,
        alert_id: &str,
        owner_pubkey: &str,
        status: AlertStatus,
    ) -> Result<AlertSubscription, AlertError>;
    async fn update_trigger_state(
        &self,
        alert_id: &str,
        last_seen_price: Option<f64>,
        last_triggered_price: Option<f64>,
        last_triggered_at: Option<DateTime<Utc>>,
    ) -> Result<(), AlertError>;
    async fn count_by_pubkey(&self, pubkey: &str) -> Result<usize, AlertError>;
    async fn update_delivery_by_owner(
        &self,
        alert_id: &str,
        owner_pubkey: &str,
        delivery: DeliveryTarget,
    ) -> Result<AlertSubscription, AlertError>;
}

pub mod memory;
pub mod sqlite;
