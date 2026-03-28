use std::sync::Arc;

use crate::alerts::manager::AlertManager;
use crate::format;

pub async fn handle(manager: Arc<AlertManager>, owner_pubkey_hex: &str, args: &str) -> String {
    let alert_id = args.trim();
    if alert_id.is_empty() {
        return "Usage: alert pause <alert-id>".to_owned();
    }

    match manager.pause_alert(owner_pubkey_hex, alert_id).await {
        Ok(_) => format::format_alert_paused(alert_id),
        Err(e) => e.user_message(),
    }
}
