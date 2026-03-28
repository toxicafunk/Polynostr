use std::sync::Arc;

use crate::alerts::manager::AlertManager;

pub async fn handle(manager: Arc<AlertManager>, owner_pubkey_hex: &str, args: &str) -> String {
    let alert_id = args.trim();
    if alert_id.is_empty() {
        return "Usage: alert test <alert-id>".to_owned();
    }

    match manager.send_test(owner_pubkey_hex, alert_id).await {
        Ok(_) => format!("Test notification sent for alert: {alert_id}"),
        Err(e) => e.user_message(),
    }
}
