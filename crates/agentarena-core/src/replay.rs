use crate::types::{Action, Observation, ScenarioType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayStep {
    pub tick: u64,
    pub observation: Observation,
    pub action: Action,
    pub reward: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReplay {
    pub run_id: String,
    pub seed: u64,
    pub scenario: ScenarioType,
    pub agent_name: String,
    pub total_reward: f64,
    pub steps_count: usize,
    pub steps: Vec<ReplayStep>,
}

impl RunReplay {
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(self)
            .context("Failed serializing replay run")?;
        std::fs::write(path_ref, json)?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = std::fs::read_to_string(path_ref)
            .with_context(|| format!("Failed to read replay file at {:?}", path_ref))?;
        let replay: Self = serde_json::from_str(&content)?;
        Ok(replay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_replay_serde() {
        let replay = RunReplay {
            run_id: "run-001".to_string(),
            seed: 42,
            scenario: ScenarioType::TrafficGrid,
            agent_name: "TestAgent".to_string(),
            total_reward: 125.5,
            steps_count: 1,
            steps: vec![ReplayStep {
                tick: 1,
                observation: Observation {
                    tick: 1,
                    scenario: ScenarioType::TrafficGrid,
                    state_summary: "Summary".to_string(),
                    metrics: serde_json::json!({}),
                    available_actions: vec![],
                    terminated: false,
                },
                action: Action {
                    name: "hold".to_string(),
                    parameters: serde_json::json!({}),
                },
                reward: 12.5,
            }],
        };

        let temp_file = NamedTempFile::new().unwrap();
        replay.save_to_file(temp_file.path()).unwrap();
        let loaded = RunReplay::load_from_file(temp_file.path()).unwrap();

        assert_eq!(loaded.run_id, "run-001");
        assert_eq!(loaded.seed, 42);
        assert_eq!(loaded.steps.len(), 1);
    }
}
