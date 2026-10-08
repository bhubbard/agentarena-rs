use crate::types::{Action, Observation, ScenarioType, StepResult};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrowdMetrics {
    pub total_pedestrians: usize,
    pub evacuated_count: usize,
    pub remaining_count: usize,
    pub evacuation_rate_percent: f64,
    pub crush_incidents: usize,
    pub main_corridor_density: f64,
}

pub struct CrowdEnvironment {
    pub seed: u64,
    pub tick: u64,
    pub max_ticks: u64,
    pub total_pedestrians: usize,
    pub evacuated: usize,
    pub crush_incidents: usize,
    pub main_exit_open: bool,
    pub aux_exit_open: bool,
    pub corridor_metered: bool,
    rng: StdRng,
}

impl CrowdEnvironment {
    pub fn new(seed: u64, max_ticks: u64, total_pedestrians: usize) -> Self {
        let rng = StdRng::seed_from_u64(seed);
        Self {
            seed,
            tick: 0,
            max_ticks,
            total_pedestrians,
            evacuated: 0,
            crush_incidents: 0,
            main_exit_open: true,
            aux_exit_open: false,
            corridor_metered: false,
            rng,
        }
    }

    pub fn observation(&self) -> Observation {
        let remaining = self.total_pedestrians.saturating_sub(self.evacuated);
        let rate = if self.total_pedestrians > 0 {
            (self.evacuated as f64 / self.total_pedestrians as f64) * 100.0
        } else {
            100.0
        };

        let density = if remaining > 0 {
            (remaining as f64 / 100.0).min(5.0)
        } else {
            0.0
        };

        let metrics = serde_json::to_value(CrowdMetrics {
            total_pedestrians: self.total_pedestrians,
            evacuated_count: self.evacuated,
            remaining_count: remaining,
            evacuation_rate_percent: rate,
            crush_incidents: self.crush_incidents,
            main_corridor_density: density,
        })
        .unwrap_or_default();

        let available_actions = vec![
            "open_auxiliary_exit".to_string(),
            "meter_main_corridor".to_string(),
            "deploy_staggered_barriers".to_string(),
            "broadcast_alternate_route".to_string(),
        ];

        let state_summary = format!(
            "Tick {}: Evacuated {}/{} ({:.1}%), Remaining: {}, Density: {:.2}, Crushes: {}",
            self.tick, self.evacuated, self.total_pedestrians, rate, remaining, density, self.crush_incidents
        );

        Observation {
            tick: self.tick,
            scenario: ScenarioType::CrowdEvacuation,
            state_summary,
            metrics,
            available_actions,
            terminated: self.tick >= self.max_ticks || self.evacuated >= self.total_pedestrians,
        }
    }

    pub fn step(&mut self, action: &Action) -> StepResult {
        if self.tick >= self.max_ticks || self.evacuated >= self.total_pedestrians {
            return StepResult {
                observation: self.observation(),
                reward: 0.0,
                terminated: true,
                info: serde_json::json!({ "reason": "Evacuation complete or timed out" }),
            };
        }

        self.tick += 1;

        match action.name.as_str() {
            "open_auxiliary_exit" => {
                self.aux_exit_open = true;
            }
            "meter_main_corridor" => {
                self.corridor_metered = true;
            }
            "deploy_staggered_barriers" => {
                self.corridor_metered = true;
                self.aux_exit_open = true;
            }
            _ => {}
        }

        let remaining = self.total_pedestrians.saturating_sub(self.evacuated);
        let mut exit_capacity: usize = 0;

        if self.main_exit_open {
            exit_capacity += if self.corridor_metered { 8 } else { 12 };
        }
        if self.aux_exit_open {
            exit_capacity += 10;
        }

        // Density & crush physics
        let mut step_crushes = 0;
        if !self.corridor_metered && remaining > 50 && self.rng.gen_bool(0.35) {
            step_crushes += self.rng.gen_range(1..3);
            exit_capacity = exit_capacity.saturating_sub(4); // Bottleneck slowdown
        }

        self.crush_incidents += step_crushes;
        let step_evacuated = exit_capacity.min(remaining);
        self.evacuated += step_evacuated;

        let reward = (step_evacuated as f64 * 3.0) - (step_crushes as f64 * 50.0)
            - (remaining as f64 * 0.02);

        let terminated = self.tick >= self.max_ticks || self.evacuated >= self.total_pedestrians;

        StepResult {
            observation: self.observation(),
            reward,
            terminated,
            info: serde_json::json!({
                "step_evacuated": step_evacuated,
                "step_crushes": step_crushes,
                "aux_exit_open": self.aux_exit_open,
                "metered": self.corridor_metered,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crowd_evacuation_determinism() {
        let mut env1 = CrowdEnvironment::new(101, 15, 100);
        let mut env2 = CrowdEnvironment::new(101, 15, 100);

        let action = Action {
            name: "open_auxiliary_exit".to_string(),
            parameters: serde_json::json!({}),
        };

        let res1 = env1.step(&action);
        let res2 = env2.step(&action);

        assert_eq!(res1.reward, res2.reward);
        assert_eq!(env1.evacuated, env2.evacuated);
    }
}
