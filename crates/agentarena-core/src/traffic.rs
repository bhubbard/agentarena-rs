use crate::types::{Action, Observation, ScenarioType, StepResult};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalPhase {
    NorthSouthGreen,
    EastWestGreen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficIntersection {
    pub id: usize,
    pub phase: SignalPhase,
    pub queue_ns: usize,
    pub queue_ew: usize,
    pub time_in_current_phase: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficMetrics {
    pub total_cleared: usize,
    pub total_delay: u64,
    pub max_queue: usize,
    pub gridlock: bool,
    pub average_waiting_time: f64,
}

pub struct TrafficEnvironment {
    pub seed: u64,
    pub tick: u64,
    pub max_ticks: u64,
    pub intersections: Vec<TrafficIntersection>,
    pub total_cleared: usize,
    pub total_delay: u64,
    pub max_queue_observed: usize,
    pub gridlock: bool,
    rng: StdRng,
}

impl TrafficEnvironment {
    pub fn new(seed: u64, max_ticks: u64, num_intersections: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut intersections = Vec::with_capacity(num_intersections);

        for id in 0..num_intersections {
            intersections.push(TrafficIntersection {
                id,
                phase: if rng.gen_bool(0.5) {
                    SignalPhase::NorthSouthGreen
                } else {
                    SignalPhase::EastWestGreen
                },
                queue_ns: rng.gen_range(2..6),
                queue_ew: rng.gen_range(2..6),
                time_in_current_phase: 0,
            });
        }

        Self {
            seed,
            tick: 0,
            max_ticks,
            intersections,
            total_cleared: 0,
            total_delay: 0,
            max_queue_observed: 0,
            gridlock: false,
            rng,
        }
    }

    pub fn observation(&self) -> Observation {
        let total_ns: usize = self.intersections.iter().map(|i| i.queue_ns).sum();
        let total_ew: usize = self.intersections.iter().map(|i| i.queue_ew).sum();
        let total_waiting = total_ns + total_ew;

        let metrics = serde_json::to_value(TrafficMetrics {
            total_cleared: self.total_cleared,
            total_delay: self.total_delay,
            max_queue: self.max_queue_observed,
            gridlock: self.gridlock,
            average_waiting_time: if self.total_cleared > 0 {
                self.total_delay as f64 / self.total_cleared as f64
            } else {
                0.0
            },
        })
        .unwrap_or_default();

        let available_actions = vec![
            "switch_phase_all".to_string(),
            "hold_phase".to_string(),
            "switch_heaviest".to_string(),
        ];

        let state_summary = format!(
            "Tick {}: {} intersections, {} total waiting vehicles (NS: {}, EW: {}), {} cleared.",
            self.tick,
            self.intersections.len(),
            total_waiting,
            total_ns,
            total_ew,
            self.total_cleared
        );

        Observation {
            tick: self.tick,
            scenario: ScenarioType::TrafficGrid,
            state_summary,
            metrics,
            available_actions,
            terminated: self.tick >= self.max_ticks || self.gridlock,
        }
    }

    pub fn step(&mut self, action: &Action) -> StepResult {
        if self.tick >= self.max_ticks || self.gridlock {
            return StepResult {
                observation: self.observation(),
                reward: 0.0,
                terminated: true,
                info: serde_json::json!({ "reason": "Already terminated" }),
            };
        }

        self.tick += 1;

        // Apply action
        match action.name.as_str() {
            "switch_phase_all" => {
                for inter in &mut self.intersections {
                    inter.phase = match inter.phase {
                        SignalPhase::NorthSouthGreen => SignalPhase::EastWestGreen,
                        SignalPhase::EastWestGreen => SignalPhase::NorthSouthGreen,
                    };
                    inter.time_in_current_phase = 0;
                }
            }
            "switch_heaviest" => {
                for inter in &mut self.intersections {
                    if inter.phase == SignalPhase::NorthSouthGreen && inter.queue_ew > inter.queue_ns + 3 {
                        inter.phase = SignalPhase::EastWestGreen;
                        inter.time_in_current_phase = 0;
                    } else if inter.phase == SignalPhase::EastWestGreen && inter.queue_ns > inter.queue_ew + 3 {
                        inter.phase = SignalPhase::NorthSouthGreen;
                        inter.time_in_current_phase = 0;
                    } else {
                        inter.time_in_current_phase += 1;
                    }
                }
            }
            _ => {
                // hold_phase or default
                for inter in &mut self.intersections {
                    inter.time_in_current_phase += 1;
                }
            }
        }

        let mut step_cleared = 0;
        let mut step_delay = 0;

        // Simulation physics: process green light throughput & new vehicle arrivals
        for inter in &mut self.intersections {
            // New arrivals
            if self.rng.gen_bool(0.4) {
                inter.queue_ns += self.rng.gen_range(1..3);
            }
            if self.rng.gen_bool(0.4) {
                inter.queue_ew += self.rng.gen_range(1..3);
            }

            // Green phase discharge (throughput)
            match inter.phase {
                SignalPhase::NorthSouthGreen => {
                    let flow = 2.min(inter.queue_ns);
                    inter.queue_ns -= flow;
                    step_cleared += flow;
                }
                SignalPhase::EastWestGreen => {
                    let flow = 2.min(inter.queue_ew);
                    inter.queue_ew -= flow;
                    step_cleared += flow;
                }
            }

            step_delay += inter.queue_ns + inter.queue_ew;

            let max_q = inter.queue_ns.max(inter.queue_ew);
            if max_q > self.max_queue_observed {
                self.max_queue_observed = max_q;
            }

            // Gridlock condition: queue exceeds buffer capacity (e.g. 25 cars)
            if max_q >= 25 {
                self.gridlock = true;
            }
        }

        self.total_cleared += step_cleared;
        self.total_delay += step_delay as u64;

        let reward = (step_cleared as f64 * 2.0) - (step_delay as f64 * 0.05)
            - if self.gridlock { 100.0 } else { 0.0 };

        let terminated = self.tick >= self.max_ticks || self.gridlock;

        StepResult {
            observation: self.observation(),
            reward,
            terminated,
            info: serde_json::json!({
                "step_cleared": step_cleared,
                "step_delay": step_delay,
                "gridlock": self.gridlock,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_env_determinism() {
        let mut env1 = TrafficEnvironment::new(12345, 20, 4);
        let mut env2 = TrafficEnvironment::new(12345, 20, 4);

        let action = Action {
            name: "hold_phase".to_string(),
            parameters: serde_json::json!({}),
        };

        let res1 = env1.step(&action);
        let res2 = env2.step(&action);

        assert_eq!(res1.reward, res2.reward);
        assert_eq!(env1.total_cleared, env2.total_cleared);
    }
}
