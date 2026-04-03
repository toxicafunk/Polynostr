pub mod alert_add;
pub mod alert_list;
pub mod alert_pause;
pub mod alert_remove;
pub mod alert_resume;
pub mod alert_test;
pub mod closing;
pub mod help;
pub mod market;
pub mod price;
pub mod search;
pub mod trending;

use std::sync::Arc;

use crate::alerts::manager::AlertManager;
use crate::alerts::model::DeliveryChannel;
use crate::polymarket::gamma::GammaClient;

/// Parse a message and route to the appropriate command handler.
///
/// Commands start with `/`. If no recognized command is found, returns help text.
pub async fn handle_command(
    gamma: &GammaClient,
    manager: Arc<AlertManager>,
    sender_pubkey_hex: Option<String>,
    channel: DeliveryChannel,
    message: &str,
) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return help::help_text();
    }

    let mut parts = trimmed.split_whitespace();
    let command = parts.next().unwrap_or("").to_lowercase();

    if command == "/alert" || command == "alert" {
        let Some(subcommand) = parts.next() else {
            return "Usage: alert <add|list|remove|pause|resume|test> ...".to_owned();
        };
        let args = parts.collect::<Vec<_>>().join(" ");
        let Some(owner_pubkey_hex) = sender_pubkey_hex else {
            return "Alert commands are available only for messages with a valid sender pubkey."
                .to_owned();
        };

        return match subcommand.to_lowercase().as_str() {
            "add" => alert_add::handle_add(manager, owner_pubkey_hex, channel, &args).await,
            "list" => alert_list::handle(manager, &owner_pubkey_hex).await,
            "remove" => alert_remove::handle(manager, &owner_pubkey_hex, &args).await,
            "pause" => alert_pause::handle(manager, &owner_pubkey_hex, &args).await,
            "resume" => alert_resume::handle(manager, &owner_pubkey_hex, &args).await,
            "test" => alert_test::handle(manager, &owner_pubkey_hex, &args).await,
            _ => "Unknown alert subcommand. Use: alert <add|list|remove|pause|resume|test>"
                .to_owned(),
        };
    }

    let (_, args) = match trimmed.split_once(char::is_whitespace) {
        Some((cmd, rest)) => (cmd, rest),
        None => (trimmed, ""),
    };

    match command.as_str() {
        "/search" | "search" => search::handle(gamma, args).await,
        "/price" | "price" => price::handle(gamma, args).await,
        "/market" | "market" => market::handle(gamma, args).await,
        "/trending" | "trending" => trending::handle(gamma).await,
        "/closing" | "closing" => closing::handle(gamma).await,
        "/help" | "help" | "/start" => help::help_text(),
        _ => format!("Unknown command: \"{}\"\n\n{}", command, help::help_text()),
    }
}
