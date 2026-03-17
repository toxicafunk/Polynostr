/// Return the help text listing all available commands.
pub fn help_text() -> String {
    String::from(
        "Polynostr Bot — Polymarket data on Nostr\n\
         \n\
         Commands:\n\
         /search <query>    Search for prediction markets\n\
         /price <slug>      Get current price for a market\n\
         /market <slug>     Detailed market information\n\
         /trending          Top active markets\n\
         /help              Show this message\n\
         \n\
         Examples:\n\
         /search bitcoin\n\
         /price will-bitcoin-hit-100k\n\
         /trending",
    )
}
