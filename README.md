# AgentArena-rs

[![Code Coverage](https://github.com/bhubbard/agentarena-rs/actions/workflows/coverage.yml/badge.svg)](https://github.com/bhubbard/agentarena-rs/actions/workflows/coverage.yml)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](Cargo.toml)

> **Dynamic simulation test bench and deterministic benchmark arena for AI agents across traffic grids, tactical combat, crowd steering, and multi-agent coordination.**

Most agent benchmarks are static, text-based puzzles or brittle web coding tests. **AgentArena** evaluates AI agents inside complex, dynamic, deterministic physical and mathematical simulation environments with verifiable outcome metrics.

---

## 🎯 Benchmark Environments

AgentArena unifies 4 simulation domains derived from real physical and systems dynamics:

1. **Traffic Grid Simulation** (inspired by [`sumo-rs`](https://github.com/bhubbard/sumo-rs) & [`uxsim-rs`](https://github.com/bhubbard/uxsim-rs))
   * **Domain**: Arterial road network with Intelligent Driver Model (IDM) vehicle queues and traffic signal phases.
   * **Agent Role**: Urban Grid Traffic Controller.
   * **Metrics**: Vehicle throughput, total waiting delay, maximum queue buffer, gridlock prevention.

2. **Tactical Combat Simulation** (inspired by [`tactical-ai-rs`](https://github.com/bhubbard/tactical-ai-rs))
   * **Domain**: Squad-level tactical combat with spatial influence maps, cover mechanics, and perception line-of-sight.
   * **Agent Role**: Fireteam Commander.
   * **Metrics**: Enemies eliminated, friendly casualties, ammunition discipline, tactical advantage ratio.

3. **Crowd Evacuation Simulation** (inspired by [`detour-crowd-rs`](https://github.com/bhubbard/detour-crowd-rs))
   * **Domain**: Pedestrian crowd steering with Reciprocal Velocity Obstacle (RVO) collision avoidance and bottleneck density turbulence.
   * **Agent Role**: Emergency Evacuation Director.
   * **Metrics**: Evacuation completion rate, evacuation velocity, crowd crush incident avoidance.

4. **Software Engineering Coordination** (inspired by [`three-man-team-rs`](https://github.com/bhubbard/three-man-team-rs) & [`paperclip-rs`](https://github.com/bhubbard/paperclip-rs))
   * **Domain**: Multi-agent software engineering team (Architect, Implementer, Reviewer) managing task DAGs and token budgets.
   * **Agent Role**: Engineering Lead / Orchestrator.
   * **Metrics**: Verified feature velocity, regression rate, token expenditure ($/task).

---

## 🚀 Quick Start

### Installation & Build
```bash
git clone https://github.com/bhubbard/agentarena-rs.git
cd agentarena-rs
make build
```

### Run an Evaluation
Run an agent on the Traffic Grid scenario:
```bash
cargo run -p agentarena-cli -- run --scenario traffic --agent zev --seed 42 --ticks 30
```

### Run the Full Benchmark Suite
Execute the multi-scenario benchmark leaderboard comparing baseline policies:
```bash
cargo run -p agentarena-cli -- benchmark --seed 42 --ticks 25
```

### Replay & Verification
Every evaluation run is seeded (`u64`) and produces byte-for-byte deterministic replays:
```bash
cargo run -p agentarena-cli -- run -s tactical -a zev -o replay.json
cargo run -p agentarena-cli -- replay -f replay.json --verbose
```

---

## 🏛️ Architecture

```
agentarena-rs/
├── crates/
│   ├── agentarena-core/       # Simulation environments, agent traits, scoring engine
│   │   ├── src/
│   │   │   ├── traffic.rs     # Traffic grid environment (IDM, signals)
│   │   │   ├── tactical.rs    # Tactical squad combat (cover, influence maps)
│   │   │   ├── crowd.rs       # Crowd evacuation (RVO, crush density)
│   │   │   ├── orchestrator.rs# Software team DAG orchestration
│   │   │   ├── agent.rs       # Baseline agents (Random, Zev Zero-Token)
│   │   │   ├── runner.rs      # Scenario execution engine
│   │   │   ├── score.rs       # Arena Index (0-1000) composite score
│   │   │   └── replay.rs      # Deterministic replay serialization
│   └── agentarena-cli/        # High-performance benchmark runner CLI
├── Makefile                   # Standard build, test, and coverage recipes
└── .github/workflows/         # CI code coverage with cargo-llvm-cov
```

---

## 📊 Scoring Rubric: The Arena Index (0–1000)

The **Arena Index** calculates a composite score based on:
* **Success Rate (60%)**: Objective completion (vehicles cleared, enemies eliminated, pedestrians evacuated, features verified).
* **Efficiency Metric (40%)**: Resource conservation (zero gridlock, zero friendly casualties, zero crowd crushes, minimal regressions).

---

## 📜 License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).
