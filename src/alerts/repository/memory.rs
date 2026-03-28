use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::alerts::error::AlertError;
use crate::alerts::model::{AlertStatus, AlertSubscription, DeliveryTarget};
use crate::alerts::repository::AlertRepository;

#[derive(Clone, Default)]
pub struct InMemoryAlertRepository {
    alerts: Arc<RwLock<HashMap<String, AlertSubscription>>>,
}

#[async_trait]
impl AlertRepository for InMemoryAlertRepository {
    async fn create(&self, alert: AlertSubscription) -> Result<AlertSubscription, AlertError> {
        let mut guard = self.alerts.write().await;
        guard.insert(alert.id.clone(), alert.clone());
        Ok(alert)
    }

    async fn list_by_pubkey(&self, pubkey: &str) -> Result<Vec<AlertSubscription>, AlertError> {
        let guard = self.alerts.read().await;
        Ok(guard
            .values()
            .filter(|a| a.delivery.pubkey == pubkey)
            .cloned()
            .collect())
    }

    async fn list_active(&self) -> Result<Vec<AlertSubscription>, AlertError> {
        let guard = self.alerts.read().await;
        Ok(guard
            .values()
            .filter(|a| a.status == AlertStatus::Active)
            .cloned()
            .collect())
    }

    async fn get(&self, alert_id: &str) -> Result<Option<AlertSubscription>, AlertError> {
        let guard = self.alerts.read().await;
        Ok(guard.get(alert_id).cloned())
    }

    async fn delete_by_owner(&self, alert_id: &str, owner_pubkey: &str) -> Result<(), AlertError> {
        let mut guard = self.alerts.write().await;
        let Some(alert) = guard.get(alert_id) else {
            return Err(AlertError::AlertNotFound(alert_id.to_owned()));
        };
        if alert.delivery.pubkey != owner_pubkey {
            return Err(AlertError::Unauthorized);
        }
        guard.remove(alert_id);
        Ok(())
    }

    async fn set_status_by_owner(
        &self,
        alert_id: &str,
        owner_pubkey: &str,
        status: AlertStatus,
    ) -> Result<AlertSubscription, AlertError> {
        let mut guard = self.alerts.write().await;
        let Some(alert) = guard.get_mut(alert_id) else {
            return Err(AlertError::AlertNotFound(alert_id.to_owned()));
        };
        if alert.delivery.pubkey != owner_pubkey {
            return Err(AlertError::Unauthorized);
        }
        alert.status = status;
        alert.updated_at = Utc::now();
        Ok(alert.clone())
    }

    async fn update_trigger_state(
        &self,
        alert_id: &str,
        last_seen_price: Option<f64>,
        last_triggered_price: Option<f64>,
        last_triggered_at: Option<DateTime<Utc>>,
    ) -> Result<(), AlertError> {
        let mut guard = self.alerts.write().await;
        let Some(alert) = guard.get_mut(alert_id) else {
            return Err(AlertError::AlertNotFound(alert_id.to_owned()));
        };
        alert.trigger_state.last_seen_price = last_seen_price;
        if last_triggered_price.is_some() {
            alert.trigger_state.last_triggered_price = last_triggered_price;
        }
        if last_triggered_at.is_some() {
            alert.trigger_state.last_triggered_at = last_triggered_at;
        }
        alert.updated_at = Utc::now();
        Ok(())
    }

    async fn count_by_pubkey(&self, pubkey: &str) -> Result<usize, AlertError> {
        Ok(self
            .list_by_pubkey(pubkey)
            .await?
            .into_iter()
            .filter(|a| a.status == AlertStatus::Active)
            .count())
    }

    async fn update_delivery_by_owner(
        &self,
        alert_id: &str,
        owner_pubkey: &str,
        delivery: DeliveryTarget,
    ) -> Result<AlertSubscription, AlertError> {
        let mut guard = self.alerts.write().await;
        let Some(alert) = guard.get_mut(alert_id) else {
            return Err(AlertError::AlertNotFound(alert_id.to_owned()));
        };
        if alert.delivery.pubkey != owner_pubkey {
            return Err(AlertError::Unauthorized);
        }
        alert.delivery = delivery;
        alert.updated_at = Utc::now();
        Ok(alert.clone())
    }
}
