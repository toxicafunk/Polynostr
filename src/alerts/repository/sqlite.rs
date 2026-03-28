use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};

use crate::alerts::error::AlertError;
use crate::alerts::model::{
    AlertRule, AlertStatus, AlertSubscription, DeliveryChannel, DeliveryTarget, TriggerState,
};
use crate::alerts::repository::AlertRepository;

#[derive(Clone)]
pub struct SqliteAlertRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteAlertRepository {
    pub fn new(path: &str) -> Result<Self, AlertError> {
        let conn = Connection::open(path).map_err(|e| AlertError::Storage(e.to_string()))?;
        let repo = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        repo.init_schema()?;
        Ok(repo)
    }

    fn init_schema(&self) -> Result<(), AlertError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS alerts (
                id TEXT PRIMARY KEY,
                slug TEXT NOT NULL,
                rule_kind TEXT NOT NULL,
                rule_value REAL NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                pubkey TEXT NOT NULL,
                channel TEXT NOT NULL,
                last_seen_price REAL,
                last_triggered_price REAL,
                last_triggered_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_alerts_pubkey ON alerts(pubkey);
            CREATE INDEX IF NOT EXISTS idx_alerts_status ON alerts(status);
            "#,
        )
        .map_err(|e| AlertError::Storage(e.to_string()))?;
        Ok(())
    }

    fn parse_rule(kind: &str, value: f64) -> Result<AlertRule, AlertError> {
        match kind {
            "above" => Ok(AlertRule::Above { threshold: value }),
            "below" => Ok(AlertRule::Below { threshold: value }),
            "move" => Ok(AlertRule::PercentMove { percent: value }),
            _ => Err(AlertError::Storage(format!("unknown rule kind: {kind}"))),
        }
    }

    fn encode_rule(rule: &AlertRule) -> (&'static str, f64) {
        match rule {
            AlertRule::Above { threshold } => ("above", *threshold),
            AlertRule::Below { threshold } => ("below", *threshold),
            AlertRule::PercentMove { percent } => ("move", *percent),
        }
    }

    fn row_to_alert(row: &rusqlite::Row<'_>) -> rusqlite::Result<AlertSubscription> {
        let rule_kind: String = row.get(2)?;
        let rule_value: f64 = row.get(3)?;
        let status_raw: String = row.get(4)?;
        let created_at_raw: String = row.get(5)?;
        let updated_at_raw: String = row.get(6)?;
        let pubkey: String = row.get(7)?;
        let channel_raw: String = row.get(8)?;
        let last_seen_price: Option<f64> = row.get(9)?;
        let last_triggered_price: Option<f64> = row.get(10)?;
        let last_triggered_at_raw: Option<String> = row.get(11)?;

        let rule = Self::parse_rule(&rule_kind, rule_value)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let status = if status_raw == "paused" {
            AlertStatus::Paused
        } else {
            AlertStatus::Active
        };

        let channel = if channel_raw == "nip04" {
            DeliveryChannel::Nip04
        } else {
            DeliveryChannel::Nip17
        };

        let created_at = DateTime::parse_from_rfc3339(&created_at_raw)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_raw)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
            .with_timezone(&Utc);

        let last_triggered_at = match last_triggered_at_raw {
            Some(ts) => Some(
                DateTime::parse_from_rfc3339(&ts)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                    .with_timezone(&Utc),
            ),
            None => None,
        };

        Ok(AlertSubscription {
            id: row.get(0)?,
            slug: row.get(1)?,
            rule,
            status,
            created_at,
            updated_at,
            delivery: DeliveryTarget { pubkey, channel },
            trigger_state: TriggerState {
                last_seen_price,
                last_triggered_price,
                last_triggered_at,
            },
        })
    }
}

