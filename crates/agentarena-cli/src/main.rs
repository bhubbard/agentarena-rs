use agentarena_core::{
    Agent, ArenaRunner, RandomAgent, RunReplay, ScenarioType, ZevDecisionAgent,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "agentarena",
    version,
    about = "AgentArena: Dynamic simulation test bench and deterministic benchmark arena for AI agents",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a scenario evaluation with an AI agent
    Run {
        /// Scenario: traffic, tactical, crowd, team
        #[arg(short = 's', long = "scenario", default_value = "traffic")]
        scenario: String,

        /// Agent policy: zev, random
        #[arg(short = 'a', long = "agent", default_value = "zev")]
        agent: String,

        /// Deterministic random seed
        #[arg(long = "seed", default_value = "42")]
        seed: u64,

        /// Maximum simulation ticks
        #[arg(long = "ticks", default_value = "30")]
        ticks: u64,

        /// Optional replay export path (.json)
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,
    },

    /// Run full multi-scenario benchmark leaderboard across all environments
    Benchmark {
        /// Deterministic random seed
        #[arg(long = "seed", default_value = "42")]
        seed: u64,

        /// Simulation ticks per scenario
        #[arg(long = "ticks", default_value = "25")]
        ticks: u64,
    },

    /// Inspect and verify a recorded execution replay
    Replay {
        /// Path to replay JSON file
        #[arg(short = 'f', long = "file")]
        file: PathBuf,

        /// Verbose step-by-step dump
        #[arg(short = 'v', long = "verbose")]
        verbose: bool,
    },

    /// List all available simulation environments and evaluation criteria
    Scenarios,
}

fn parse_scenario(s: &str) -> Result<ScenarioType> {
    match s.to_lowercase().as_str() {
        "traffic" | "traffic_grid" | "sumo" | "uxsim" => Ok(ScenarioType::TrafficGrid),
        "tactical" | "tactical_combat" | "combat" => Ok(ScenarioType::TacticalCombat),
        "crowd" | "crowd_evacuation" | "evac" => Ok(ScenarioType::CrowdEvacuation),
        "team" | "software_team" | "orchestrator" => Ok(ScenarioType::SoftwareTeam),
        other => anyhow::bail!("Unknown scenario: '{}'. Available: traffic, tactical, crowd, team", other),
    }
}

