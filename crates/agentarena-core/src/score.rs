use crate::types::ScenarioType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreCard {
    pub scenario: ScenarioType,
    pub agent_name: String,
    pub total_reward: f64,
    pub success_rate: f64,    // 0.0 - 100.0
    pub efficiency_score: f64, // 0.0 - 100.0
    pub arena_index: u32,      // 0 - 1000
    pub metrics_summary: serde_json::Value,
}

pub struct ScoreCalculator;

impl ScoreCalculator {
    pub fn calculate(
        scenario: ScenarioType,
        agent_name: &str,
        total_reward: f64,
        metrics: &serde_json::Value,
    ) -> ScoreCard {
        let (success_rate, efficiency) = match scenario {
            ScenarioType::TrafficGrid => {
                let cleared = metrics.get("total_cleared").and_then(|v| v.as_u64()).unwrap_or(0);
                let gridlock = metrics.get("gridlock").and_then(|v| v.as_bool()).unwrap_or(false);
                let success = if gridlock { 20.0 } else { (cleared as f64 * 3.0).min(100.0) };
                let eff = if cleared > 0 { 85.0 } else { 10.0 };
                (success, eff)
            }
            ScenarioType::TacticalCombat => {
                let enemies_killed = metrics.get("enemies_eliminated").and_then(|v| v.as_u64()).unwrap_or(0);
                let friendly_losses = metrics.get("friendly_casualties").and_then(|v| v.as_u64()).unwrap_or(0);
                let success = (enemies_killed as f64 * 25.0).min(100.0);
                let eff = (100.0 - (friendly_losses as f64 * 25.0)).max(0.0);
                (success, eff)
            }
            ScenarioType::CrowdEvacuation => {
                let rate = metrics.get("evacuation_rate_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let crushes = metrics.get("crush_incidents").and_then(|v| v.as_u64()).unwrap_or(0);
                let eff = (100.0 - (crushes as f64 * 15.0)).max(0.0);
                (rate, eff)
            }
            ScenarioType::SoftwareTeam => {
                let verified = metrics.get("verified_tasks").and_then(|v| v.as_u64()).unwrap_or(0);
                let total = metrics.get("total_tasks").and_then(|v| v.as_u64()).unwrap_or(1);
                let regressions = metrics.get("regressions_found").and_then(|v| v.as_u64()).unwrap_or(0);
                let success = (verified as f64 / total as f64) * 100.0;
                let eff = (100.0 - (regressions as f64 * 20.0)).max(0.0);
                (success, eff)
            }
        };

        let arena_index = ((success_rate * 6.0) + (efficiency * 4.0)).clamp(0.0, 1000.0) as u32;

        ScoreCard {
            scenario,
            agent_name: agent_name.to_string(),
            total_reward,
            success_rate,
            efficiency_score: efficiency,
            arena_index,
            metrics_summary: metrics.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_calculation() {
        let metrics = serde_json::json!({
            "total_cleared": 30,
            "gridlock": false,
        });

        let card = ScoreCalculator::calculate(ScenarioType::TrafficGrid, "ZevAgent", 150.0, &metrics);
        assert!(card.arena_index > 500);
        assert_eq!(card.agent_name, "ZevAgent");
    }
}
