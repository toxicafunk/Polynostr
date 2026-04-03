# Polynostr

A Nostr bot that bridges Polymarket prediction market data into the Nostr protocol. Query prediction markets, prices, and trending events through direct messages or public mentions.

## Features (Phase 1 + Phase 2)

- **Search Markets**: Find prediction markets by keyword
- **Get Prices**: Check current Yes/No prices for any market
- **Trending Markets**: List top active markets by volume
- **Market Details**: Get comprehensive information about any market
- **User Alerts**: Create, list, pause/resume, remove, and test price alerts
- **Real-Time Notifications**: Background alert evaluation with private DM delivery
- **Persistence**: Alert subscriptions and trigger state survive restarts (SQLite)
- **Privacy-First**: Supports NIP-17 Gift Wrap private DMs (with NIP-04 compatibility)

## Requirements

- Rust 1.88.0+ (edition 2024 support required)
- A Nostr secret key (nsec)
- Access to Nostr relays

**Note**: The project uses Rustls 0.23 with the `ring` crypto provider for maximum compatibility across different systems and compilers.

## Setup

1. **Clone and build:**

```bash
cd [WORK_DIR]/polynostr
cargo build --release
```

2. **Configure environment:**

```bash
cp .env.example .env
# Edit .env and set your NOSTR_SECRET_KEY
```

To generate a new Nostr key:
- Use https://nostrtool.com/
- Or use the nostr-sdk CLI: `nostr-keygen`

3. **Run the bot:**

```bash
cargo run --release
```

The bot will print its public key (npub) on startup. Save this to send it messages.

## Usage

### Commands

Send any of these commands via DM or mention:

```
/help                           Show available commands
/search <query>                 Search for prediction markets
/price <slug>                   Get current price for a market
/market <slug>                  Detailed market information
/trending                       Top active markets
/alert add <slug> <rule> <v>    Create alert (rules: above|below|move)
/alert list                     List your alerts
/alert remove <alert-id>        Remove alert
/alert pause <alert-id>         Pause alert
/alert resume <alert-id>        Resume alert
/alert test <alert-id>          Send test notification
```

### Examples

**Search for markets:**
```
/search bitcoin
```

**Get current prices:**
```
/price will-bitcoin-hit-100k
```

**List trending markets:**
```
/trending
```

**Get detailed market info:**
```
/market will-bitcoin-hit-100k
```

**Create an alert when price crosses above 52¢:**
```
/alert add will-bitcoin-hit-100k above 52
```

**List your alerts:**
```
/alert list
```

**Pause and resume an alert:**
```
/alert pause <alert-id>
/alert resume <alert-id>
```

**Send a test alert notification:**
```
/alert test <alert-id>
```

### How to Test

1. Start the bot and note its npub (public key) from the logs
2. Open any Nostr client (Damus, Amethyst, Primal, Snort, etc.)
3. Send a DM to the bot's npub with `/help`
4. Try the example commands above

The bot supports both:
- **Private DMs (NIP-17)**: Send encrypted direct messages for private queries and alert notifications
- **Public mentions**: Mention the bot's npub in a public note (commands still supported)

## Architecture

```
Nostr Relays ←WebSocket→ polynostr bot ←HTTP→ Polymarket APIs
                         │
                         ├─ nostr-sdk v0.44 (NIP-17/NIP-04 messaging)
                         ├─ alert manager (rules, evaluator, notifier)
                         │   ├─ market update source (WebSocket-first, polling fallback)
                         │   └─ SQLite persistence (subscriptions + trigger state)
                         └─ polymarket-client-sdk v0.4 (Gamma API)
```

### APIs Used

- **Polymarket Gamma API**: Public market data, search, events, and alert polling fallback (no auth required)
- **Polymarket Data API**: Volume, open interest (future phase)
- **Nostr Protocol**: NIP-01 (basic), NIP-17 (private DMs), NIP-04 (compatibility path)

