# AgentArena-rs

[![CI](https://github.com/bhubbard/agentarena-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/bhubbard/agentarena-rs/actions/workflows/ci.yml)
[![Code Coverage](https://github.com/bhubbard/agentarena-rs/actions/workflows/coverage.yml/badge.svg)](https://github.com/bhubbard/agentarena-rs/actions/workflows/coverage.yml)
[![Deploy GitHub Pages](https://github.com/bhubbard/agentarena-rs/actions/workflows/pages.yml/badge.svg)](https://code.brandonhubbard.com/agentarena-rs/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg)](Cargo.toml)

> **Dynamic simulation test bench and deterministic benchmark arena for AI agents across traffic grids, tactical combat, crowd steering, and multi-agent software engineering coordination.**

🌐 **Live Interactive Multi-Domain Simulation Arena & ELO Studio**: **[https://code.brandonhubbard.com/agentarena-rs/](https://code.brandonhubbard.com/agentarena-rs/)**  
📊 **Full Empirical Performance Benchmarks**: **[BENCHMARKS.md](BENCHMARKS.md)** (**45.4x faster** than Python Gymnasium / PettingZoo)

Most agent benchmarks are static, text-based puzzles or brittle web coding tests. **AgentArena** evaluates AI agents inside complex, dynamic, deterministic physical and mathematical simulation environments with verifiable outcome metrics, composite **Arena Index (0–1000)** scoring, and head-to-head **FIDE ELO ratings**.

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
   * **Metrics**: Verified feature velocity, regression rate, token expenditure (\$/task).

---

## ⚡ Empirical Performance Benchmarks

Measured across 10,000 full episode rollouts (`1,000,000` environment steps) on AMD EPYC 7763 vs. Python 3.12 (`gymnasium` / `pettingzoo` multi-agent loops):

| Benchmark Workload | `agentarena-rs` (Rust) | Python Gymnasium / PettingZoo | Speedup | Peak Memory |
| :--- | :---: | :---: | :---: | :---: |
| **Crowd Evacuation Rollout** (`100 pedestrians`, RVO crush physics) | **0.76 ms** | 34.50 ms | **45.4x faster** | **2.2 MB** vs 142 MB |
| **Full 4-Domain Tournament Suite** (`Random` vs `ZevZeroToken` + ELO) | **3.39 ms** | 150.40 ms | **44.4x faster** | **3.1 MB** vs 184 MB |
| **Traffic Grid Rollout** (`4 intersections`, IDM queues) | **0.88 ms** | 38.90 ms | **44.2x faster** | **2.4 MB** vs 148 MB |
| **Software Team DAG Rollout** (`6 modules`, token/regression tracking) | **0.81 ms** | 35.80 ms | **44.2x faster** | **2.3 MB** vs 145 MB |
| **Tactical Combat Rollout** (`4v4 squad`, cover & LOS) | **0.94 ms** | 41.20 ms | **43.8x faster** | **2.6 MB** vs 156 MB |
| **Deterministic Replay Serialization** (`RunReplay` JSON round-trip) | **0.14 ms** | 4.85 ms | **34.6x faster** | **1.8 MB** vs 92 MB |

See [BENCHMARKS.md](BENCHMARKS.md) for complete methodology and policy tournament results.

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
Execute the multi-scenario benchmark leaderboard comparing baseline policies and computing head-to-head ELO ratings:
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
│   ├── agentarena-core/       # Simulation environments, agent traits, scoring & ELO engine
│   │   ├── src/
│   │   │   ├── traffic.rs     # Traffic grid environment (IDM, signals)
│   │   │   ├── tactical.rs    # Tactical squad combat (cover, influence maps)
│   │   │   ├── crowd.rs       # Crowd evacuation (RVO, crush density)
│   │   │   ├── orchestrator.rs# Software team DAG orchestration
│   │   │   ├── agent.rs       # Baseline agents (Random, Zev Zero-Token)
│   │   │   ├── runner.rs      # Scenario execution engine
│   │   │   ├── score.rs       # Arena Index (0-1000) & FIDE ELO rating calculator
│   │   │   └── replay.rs      # Deterministic replay serialization
│   └── agentarena-cli/        # High-performance benchmark runner CLI
├── docs/                      # Interactive Multi-Domain AI Arena Web Studio
├── BENCHMARKS.md              # Empirical Rust vs Python simulation benchmarks
├── Makefile                   # Standard build, test, and coverage recipes
└── .github/workflows/         # CI, llvm-cov coverage, and GitHub Pages deployment
```

---

## 📊 Scoring Rubric: The Arena Index (0–1000) & FIDE ELO

The **Arena Index** calculates a composite score based on:
* **Success Rate (60%)**: Objective completion (vehicles cleared, enemies eliminated, pedestrians evacuated, features verified).
* **Efficiency Metric (40%)**: Resource conservation (zero gridlock, zero friendly casualties, zero crowd crushes, minimal regressions).
* **Head-to-Head ELO (`EloCalculator`)**: Computes expected win probabilities $E_A = \frac{1}{1 + 10^{(R_B - R_A)/400}}$ and updates agent ratings with $K = 32$.

---

## 📜 License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).
