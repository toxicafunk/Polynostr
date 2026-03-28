/// Return the help text listing all available commands.
pub fn help_text() -> String {
    String::from(
        "Polynostr Bot — Polymarket data on Nostr\n\
         \n\
         Commands:\n\
         /search <query>                 Search for prediction markets\n\
         /price <slug>                   Get current price for a market\n\
         /market <slug>                  Detailed market information\n\
         /trending                       Top active markets\n\
         /alert add <slug> <rule> <v>   Create alert (rules: above/below/move)\n\
         /alert list                     List your alerts\n\
         /alert remove <id>              Remove alert\n\
         /alert pause <id>               Pause alert\n\
         /alert resume <id>              Resume alert\n\
         /alert test <id>                Send test notification\n\
         /help                           Show this message\n\
         \n\
         Examples:\n\
         /search bitcoin\n\
         /price will-bitcoin-hit-100k\n\
         /alert add will-bitcoin-hit-100k above 52\n\
         /alert list",
    )
}
