use std::sync::Arc;

use crate::alerts::manager::AlertManager;
use crate::format;

pub async fn handle(manager: Arc<AlertManager>, owner_pubkey_hex: &str) -> String {
    match manager.list_alerts(owner_pubkey_hex).await {
        Ok(alerts) => format::format_alert_list(&alerts),
        Err(e) => e.user_message(),
    }
}
