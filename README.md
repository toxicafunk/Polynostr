# Polynostr

A Nostr bot that bridges Polymarket prediction market data into the Nostr protocol. Query prediction markets, prices, and trending events through direct messages or public mentions.

## Features (Phase 1)

- **Search Markets**: Find prediction markets by keyword
- **Get Prices**: Check current Yes/No prices for any market
- **Trending Markets**: List top active markets by volume
- **Market Details**: Get comprehensive information about any market
- **Privacy-First**: Supports NIP-17 Gift Wrap private DMs

## Requirements

- Rust 1.88.0+ (edition 2024 support required)
- A Nostr secret key (nsec)
- Access to Nostr relays

**Note**: The project uses Rustls 0.23 with the `aws-lc-rs` crypto provider, which is automatically installed at runtime.

## Setup

1. **Clone and build:**

```bash
cd /Users/ericrodriguez/src/rust/polynostr
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
/help              Show available commands
/search <query>    Search for prediction markets
/price <slug>      Get current price for a market
/market <slug>     Detailed market information
/trending          Top active markets
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

### How to Test

1. Start the bot and note its npub (public key) from the logs
2. Open any Nostr client (Damus, Amethyst, Primal, Snort, etc.)
3. Send a DM to the bot's npub with `/help`
4. Try the example commands above

The bot supports both:
- **Private DMs (NIP-17)**: Send encrypted direct messages for private queries
- **Public mentions**: Mention the bot's npub in a public note

## Architecture

```
Nostr Relays ←WebSocket→ polynostr bot ←HTTP→ Polymarket APIs
                         │
                         ├─ nostr-sdk v0.44 (NIP-17 DMs)
                         └─ polymarket-client-sdk v0.4 (Gamma API)
```

### APIs Used

- **Polymarket Gamma API**: Public market data, search, events (no auth required)
- **Polymarket Data API**: Volume, open interest (future phase)
- **Nostr Protocol**: NIP-01 (basic), NIP-17 (private DMs)

## Development

### Project Structure

```
src/
├── main.rs           # Entry point
├── config.rs         # Environment configuration
├── bot.rs            # Event loop and message handling
├── format.rs         # Response formatting
├── commands/         # Command handlers
│   ├── mod.rs
│   ├── help.rs
│   ├── search.rs
│   ├── price.rs
│   ├── trending.rs
│   └── market.rs
└── polymarket/       # Polymarket API wrappers
    ├── mod.rs
    ├── gamma.rs
    └── data.rs
```

### Logging

Set the `RUST_LOG` environment variable to control log verbosity:

```bash
RUST_LOG=info cargo run       # Default
RUST_LOG=debug cargo run      # Verbose
RUST_LOG=polynostr=trace cargo run  # Very verbose
```

## Roadmap

- **Phase 1** (✅ Complete): Basic read-only bot with search, price, trending commands
- **Phase 2** (Planned): Real-time price alerts via WebSocket streaming
- **Phase 3** (Planned): User portfolio tracking by wallet address
- **Phase 4** (Planned): Trading commands with server-side EVM signer
- **Phase 5** (Planned): Optional web dashboard

## License

MIT

## Credits

Built with:
- [nostr-sdk](https://github.com/rust-nostr/nostr) by Rust Nostr Developers
- [polymarket-client-sdk](https://github.com/Polymarket/rs-clob-client) by Polymarket
