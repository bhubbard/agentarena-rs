use crate::types::{Action, Observation, ScenarioType, StepResult};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    pub id: usize,
    pub title: String,
    pub complexity: u32,
    pub in_progress: bool,
    pub completed: bool,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMetrics {
    pub completed_tasks: usize,
    pub verified_tasks: usize,
    pub total_tasks: usize,
    pub tokens_spent: usize,
    pub regressions_found: usize,
    pub cost_usd: f64,
}

pub struct SoftwareTeamEnvironment {
    pub seed: u64,
    pub tick: u64,
    pub max_ticks: u64,
    pub tasks: Vec<TaskItem>,
    pub tokens_spent: usize,
    pub regressions: usize,
    rng: StdRng,
}

impl SoftwareTeamEnvironment {
    pub fn new(seed: u64, max_ticks: u64, task_count: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut tasks = Vec::with_capacity(task_count);

        for id in 0..task_count {
            tasks.push(TaskItem {
                id,
                title: format!("Feature/Module #{}", id + 1),
                complexity: rng.gen_range(1..4),
                in_progress: false,
                completed: false,
                verified: false,
            });
        }

        Self {
            seed,
            tick: 0,
            max_ticks,
            tasks,
            tokens_spent: 0,
            regressions: 0,
            rng,
        }
    }

    pub fn observation(&self) -> Observation {
        let completed = self.tasks.iter().filter(|t| t.completed).count();
        let verified = self.tasks.iter().filter(|t| t.verified).count();
        let cost_usd = (self.tokens_spent as f64 / 1_000_000.0) * 3.0; // ~$3 per M tokens

        let metrics = serde_json::to_value(TeamMetrics {
            completed_tasks: completed,
            verified_tasks: verified,
            total_tasks: self.tasks.len(),
            tokens_spent: self.tokens_spent,
            regressions_found: self.regressions,
            cost_usd,
        })
        .unwrap_or_default();

        let available_actions = vec![
            "dispatch_task".to_string(),
            "conduct_code_review".to_string(),
            "run_automated_tests".to_string(),
            "optimize_token_budget".to_string(),
        ];

        let state_summary = format!(
            "Tick {}: Verified: {}/{} tasks, Completed: {}, Tokens: {}, Regressions: {}, Cost: ${:.4}",
            self.tick, verified, self.tasks.len(), completed, self.tokens_spent, self.regressions, cost_usd
        );

        Observation {
            tick: self.tick,
            scenario: ScenarioType::SoftwareTeam,
            state_summary,
            metrics,
            available_actions,
            terminated: self.tick >= self.max_ticks || verified >= self.tasks.len(),
        }
    }

    pub fn step(&mut self, action: &Action) -> StepResult {
        let verified = self.tasks.iter().filter(|t| t.verified).count();
        if self.tick >= self.max_ticks || verified >= self.tasks.len() {
            return StepResult {
                observation: self.observation(),
                reward: 0.0,
                terminated: true,
                info: serde_json::json!({ "reason": "Sprints complete" }),
            };
        }

        self.tick += 1;

        let mut step_tokens = 0;
        let mut newly_verified = 0;

        match action.name.as_str() {
            "dispatch_task" => {
                step_tokens += 8000;
                if let Some(task) = self.tasks.iter_mut().find(|t| !t.completed) {
                    task.in_progress = true;
                    task.completed = true;
                }
            }
            "conduct_code_review" => {
                step_tokens += 4000;
                if let Some(task) = self.tasks.iter_mut().find(|t| t.completed && !t.verified) {
                    let has_bug = self.rng.gen_bool(0.25);
                    if has_bug {
                        task.completed = false; // Send back for revisions
                        self.regressions += 1;
                    } else {
                        task.verified = true;
                        newly_verified += 1;
                    }
                }
            }
            "run_automated_tests" => {
                step_tokens += 2000;
                for task in self.tasks.iter_mut().filter(|t| t.completed && !t.verified) {
                    if self.rng.gen_bool(0.85) {
                        task.verified = true;
                        newly_verified += 1;
                    }
                }
            }
            _ => {
                // optimize_token_budget or hold
                step_tokens += 500;
            }
        }

        self.tokens_spent += step_tokens;

        let reward = (newly_verified as f64 * 40.0)
            - (step_tokens as f64 * 0.001)
            - (self.regressions as f64 * 10.0);

        let terminated = self.tick >= self.max_ticks || self.tasks.iter().all(|t| t.verified);

        StepResult {
            observation: self.observation(),
            reward,
            terminated,
            info: serde_json::json!({
                "step_tokens": step_tokens,
                "newly_verified": newly_verified,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_team_determinism() {
        let mut env1 = SoftwareTeamEnvironment::new(777, 10, 5);
        let mut env2 = SoftwareTeamEnvironment::new(777, 10, 5);

        let action = Action {
            name: "dispatch_task".to_string(),
            parameters: serde_json::json!({}),
        };

        let res1 = env1.step(&action);
        let res2 = env2.step(&action);

        assert_eq!(res1.reward, res2.reward);
        assert_eq!(env1.tokens_spent, env2.tokens_spent);
    }
}
