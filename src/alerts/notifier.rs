use std::sync::Arc;

use async_trait::async_trait;
use nostr_sdk::prelude::*;

use crate::alerts::error::AlertError;
use crate::alerts::model::{AlertEvent, AlertSubscription, DeliveryChannel};
use crate::format;

#[async_trait]
pub trait AlertDelivery: Send + Sync {
    async fn send_trigger(
        &self,
        alert: &AlertSubscription,
        event: &AlertEvent,
    ) -> Result<(), AlertError>;
    async fn send_test(&self, alert: &AlertSubscription) -> Result<(), AlertError>;
}

#[derive(Clone)]
pub struct AlertNotifier {
    client: Arc<Client>,
}

impl AlertNotifier {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AlertDelivery for AlertNotifier {
    async fn send_trigger(
        &self,
        alert: &AlertSubscription,
        event: &AlertEvent,
    ) -> Result<(), AlertError> {
        let body = format::format_alert_trigger(alert, event.price, event.triggered_at);
        let pubkey = PublicKey::from_hex(&alert.delivery.pubkey).map_err(|e| {
            AlertError::Notification(format!(
                "invalid recipient pubkey {}: {e}",
                alert.delivery.pubkey
            ))
        })?;

        match alert.delivery.channel {
            DeliveryChannel::Nip17 => {
                self.client
                    .send_private_msg(pubkey, body, [])
                    .await
                    .map_err(|e| AlertError::Notification(e.to_string()))?;
            }
            DeliveryChannel::Nip04 => {
                let signer = self
                    .client
                    .signer()
                    .await
                    .map_err(|e| AlertError::Notification(e.to_string()))?;
                let encrypted = signer
                    .nip04_encrypt(&pubkey, &body)
                    .await
                    .map_err(|e| AlertError::Notification(e.to_string()))?;
                let dm = EventBuilder::new(Kind::EncryptedDirectMessage, encrypted)
                    .tag(Tag::public_key(pubkey));
                self.client
                    .send_event_builder(dm)
                    .await
                    .map_err(|e| AlertError::Notification(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn send_test(&self, alert: &AlertSubscription) -> Result<(), AlertError> {
        let body = format::format_alert_test(alert);
        let pubkey = PublicKey::from_hex(&alert.delivery.pubkey)
            .map_err(|e| AlertError::Notification(format!("invalid recipient pubkey: {e}")))?;
        self.client
            .send_private_msg(pubkey, body, [])
            .await
            .map_err(|e| AlertError::Notification(e.to_string()))?;
        Ok(())
    }
}
