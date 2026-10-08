use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioType {
    TrafficGrid,
    TacticalCombat,
    CrowdEvacuation,
    SoftwareTeam,
}

impl fmt::Display for ScenarioType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScenarioType::TrafficGrid => write!(f, "Traffic Grid (SUMO/UXSim)"),
            ScenarioType::TacticalCombat => write!(f, "Tactical Combat (TacticalAI)"),
            ScenarioType::CrowdEvacuation => write!(f, "Crowd Evacuation (DetourCrowd)"),
            ScenarioType::SoftwareTeam => write!(f, "Software Team (ThreeManTeam)"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub tick: u64,
    pub scenario: ScenarioType,
    pub state_summary: String,
    pub metrics: serde_json::Value,
    pub available_actions: Vec<String>,
    pub terminated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub observation: Observation,
    pub reward: f64,
    pub terminated: bool,
    pub info: serde_json::Value,
}