fn create_agent(name: &str, seed: u64) -> Result<Box<dyn Agent>> {
    match name.to_lowercase().as_str() {
        "zev" | "zero_token" | "pareto" => Ok(Box::new(ZevDecisionAgent::new())),
        "random" => Ok(Box::new(RandomAgent::new(seed))),
        other => anyhow::bail!("Unknown agent: '{}'. Available: zev, random", other),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            scenario,
            agent,
            seed,
            ticks,
            output,
        } => {
            let sc_type = parse_scenario(&scenario)?;
            let mut agent_box = create_agent(&agent, seed)?;

            println!(
                "{} AgentArena evaluation: [{}] on [{}] (Seed: {})...",
                "Starting".cyan().bold(),
                agent_box.name().yellow().bold(),
                sc_type.to_string().cyan().bold(),
                seed
            );

            let (score, replay) = ArenaRunner::run_scenario(sc_type, agent_box.as_mut(), seed, ticks);

            println!("\n{}", "Scenario Evaluation Complete:".green().bold());
            println!("  Scenario         : {}", score.scenario);
            println!("  Agent Policy     : {}", score.agent_name.yellow());
            println!("  Total Reward     : {:.2}", score.total_reward);
            println!("  Success Rate     : {:.1}%", score.success_rate);
            println!("  Efficiency Score : {:.1}%", score.efficiency_score);
            println!("  Arena Index      : {}", format!("{}/1000", score.arena_index).green().bold());

            if let Some(out_p) = output {
                replay.save_to_file(&out_p)?;
                println!("  Replay Saved To  : {}", out_p.display().to_string().green());
            }
        }

        Commands::Benchmark { seed, ticks } => {
            println!(
                "{} AgentArena Full Benchmark Suite (Seed: {}, Ticks: {})...",
                "Running".cyan().bold(),
                seed,
                ticks
            );

            let scenarios = [
                ScenarioType::TrafficGrid,
                ScenarioType::TacticalCombat,
                ScenarioType::CrowdEvacuation,
                ScenarioType::SoftwareTeam,
            ];

            println!("\n{:<28} {:<18} {:<12} {:<10} {:<12}", "Scenario", "Agent", "Reward", "Success", "Arena Index");
            println!("{}", "=".repeat(80));

            let mut zev_total_index = 0u32;
            let mut rnd_total_index = 0u32;

            for sc in &scenarios {
                // Run Random Baseline
                let mut rnd = RandomAgent::new(seed);
                let (score_rnd, _) = ArenaRunner::run_scenario(*sc, &mut rnd, seed, ticks);
                rnd_total_index += score_rnd.arena_index;

                // Run Zev Decision Agent
                let mut zev = ZevDecisionAgent::new();
                let (score_zev, _) = ArenaRunner::run_scenario(*sc, &mut zev, seed, ticks);
                zev_total_index += score_zev.arena_index;

                println!(
                    "{:<28} {:<18} {:<12.1} {:<9.1}% {:<12}",
                    sc.to_string(),
                    score_rnd.agent_name,
                    score_rnd.total_reward,
                    score_rnd.success_rate,
                    score_rnd.arena_index
                );
                println!(
                    "{:<28} {:<18} {:<12.1} {:<9.1}% {:<12}",
                    sc.to_string(),
                    score_zev.agent_name.yellow(),
                    score_zev.total_reward,
                    score_zev.success_rate,
                    format!("{}/1000", score_zev.arena_index).green().bold()
                );
                println!("{}", "-".repeat(80));
            }

            println!("\n{}", "Benchmark Summary:".bold());
            println!("  Random Baseline Overall Score: {} / 4000", rnd_total_index);
            println!("  Zev Decision Agent Overall   : {} / 4000 ({:+.1}% over baseline)",
                zev_total_index.to_string().green().bold(),
                ((zev_total_index as f64 - rnd_total_index as f64) / rnd_total_index.max(1) as f64) * 100.0
            );
        }

        Commands::Replay { file, verbose } => {
            println!(
                "{} replay file: {}",
                "Loading".cyan().bold(),
                file.display().to_string().yellow()
            );

            let replay = RunReplay::load_from_file(&file)?;
            println!("\n{}", "Replay Metadata:".bold());
            println!("  Run ID       : {}", replay.run_id);
            println!("  Scenario     : {}", replay.scenario);
            println!("  Agent        : {}", replay.agent_name.yellow());
            println!("  Random Seed  : {}", replay.seed);
            println!("  Total Steps  : {}", replay.steps_count);
            println!("  Total Reward : {:.2}", replay.total_reward);

            if verbose {
                println!("\n{}", "Execution Trace:".bold());
                for step in &replay.steps {
                    println!(
                        "  [Tick {:<3}] Action: {:<25} Reward: {:+6.2} | {}",
                        step.tick,
                        step.action.name.cyan(),
                        step.reward,
                        step.observation.state_summary
                    );
                }
            }
        }

        Commands::Scenarios => {
            println!("{}", "AgentArena Simulation Scenarios:".cyan().bold());

            println!("\n1. {}", "Traffic Grid (inspired by SUMO & UXSim)".yellow().bold());
            println!("   - State : Intersecting arterial queues, phase elapsed times, vehicle arrivals.");
            println!("   - Action: switch_phase_all, switch_heaviest, hold_phase.");
            println!("   - Goal  : Maximize vehicle throughput and prevent gridlock bottlenecks.");

            println!("\n2. {}", "Tactical Combat (inspired by TacticalAI)".yellow().bold());
            println!("   - State : Friendly vs enemy fireteams, line-of-sight, cover status, ammo reserves.");
            println!("   - Action: advance_to_cover, flank_enemy, suppress_fire, tactical_retreat.");
            println!("   - Goal  : Neutralize hostile forces while minimizing friendly casualties.");

            println!("\n3. {}", "Crowd Evacuation (inspired by DetourCrowd)".yellow().bold());
            println!("   - State : Pedestrian egress corridors, density turbulence, bottleneck crush risk.");
            println!("   - Action: open_auxiliary_exit, meter_main_corridor, deploy_staggered_barriers.");
            println!("   - Goal  : Rapidly clear venue with zero crowd crush injuries.");

            println!("\n4. {}", "Software Engineering Team (inspired by ThreeManTeam & Paperclip)".yellow().bold());
            println!("   - State : Multi-agent backlog (Architect, Implementer, Reviewer), token expenditures.");
            println!("   - Action: dispatch_task, conduct_code_review, run_automated_tests, optimize_token_budget.");
            println!("   - Goal  : Deliver verified features with minimal regression rates and token costs.");
        }
    }

    Ok(())
}
