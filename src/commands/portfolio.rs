use crate::format;
use crate::polymarket::data::DataClient;

const DEFAULT_LIMIT: i32 = 5;
const MAX_LIMIT: i32 = 20;

/// Handle the `/portfolio <wallet-address> [limit]` command.
pub async fn handle_portfolio(data: &DataClient, args: &str) -> String {
    let (address_input, limit) = match parse_address_and_limit(args, "portfolio") {
        Ok(parsed) => parsed,
        Err(err) => return err,
    };

    let address = match DataClient::normalize_address(&address_input) {
        Ok(address) => address,
        Err(err) => return err,
    };

    let (value_result, traded_result, positions_result, trades_result) = tokio::join!(
        data.get_total_value(&address),
        data.get_traded_markets(&address),
        data.get_positions(&address, limit),
        data.get_trades(&address, limit),
    );

    let positions = match positions_result {
        Ok(positions) => positions,
        Err(err) => return format!("Could not fetch positions for {address}.\n\n{err}"),
    };

    let trades = match trades_result {
        Ok(trades) => trades,
        Err(err) => return format!("Could not fetch trades for {address}.\n\n{err}"),
    };

    let total_value = value_result.ok().flatten().map(|entry| entry.value);
    let traded_markets = traded_result.ok().map(|entry| entry.traded);

    format::format_portfolio_overview(
        &address,
        total_value.as_ref(),
        traded_markets,
        &positions,
        &trades,
    )
}

/// Handle the `/positions <wallet-address> [limit]` command.
pub async fn handle_positions(data: &DataClient, args: &str) -> String {
    let (address_input, limit) = match parse_address_and_limit(args, "positions") {
        Ok(parsed) => parsed,
        Err(err) => return err,
    };

    let address = match DataClient::normalize_address(&address_input) {
        Ok(address) => address,
        Err(err) => return err,
    };

    match data.get_positions(&address, limit).await {
        Ok(positions) => format::format_positions(&address, &positions),
        Err(err) => format!("Could not fetch positions for {address}.\n\n{err}"),
    }
}

/// Handle the `/trades <wallet-address> [limit]` command.
pub async fn handle_trades(data: &DataClient, args: &str) -> String {
    let (address_input, limit) = match parse_address_and_limit(args, "trades") {
        Ok(parsed) => parsed,
        Err(err) => return err,
    };

    let address = match DataClient::normalize_address(&address_input) {
        Ok(address) => address,
        Err(err) => return err,
    };

    match data.get_trades(&address, limit).await {
        Ok(trades) => format::format_trades(&address, &trades),
        Err(err) => format!("Could not fetch trades for {address}.\n\n{err}"),
    }
}

fn parse_address_and_limit(args: &str, command: &str) -> Result<(String, i32), String> {
    let mut parts = args.split_whitespace();
    let Some(address) = parts.next() else {
        return Err(usage(command));
    };

    let limit = match parts.next() {
        Some(raw) => parse_limit(raw)?,
        None => DEFAULT_LIMIT,
    };

    if parts.next().is_some() {
        return Err(usage(command));
    }

    Ok((address.to_owned(), limit))
}

fn parse_limit(raw: &str) -> Result<i32, String> {
    let value = raw.parse::<i32>().map_err(|_| {
        format!("Invalid limit '{raw}'. Limit must be a number between 1 and {MAX_LIMIT}.")
    })?;

    if !(1..=MAX_LIMIT).contains(&value) {
        return Err(format!(
            "Invalid limit '{raw}'. Limit must be between 1 and {MAX_LIMIT}."
        ));
    }

    Ok(value)
}

fn usage(command: &str) -> String {
    match command {
        "portfolio" => String::from(
            "Usage: /portfolio <wallet-address> [limit]\n\n\
             Example: /portfolio 0x56687bf447db6ffa42ffe2204a05edaa20f55839\n\
             Example: /portfolio 0x56687bf447db6ffa42ffe2204a05edaa20f55839 8",
        ),
        "positions" => String::from(
            "Usage: /positions <wallet-address> [limit]\n\n\
             Example: /positions 0x56687bf447db6ffa42ffe2204a05edaa20f55839\n\
             Example: /positions 0x56687bf447db6ffa42ffe2204a05edaa20f55839 10",
        ),
        "trades" => String::from(
            "Usage: /trades <wallet-address> [limit]\n\n\
             Example: /trades 0x56687bf447db6ffa42ffe2204a05edaa20f55839\n\
             Example: /trades 0x56687bf447db6ffa42ffe2204a05edaa20f55839 10",
        ),
        _ => String::from("Usage error."),
    }
}
