use crate::types::{Action, Observation, ScenarioType};
use rand::seq::SliceRandom;
use rand::SeedableRng;

pub trait Agent {
    fn name(&self) -> &str;
    fn act(&mut self, obs: &Observation) -> Action;
}

/// Baseline agent that selects randomly among valid actions
pub struct RandomAgent {
    name: String,
    rng: rand::rngs::StdRng,
}

impl RandomAgent {
    pub fn new(seed: u64) -> Self {
        Self {
            name: "RandomBaseline".to_string(),
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }
}

impl Agent for RandomAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn act(&mut self, obs: &Observation) -> Action {
        let action_name = obs
            .available_actions
            .choose(&mut self.rng)
            .cloned()
            .unwrap_or_else(|| "hold".to_string());

        Action {
            name: action_name,
            parameters: serde_json::json!({}),
        }
    }
}

/// Zero-token deterministic decision policy agent inspired by zev-rs
pub struct ZevDecisionAgent {
    name: String,
}

impl ZevDecisionAgent {
    pub fn new() -> Self {
        Self {
            name: "ZevZeroTokenAgent".to_string(),
        }
    }
}

impl Default for ZevDecisionAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for ZevDecisionAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn act(&mut self, obs: &Observation) -> Action {
        let action_name = match obs.scenario {
            ScenarioType::TrafficGrid => {
                // Heuristic: Switch heaviest queue
                "switch_heaviest".to_string()
            }
            ScenarioType::TacticalCombat => {
                // Heuristic: Advance to cover if unpinned, otherwise flank
                if obs.tick % 3 == 0 {
                    "advance_to_cover".to_string()
                } else if obs.tick % 3 == 1 {
                    "suppress_fire".to_string()
                } else {
                    "flank_enemy".to_string()
                }
            }
            ScenarioType::CrowdEvacuation => {
                // Heuristic: Deploy barriers and open auxiliary exits early
                if obs.tick <= 2 {
                    "deploy_staggered_barriers".to_string()
                } else {
                    "open_auxiliary_exit".to_string()
                }
            }
            ScenarioType::SoftwareTeam => {
                // Heuristic: Dispatch then test then review
                if obs.tick % 3 == 0 {
                    "dispatch_task".to_string()
                } else if obs.tick % 3 == 1 {
                    "run_automated_tests".to_string()
                } else {
                    "conduct_code_review".to_string()
                }
            }
        };

        Action {
            name: action_name,
            parameters: serde_json::json!({ "policy": "zev_deterministic_pareto" }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agents() {
        let mut random_agent = RandomAgent::new(42);
        let mut zev_agent = ZevDecisionAgent::new();

        let obs = Observation {
            tick: 1,
            scenario: ScenarioType::TrafficGrid,
            state_summary: "Test".to_string(),
            metrics: serde_json::json!({}),
            available_actions: vec!["switch_heaviest".to_string(), "hold_phase".to_string()],
            terminated: false,
        };

        let act1 = random_agent.act(&obs);
        let act2 = zev_agent.act(&obs);

        assert!(!act1.name.is_empty());
        assert_eq!(act2.name, "switch_heaviest");
    }
}
