use crate::agent::Agent;
use crate::crowd::CrowdEnvironment;
use crate::orchestrator::SoftwareTeamEnvironment;
use crate::replay::{ReplayStep, RunReplay};
use crate::score::{ScoreCalculator, ScoreCard};
use crate::tactical::TacticalEnvironment;
use crate::traffic::TrafficEnvironment;
use crate::types::ScenarioType;

pub struct ArenaRunner;

impl ArenaRunner {
    pub fn run_scenario(
        scenario: ScenarioType,
        agent: &mut dyn Agent,
        seed: u64,
        max_ticks: u64,
    ) -> (ScoreCard, RunReplay) {
        let mut steps = Vec::new();
        let mut total_reward = 0.0;
        let final_metrics: serde_json::Value;

        match scenario {
            ScenarioType::TrafficGrid => {
                let mut env = TrafficEnvironment::new(seed, max_ticks, 4);
                let mut obs = env.observation();

                while !obs.terminated {
                    let action = agent.act(&obs);
                    let res = env.step(&action);
                    total_reward += res.reward;

                    steps.push(ReplayStep {
                        tick: obs.tick,
                        observation: obs,
                        action,
                        reward: res.reward,
                    });

                    obs = res.observation;
                }
                final_metrics = obs.metrics.clone();
            }
            ScenarioType::TacticalCombat => {
                let mut env = TacticalEnvironment::new(seed, max_ticks);
                let mut obs = env.observation();

                while !obs.terminated {
                    let action = agent.act(&obs);
                    let res = env.step(&action);
                    total_reward += res.reward;

                    steps.push(ReplayStep {
                        tick: obs.tick,
                        observation: obs,
                        action,
                        reward: res.reward,
                    });

                    obs = res.observation;
                }
                final_metrics = obs.metrics.clone();
            }
            ScenarioType::CrowdEvacuation => {
                let mut env = CrowdEnvironment::new(seed, max_ticks, 100);
                let mut obs = env.observation();

                while !obs.terminated {
                    let action = agent.act(&obs);
                    let res = env.step(&action);
                    total_reward += res.reward;

                    steps.push(ReplayStep {
                        tick: obs.tick,
                        observation: obs,
                        action,
                        reward: res.reward,
                    });

                    obs = res.observation;
                }
                final_metrics = obs.metrics.clone();
            }
            ScenarioType::SoftwareTeam => {
                let mut env = SoftwareTeamEnvironment::new(seed, max_ticks, 6);
                let mut obs = env.observation();

                while !obs.terminated {
                    let action = agent.act(&obs);
                    let res = env.step(&action);
                    total_reward += res.reward;

                    steps.push(ReplayStep {
                        tick: obs.tick,
                        observation: obs,
                        action,
                        reward: res.reward,
                    });

                    obs = res.observation;
                }
                final_metrics = obs.metrics.clone();
            }
        }

        let score_card = ScoreCalculator::calculate(scenario, agent.name(), total_reward, &final_metrics);

        let replay = RunReplay {
            run_id: format!("{}-{}-{}", scenario, agent.name(), seed),
            seed,
            scenario,
            agent_name: agent.name().to_string(),
            total_reward,
            steps_count: steps.len(),
            steps,
        };

        (score_card, replay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::ZevDecisionAgent;

    #[test]
    fn test_runner() {
        let mut agent = ZevDecisionAgent::new();
        let (score, replay) = ArenaRunner::run_scenario(ScenarioType::TrafficGrid, &mut agent, 42, 10);

        assert!(score.arena_index > 0);
        assert_eq!(replay.seed, 42);
        assert!(!replay.steps.is_empty());
    }
}
