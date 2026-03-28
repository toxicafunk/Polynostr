use std::sync::Arc;

use crate::alerts::manager::AlertManager;
use crate::format;

pub async fn handle(manager: Arc<AlertManager>, owner_pubkey_hex: &str, args: &str) -> String {
    let alert_id = args.trim();
    if alert_id.is_empty() {
        return "Usage: alert resume <alert-id>".to_owned();
    }

    match manager.resume_alert(owner_pubkey_hex, alert_id).await {
        Ok(_) => format::format_alert_resumed(alert_id),
        Err(e) => e.user_message(),
    }
}