#[async_trait]
impl AlertRepository for SqliteAlertRepository {
    async fn create(&self, alert: AlertSubscription) -> Result<AlertSubscription, AlertError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        let (rule_kind, rule_value) = Self::encode_rule(&alert.rule);
        conn.execute(
            "INSERT INTO alerts (id, slug, rule_kind, rule_value, status, created_at, updated_at, pubkey, channel, last_seen_price, last_triggered_price, last_triggered_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                alert.id,
                alert.slug,
                rule_kind,
                rule_value,
                if alert.status == AlertStatus::Paused { "paused" } else { "active" },
                alert.created_at.to_rfc3339(),
                alert.updated_at.to_rfc3339(),
                alert.delivery.pubkey,
                if alert.delivery.channel == DeliveryChannel::Nip04 { "nip04" } else { "nip17" },
                alert.trigger_state.last_seen_price,
                alert.trigger_state.last_triggered_price,
                alert.trigger_state.last_triggered_at.map(|ts| ts.to_rfc3339()),
            ],
        )
        .map_err(|e| AlertError::Storage(e.to_string()))?;

        Ok(alert)
    }

    async fn list_by_pubkey(&self, pubkey: &str) -> Result<Vec<AlertSubscription>, AlertError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, slug, rule_kind, rule_value, status, created_at, updated_at, pubkey, channel, last_seen_price, last_triggered_price, last_triggered_at FROM alerts WHERE pubkey = ?1 ORDER BY created_at DESC")
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map(params![pubkey], Self::row_to_alert)
            .map_err(|e| AlertError::Storage(e.to_string()))?;

        let mut alerts = Vec::new();
        for row in rows {
            alerts.push(row.map_err(|e| AlertError::Storage(e.to_string()))?);
        }
        Ok(alerts)
    }

    async fn list_active(&self) -> Result<Vec<AlertSubscription>, AlertError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, slug, rule_kind, rule_value, status, created_at, updated_at, pubkey, channel, last_seen_price, last_triggered_price, last_triggered_at FROM alerts WHERE status = 'active'")
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([], Self::row_to_alert)
            .map_err(|e| AlertError::Storage(e.to_string()))?;

        let mut alerts = Vec::new();
        for row in rows {
            alerts.push(row.map_err(|e| AlertError::Storage(e.to_string()))?);
        }
        Ok(alerts)
    }

    async fn get(&self, alert_id: &str) -> Result<Option<AlertSubscription>, AlertError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, slug, rule_kind, rule_value, status, created_at, updated_at, pubkey, channel, last_seen_price, last_triggered_price, last_triggered_at FROM alerts WHERE id = ?1")
            .map_err(|e| AlertError::Storage(e.to_string()))?;

        let alert = stmt
            .query_row(params![alert_id], Self::row_to_alert)
            .optional()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        Ok(alert)
    }

    async fn delete_by_owner(&self, alert_id: &str, owner_pubkey: &str) -> Result<(), AlertError> {
        let existing = self.get(alert_id).await?;
        let Some(alert) = existing else {
            return Err(AlertError::AlertNotFound(alert_id.to_owned()));
        };
        if alert.delivery.pubkey != owner_pubkey {
            return Err(AlertError::Unauthorized);
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        conn.execute("DELETE FROM alerts WHERE id = ?1", params![alert_id])
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn set_status_by_owner(
        &self,
        alert_id: &str,
        owner_pubkey: &str,
        status: AlertStatus,
    ) -> Result<AlertSubscription, AlertError> {
        let Some(mut alert) = self.get(alert_id).await? else {
            return Err(AlertError::AlertNotFound(alert_id.to_owned()));
        };
        if alert.delivery.pubkey != owner_pubkey {
            return Err(AlertError::Unauthorized);
        }

        alert.status = status;
        alert.updated_at = Utc::now();

        let conn = self
            .conn
            .lock()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        conn.execute(
            "UPDATE alerts SET status = ?2, updated_at = ?3 WHERE id = ?1",
            params![
                alert_id,
                if alert.status == AlertStatus::Paused {
                    "paused"
                } else {
                    "active"
                },
                alert.updated_at.to_rfc3339()
            ],
        )
        .map_err(|e| AlertError::Storage(e.to_string()))?;

        Ok(alert)
    }

    async fn update_trigger_state(
        &self,
        alert_id: &str,
        last_seen_price: Option<f64>,
        last_triggered_price: Option<f64>,
        last_triggered_at: Option<DateTime<Utc>>,
    ) -> Result<(), AlertError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        conn.execute(
            "UPDATE alerts
             SET last_seen_price = ?2,
                 last_triggered_price = COALESCE(?3, last_triggered_price),
                 last_triggered_at = COALESCE(?4, last_triggered_at),
                 updated_at = ?5
             WHERE id = ?1",
            params![
                alert_id,
                last_seen_price,
                last_triggered_price,
                last_triggered_at.map(|ts| ts.to_rfc3339()),
                Utc::now().to_rfc3339(),
            ],
        )
        .map_err(|e| AlertError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn count_by_pubkey(&self, pubkey: &str) -> Result<usize, AlertError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM alerts WHERE pubkey = ?1 AND status = 'active'",
                params![pubkey],
                |r| r.get(0),
            )
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        Ok(count as usize)
    }

    async fn update_delivery_by_owner(
        &self,
        alert_id: &str,
        owner_pubkey: &str,
        delivery: DeliveryTarget,
    ) -> Result<AlertSubscription, AlertError> {
        let Some(mut alert) = self.get(alert_id).await? else {
            return Err(AlertError::AlertNotFound(alert_id.to_owned()));
        };
        if alert.delivery.pubkey != owner_pubkey {
            return Err(AlertError::Unauthorized);
        }
        alert.delivery = delivery;
        alert.updated_at = Utc::now();

        let conn = self
            .conn
            .lock()
            .map_err(|e| AlertError::Storage(e.to_string()))?;
        conn.execute(
            "UPDATE alerts SET pubkey = ?2, channel = ?3, updated_at = ?4 WHERE id = ?1",
            params![
                alert_id,
                alert.delivery.pubkey,
                if alert.delivery.channel == DeliveryChannel::Nip04 {
                    "nip04"
                } else {
                    "nip17"
                },
                alert.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| AlertError::Storage(e.to_string()))?;

        Ok(alert)
    }
}

#[cfg(test)]
mod tests {
    use crate::alerts::model::{AlertRule, AlertStatus};

    use super::*;

    #[tokio::test]
    async fn sqlite_repo_persists_and_lists() {
        let repo = SqliteAlertRepository::new(":memory:").expect("repo");
        let alert = AlertSubscription::new(
            "btc-100k".to_owned(),
            AlertRule::Above { threshold: 0.5 },
            DeliveryTarget {
                pubkey: "pubkey1".to_owned(),
                channel: DeliveryChannel::Nip17,
            },
        );

        repo.create(alert.clone()).await.expect("create");
        let list = repo.list_by_pubkey("pubkey1").await.expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, alert.id);

        repo.set_status_by_owner(&alert.id, "pubkey1", AlertStatus::Paused)
            .await
            .expect("pause");
        let got = repo.get(&alert.id).await.expect("get").expect("some");
        assert_eq!(got.status, AlertStatus::Paused);
    }

    #[tokio::test]
    async fn sqlite_repo_survives_restart() {
        let db_path = format!(
            "{}/polynostr-alerts-restart-{}.sqlite3",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );

        let alert = AlertSubscription::new(
            "will-bitcoin-hit-150k".to_owned(),
            AlertRule::Below { threshold: 0.30 },
            DeliveryTarget {
                pubkey: "pubkey-restart".to_owned(),
                channel: DeliveryChannel::Nip04,
            },
        );
        let alert_id = alert.id.clone();

        {
            let repo = SqliteAlertRepository::new(&db_path).expect("repo open");
            repo.create(alert).await.expect("create");
            repo.update_trigger_state(&alert_id, Some(0.31), Some(0.30), Some(Utc::now()))
                .await
                .expect("update state");
        }

        {
            let repo = SqliteAlertRepository::new(&db_path).expect("repo reopen");
            let restored = repo.get(&alert_id).await.expect("get").expect("exists");
            assert_eq!(restored.slug, "will-bitcoin-hit-150k");
            assert_eq!(restored.delivery.channel, DeliveryChannel::Nip04);
            assert_eq!(restored.trigger_state.last_seen_price, Some(0.31));
            assert_eq!(restored.trigger_state.last_triggered_price, Some(0.30));
            assert!(restored.trigger_state.last_triggered_at.is_some());
        }

        let _ = std::fs::remove_file(&db_path);
    }
}
