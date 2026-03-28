use std::str::FromStr;

use crate::alerts::error::AlertError;
use crate::alerts::model::AlertRule;

pub fn parse_add_args(args: &str) -> Result<(String, AlertRule), AlertError> {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(AlertError::InvalidCommand(
            "Usage: alert add <slug> <above|below|move> <value>".to_owned(),
        ));
    }

    let slug = parts[0].trim().to_owned();
    if slug.is_empty() {
        return Err(AlertError::InvalidCommand(
            "Market slug cannot be empty.".to_owned(),
        ));
    }

    let rule = parts[1].to_ascii_lowercase();
    let value_raw = parts[2];

    match rule.as_str() {
        "above" | "below" => {
            let mut value = f64::from_str(value_raw).map_err(|_| {
                AlertError::InvalidRule("Threshold must be a number (e.g. 0.52 or 52).".to_owned())
            })?;
            if value > 1.0 {
                value /= 100.0;
            }
            if !(0.0..=1.0).contains(&value) {
                return Err(AlertError::InvalidRule(
                    "Threshold must be between 0 and 1 (or 0 to 100 cents).".to_owned(),
                ));
            }
            if rule == "above" {
                Ok((slug, AlertRule::Above { threshold: value }))
            } else {
                Ok((slug, AlertRule::Below { threshold: value }))
            }
        }
        "move" => {
            let value = f64::from_str(value_raw).map_err(|_| {
                AlertError::InvalidRule("Move percent must be numeric (e.g. 5).".to_owned())
            })?;
            if value <= 0.0 || value > 100.0 {
                return Err(AlertError::InvalidRule(
                    "Move percent must be > 0 and <= 100.".to_owned(),
                ));
            }
            Ok((slug, AlertRule::PercentMove { percent: value }))
        }
        _ => Err(AlertError::InvalidRule(
            "Rule must be one of: above, below, move.".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_add_args;
    use crate::alerts::model::AlertRule;

    #[test]
    fn parses_above_cents() {
        let (_, rule) = parse_add_args("btc above 52").expect("should parse");
        assert_eq!(rule, AlertRule::Above { threshold: 0.52 });
    }

    #[test]
    fn parses_below_decimal() {
        let (_, rule) = parse_add_args("btc below 0.41").expect("should parse");
        assert_eq!(rule, AlertRule::Below { threshold: 0.41 });
    }

    #[test]
    fn parses_percent_move() {
        let (_, rule) = parse_add_args("btc move 7.5").expect("should parse");
        assert_eq!(rule, AlertRule::PercentMove { percent: 7.5 });
    }

    #[test]
    fn rejects_invalid_threshold_range() {
        let err = parse_add_args("btc above 500").expect_err("should fail");
        assert!(
            err.user_message()
                .contains("Threshold must be between 0 and 1")
        );
    }

    #[test]
    fn rejects_invalid_move_percent() {
        let err = parse_add_args("btc move 0").expect_err("should fail");
        assert!(err.user_message().contains("Move percent must be > 0"));
    }

    #[test]
    fn rejects_invalid_rule() {
        let err = parse_add_args("btc weird 1").expect_err("should fail");
        assert!(err.to_string().contains("invalid rule"));
    }
}
