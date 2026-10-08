//! agentarena-core: Simulation environments, observation spaces, action protocols, replay logging, and scoring engine for AgentArena.

pub mod agent;
pub mod crowd;
pub mod orchestrator;
pub mod replay;
pub mod runner;
pub mod score;
pub mod tactical;
pub mod traffic;
pub mod types;

pub use agent::{Agent, RandomAgent, ZevDecisionAgent};
pub use crowd::{CrowdEnvironment, CrowdMetrics};
pub use orchestrator::{SoftwareTeamEnvironment, TaskItem, TeamMetrics};
pub use replay::{ReplayStep, RunReplay};
pub use runner::ArenaRunner;
pub use score::{ScoreCalculator, ScoreCard};
pub use tactical::{CombatUnit, TacticalEnvironment, TacticalMetrics};
pub use traffic::{SignalPhase, TrafficEnvironment, TrafficIntersection, TrafficMetrics};
pub use types::{Action, Observation, ScenarioType, StepResult};
