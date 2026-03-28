use thiserror::Error;

#[derive(Debug, Error)]
pub enum AlertError {
    #[error("invalid command: {0}")]
    InvalidCommand(String),
    #[error("invalid rule: {0}")]
    InvalidRule(String),
    #[error("market slug not found: {0}")]
    UnknownMarket(String),
    #[error("alert not found: {0}")]
    AlertNotFound(String),
    #[error("unauthorized alert operation")]
    Unauthorized,
    #[error("max alerts reached for this user ({0})")]
    MaxAlertsReached(usize),
    #[error("rate limit exceeded, please try again shortly")]
    RateLimited,
    #[error("data source error: {0}")]
    DataSource(String),
    #[error("notification error: {0}")]
    Notification(String),
    #[error("storage error: {0}")]
    Storage(String),
}

impl AlertError {
    pub fn user_message(&self) -> String {
        match self {
            AlertError::InvalidCommand(msg)
            | AlertError::InvalidRule(msg)
            | AlertError::UnknownMarket(msg)
            | AlertError::DataSource(msg) => msg.clone(),
            AlertError::AlertNotFound(id) => {
                format!("Alert \"{id}\" was not found. Use `alert list` to see active alerts.")
            }
            AlertError::Unauthorized => "You can only modify alerts that you created.".to_owned(),
            AlertError::MaxAlertsReached(limit) => {
                format!(
                    "You reached the maximum of {limit} alerts. Remove one before adding another."
                )
            }
            AlertError::RateLimited => {
                "Notification throughput limit reached. Retrying shortly.".to_owned()
            }
            AlertError::Notification(msg) | AlertError::Storage(msg) => {
                format!("Internal alert error: {msg}")
            }
        }
    }
}
