use chrono::Utc;

use crate::alerts::model::{AlertEvent, AlertRule, AlertSubscription, MarketTick};

#[derive(Debug, Clone)]
pub struct EvaluatorConfig {
    pub cooldown_seconds: u64,
    pub hysteresis_bps: u32,
}

pub struct AlertEvaluator {
    config: EvaluatorConfig,
}

impl AlertEvaluator {
    pub fn new(config: EvaluatorConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(
        &self,
        alert: &AlertSubscription,
        tick: &MarketTick,
    ) -> (Option<AlertEvent>, Option<f64>) {
        let last_price = alert.trigger_state.last_seen_price;
        let mut should_trigger = false;

        match alert.rule {
            AlertRule::Above { threshold } => {
                if let Some(prev) = last_price {
                    should_trigger = prev < threshold && tick.price >= threshold;
                }
            }
            AlertRule::Below { threshold } => {
                if let Some(prev) = last_price {
                    should_trigger = prev > threshold && tick.price <= threshold;
                }
            }
            AlertRule::PercentMove { percent } => {
                if let Some(prev) = last_price {
                    if prev > 0.0 {
                        let delta = ((tick.price - prev).abs() / prev) * 100.0;
                        should_trigger = delta >= percent;
                    }
                }
            }
        }

        if should_trigger {
            if let Some(last_ts) = alert.trigger_state.last_triggered_at {
                let elapsed = (tick.seen_at - last_ts).num_seconds().max(0) as u64;
                if elapsed < self.config.cooldown_seconds {
                    return (None, Some(tick.price));
                }
            }

            if let Some(last_triggered_price) = alert.trigger_state.last_triggered_price {
                let min_delta = self.config.hysteresis_bps as f64 / 10_000.0;
                if last_triggered_price > 0.0 {
                    let delta = ((tick.price - last_triggered_price).abs()) / last_triggered_price;
                    if delta < min_delta {
                        return (None, Some(tick.price));
                    }
                }
            }

            return (
                Some(AlertEvent {
                    alert_id: alert.id.clone(),
                    slug: alert.slug.clone(),
                    price: tick.price,
                    triggered_at: Utc::now(),
                }),
                Some(tick.price),
            );
        }

        (None, Some(tick.price))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::alerts::model::{
        AlertRule, AlertStatus, AlertSubscription, DeliveryChannel, DeliveryTarget, TriggerState,
    };

    use super::*;

    fn base_alert(rule: AlertRule) -> AlertSubscription {
        AlertSubscription {
            id: "a1".to_owned(),
            slug: "btc-100k".to_owned(),
            rule,
            status: AlertStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            delivery: DeliveryTarget {
                pubkey: "npub-test".to_owned(),
                channel: DeliveryChannel::Nip17,
            },
            trigger_state: TriggerState {
                last_seen_price: Some(0.45),
                last_triggered_price: None,
                last_triggered_at: None,
            },
        }
    }

    #[test]
    fn above_rule_triggers_on_crossing() {
        let eval = AlertEvaluator::new(EvaluatorConfig {
            cooldown_seconds: 60,
            hysteresis_bps: 50,
        });
        let alert = base_alert(AlertRule::Above { threshold: 0.50 });
        let tick = MarketTick {
            slug: "btc-100k".to_owned(),
            price: 0.51,
            seen_at: Utc::now(),
        };
        let (event, _) = eval.evaluate(&alert, &tick);
        assert!(event.is_some());
    }

    #[test]
    fn below_rule_triggers_on_crossing() {
        let eval = AlertEvaluator::new(EvaluatorConfig {
            cooldown_seconds: 60,
            hysteresis_bps: 50,
        });
        let mut alert = base_alert(AlertRule::Below { threshold: 0.40 });
        alert.trigger_state.last_seen_price = Some(0.45);
        let tick = MarketTick {
            slug: "btc-100k".to_owned(),
            price: 0.39,
            seen_at: Utc::now(),
        };
        let (event, _) = eval.evaluate(&alert, &tick);
        assert!(event.is_some());
    }

    #[test]
    fn percent_move_triggers_when_delta_exceeds_threshold() {
        let eval = AlertEvaluator::new(EvaluatorConfig {
            cooldown_seconds: 60,
            hysteresis_bps: 50,
        });
        let mut alert = base_alert(AlertRule::PercentMove { percent: 10.0 });
        alert.trigger_state.last_seen_price = Some(0.40);
        let tick = MarketTick {
            slug: "btc-100k".to_owned(),
            price: 0.45,
            seen_at: Utc::now(),
        };
        let (event, _) = eval.evaluate(&alert, &tick);
        assert!(event.is_some());
    }

    #[test]
    fn hysteresis_prevents_small_repeat_move() {
        let eval = AlertEvaluator::new(EvaluatorConfig {
            cooldown_seconds: 1,
            hysteresis_bps: 200,
        });
        let mut alert = base_alert(AlertRule::Above { threshold: 0.50 });
        alert.trigger_state.last_seen_price = Some(0.49);
        alert.trigger_state.last_triggered_price = Some(0.5000);
        alert.trigger_state.last_triggered_at = Some(Utc::now() - Duration::seconds(120));

        let tick = MarketTick {
            slug: "btc-100k".to_owned(),
            price: 0.505,
            seen_at: Utc::now(),
        };
        let (event, _) = eval.evaluate(&alert, &tick);
        assert!(event.is_none());
    }

    #[test]
    fn cooldown_prevents_duplicate() {
        let eval = AlertEvaluator::new(EvaluatorConfig {
            cooldown_seconds: 120,
            hysteresis_bps: 50,
        });
        let mut alert = base_alert(AlertRule::Above { threshold: 0.50 });
        alert.trigger_state.last_triggered_at = Some(Utc::now() - Duration::seconds(30));
        alert.trigger_state.last_triggered_price = Some(0.50);

        let tick = MarketTick {
            slug: "btc-100k".to_owned(),
            price: 0.52,
            seen_at: Utc::now(),
        };
        let (event, _) = eval.evaluate(&alert, &tick);
        assert!(event.is_none());
    }
}
