pub mod help;
pub mod market;
pub mod price;
pub mod search;
pub mod trending;

use crate::polymarket::gamma::GammaClient;

/// Parse a message and route to the appropriate command handler.
///
/// Commands start with `/`. If no recognized command is found, returns help text.
pub async fn handle_command(gamma: &GammaClient, message: &str) -> String {
    let trimmed = message.trim();

    // Split into command and arguments
    let (command, args) = match trimmed.split_once(char::is_whitespace) {
        Some((cmd, rest)) => (cmd, rest),
        None => (trimmed, ""),
    };

    match command.to_lowercase().as_str() {
        "/search" | "search" => search::handle(gamma, args).await,
        "/price" | "price" => price::handle(gamma, args).await,
        "/market" | "market" => market::handle(gamma, args).await,
        "/trending" | "trending" => trending::handle(gamma).await,
        "/help" | "help" | "/start" => help::help_text(),
        _ => {
            format!(
                "Unknown command: \"{}\"\n\n{}",
                command,
                help::help_text()
            )
        }
    }
}
