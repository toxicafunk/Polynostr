use std::sync::Arc;

use crate::alerts::manager::AlertManager;
use crate::alerts::model::DeliveryChannel;
use crate::alerts::parser;
use crate::format;

pub async fn handle_add(
    manager: Arc<AlertManager>,
    owner_pubkey_hex: String,
    channel: DeliveryChannel,
    args: &str,
) -> String {
    match parser::parse_add_args(args) {
        Ok((slug, rule)) => match manager
            .add_alert(owner_pubkey_hex, channel, slug, rule)
            .await
        {
            Ok(alert) => format::format_alert_created(&alert),
            Err(e) => e.user_message(),
        },
        Err(e) => e.user_message(),
    }
}
