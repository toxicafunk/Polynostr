use crate::format;
use crate::polymarket::gamma::GammaClient;

/// Handle the `/search <query>` command.
pub async fn handle(gamma: &GammaClient, args: &str) -> String {
    let query = args.trim();
    if query.is_empty() {
        return String::from(
            "Usage: /search <query>\n\nExample: /search bitcoin",
        );
    }

    match gamma.search(query).await {
        Ok(results) => format::format_search_results(&results, query),
        Err(e) => format!("Error searching for \"{query}\": {e}"),
    }
}
