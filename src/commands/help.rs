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
         /closing                        Markets closing in next 24 hours\n\
         /portfolio <address> [limit]    Portfolio snapshot (value + positions + trades)\n\
         /positions <address> [limit]    Open positions by wallet\n\
         /trades <address> [limit]       Recent trades by wallet\n\
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
         /portfolio 0x56687bf447db6ffa42ffe2204a05edaa20f55839\n\
         /positions 0x56687bf447db6ffa42ffe2204a05edaa20f55839 8\n\
         /trades 0x56687bf447db6ffa42ffe2204a05edaa20f55839 8\n\
         /alert add will-bitcoin-hit-100k above 52\n\
         /alert list",
    )
}