## Development

### Project Structure

```
src/
├── main.rs           # Entry point and alert manager wiring
├── config.rs         # Environment configuration
├── bot.rs            # Event loop and message handling
├── format.rs         # Response and alert formatting
├── alerts/           # Alerting domain and runtime
│   ├── mod.rs
│   ├── model.rs
│   ├── parser.rs
│   ├── evaluator.rs
│   ├── notifier.rs
│   ├── manager.rs
│   ├── source.rs
│   ├── error.rs
│   └── repository/
│       ├── mod.rs
│       ├── memory.rs
│       └── sqlite.rs
├── commands/         # Command handlers
│   ├── mod.rs
│   ├── help.rs
│   ├── search.rs
│   ├── price.rs
│   ├── trending.rs
│   ├── market.rs
│   ├── alert_add.rs
│   ├── alert_list.rs
│   ├── alert_remove.rs
│   ├── alert_pause.rs
│   ├── alert_resume.rs
│   └── alert_test.rs
└── polymarket/       # Polymarket API wrappers
    ├── mod.rs
    ├── gamma.rs
    └── data.rs
```

## Alert Configuration (Phase 2)

The alert system is controlled with environment variables:

- `ALERT_STREAM_ENABLED` (default: `true`)
- `ALERT_POLL_INTERVAL_SECONDS` (default: `15`)
- `ALERT_RECONNECT_BACKOFF_INITIAL_SECONDS` (default: `2`)
- `ALERT_RECONNECT_BACKOFF_MAX_SECONDS` (default: `60`)
- `ALERT_MAX_PER_USER` (default: `25`)
- `ALERT_COOLDOWN_SECONDS` (default: `120`)
- `ALERT_HYSTERESIS_BPS` (default: `50`)
- `ALERT_NOTIFICATIONS_PER_MINUTE` (default: `10`)
- `ALERT_DB_PATH` (default: `alerts.sqlite3`)


### Logging

Set the `RUST_LOG` environment variable to control log verbosity:

```bash
RUST_LOG=info cargo run       # Default
RUST_LOG=debug cargo run      # Verbose
RUST_LOG=polynostr=trace cargo run  # Very verbose
```

## Roadmap

- **Phase 1** (✅ Complete): Basic read-only bot with search, price, trending commands
- **Phase 2** (✅ Complete): Real-time price alerts with persistent subscriptions and DM notifications
- **Phase 3** (Planned): User portfolio tracking by wallet address
- **Phase 4** (Planned): Trading commands with server-side EVM signer
- **Phase 5** (Planned): Optional web dashboard

## Troubleshooting

### Build fails on Ubuntu 20.04 LTS with GCC compiler error

If you encounter an error like:
```
error: failed to run custom build command for `aws-lc-sys v0.39.1`
### COMPILER BUG DETECTED ###
Your compiler (cc) is not supported due to a memcmp related bug...
```

This is caused by a transitive dependency (`polymarket-client-sdk`) that uses `aws-lc-rs` crypto provider, which doesn't support older GCC versions in Ubuntu 20.04 LTS.

**The project includes a `.cargo/config.toml` that automatically sets the required environment variable**, so builds should work out of the box on Ubuntu 20.04. The compiler check is disabled automatically via Cargo configuration.

If you prefer to use a newer compiler for production deployments, you can upgrade GCC:

```bash
sudo add-apt-repository ppa:ubuntu-toolchain-r/test
sudo apt update
sudo apt install gcc-11 g++-11
sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-11 110
sudo update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-11 110
```

Alternatively, use Ubuntu 22.04 LTS (Jammy) or later, which comes with GCC 11+ by default.

## License

MIT

## Credits

Built with:
- [nostr-sdk](https://github.com/rust-nostr/nostr) by Rust Nostr Developers
- [polymarket-client-sdk](https://github.com/Polymarket/rs-clob-client) by Polymarket
