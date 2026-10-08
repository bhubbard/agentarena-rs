use crate::types::{Action, Observation, ScenarioType, StepResult};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatUnit {
    pub id: usize,
    pub is_friendly: bool,
    pub health: u32,
    pub ammo: u32,
    pub in_cover: bool,
    pub position_x: i32,
    pub position_y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalMetrics {
    pub friendly_count: usize,
    pub enemy_count: usize,
    pub friendly_casualties: usize,
    pub enemies_eliminated: usize,
    pub friendly_health_remaining: u32,
    pub tactical_advantage_ratio: f64,
}

pub struct TacticalEnvironment {
    pub seed: u64,
    pub tick: u64,
    pub max_ticks: u64,
    pub friendlies: Vec<CombatUnit>,
    pub enemies: Vec<CombatUnit>,
    pub friendly_casualties: usize,
    pub enemies_eliminated: usize,
    rng: StdRng,
}

impl TacticalEnvironment {
    pub fn new(seed: u64, max_ticks: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut friendlies = Vec::new();
        let mut enemies = Vec::new();

        for id in 0..4 {
            friendlies.push(CombatUnit {
                id,
                is_friendly: true,
                health: 100,
                ammo: 120,
                in_cover: rng.gen_bool(0.5),
                position_x: rng.gen_range(0..5),
                position_y: rng.gen_range(0..10),
            });
        }

        for id in 0..4 {
            enemies.push(CombatUnit {
                id,
                is_friendly: false,
                health: 100,
                ammo: 100,
                in_cover: rng.gen_bool(0.5),
                position_x: rng.gen_range(15..20),
                position_y: rng.gen_range(0..10),
            });
        }

        Self {
            seed,
            tick: 0,
            max_ticks,
            friendlies,
            enemies,
            friendly_casualties: 0,
            enemies_eliminated: 0,
            rng,
        }
    }

    pub fn observation(&self) -> Observation {
        let live_friendlies = self.friendlies.iter().filter(|u| u.health > 0).count();
        let live_enemies = self.enemies.iter().filter(|u| u.health > 0).count();
        let total_friendly_hp: u32 = self.friendlies.iter().map(|u| u.health).sum();
        let total_enemy_hp: u32 = self.enemies.iter().map(|u| u.health).sum();

        let ratio = if total_enemy_hp + total_friendly_hp > 0 {
            total_friendly_hp as f64 / (total_enemy_hp + total_friendly_hp) as f64
        } else {
            0.5
        };

        let metrics = serde_json::to_value(TacticalMetrics {
            friendly_count: live_friendlies,
            enemy_count: live_enemies,
            friendly_casualties: self.friendly_casualties,
            enemies_eliminated: self.enemies_eliminated,
            friendly_health_remaining: total_friendly_hp,
            tactical_advantage_ratio: ratio,
        })
        .unwrap_or_default();

        let available_actions = vec![
            "advance_to_cover".to_string(),
            "flank_enemy".to_string(),
            "suppress_fire".to_string(),
            "tactical_retreat".to_string(),
        ];

        let state_summary = format!(
            "Tick {}: Friendlies: {} (HP: {}), Enemies: {} (HP: {}). Tactical Ratio: {:.2}",
            self.tick, live_friendlies, total_friendly_hp, live_enemies, total_enemy_hp, ratio
        );

        Observation {
            tick: self.tick,
            scenario: ScenarioType::TacticalCombat,
            state_summary,
            metrics,
            available_actions,
            terminated: self.tick >= self.max_ticks || live_friendlies == 0 || live_enemies == 0,
        }
    }

    pub fn step(&mut self, action: &Action) -> StepResult {
        let live_friendlies = self.friendlies.iter().filter(|u| u.health > 0).count();
        let live_enemies = self.enemies.iter().filter(|u| u.health > 0).count();

        if self.tick >= self.max_ticks || live_friendlies == 0 || live_enemies == 0 {
            return StepResult {
                observation: self.observation(),
                reward: 0.0,
                terminated: true,
                info: serde_json::json!({ "reason": "Combat ended" }),
            };
        }

        self.tick += 1;

        let mut step_damage_dealt = 0u32;
        let mut step_damage_taken = 0u32;

        // Friendly action resolution
        match action.name.as_str() {
            "advance_to_cover" => {
                for u in &mut self.friendlies {
                    if u.health > 0 {
                        u.in_cover = true;
                        u.position_x = (u.position_x + 2).min(18);
                    }
                }
            }
            "flank_enemy" => {
                for u in &mut self.friendlies {
                    if u.health > 0 && u.ammo >= 15 {
                        u.ammo -= 15;
                        let hit_chance = if u.in_cover { 0.70 } else { 0.55 };
                        if self.rng.gen_bool(hit_chance) {
                            if let Some(target) = self.enemies.iter_mut().find(|e| e.health > 0) {
                                let dmg = self.rng.gen_range(25..45);
                                target.health = target.health.saturating_sub(dmg);
                                step_damage_dealt += dmg;
                                if target.health == 0 {
                                    self.enemies_eliminated += 1;
                                }
                            }
                        }
                    }
                }
            }
            "suppress_fire" => {
                for u in &mut self.friendlies {
                    if u.health > 0 && u.ammo >= 25 {
                        u.ammo -= 25;
                        for e in &mut self.enemies {
                            if e.health > 0 && self.rng.gen_bool(0.4) {
                                let dmg = self.rng.gen_range(10..20);
                                e.health = e.health.saturating_sub(dmg);
                                step_damage_dealt += dmg;
                                e.in_cover = true; // pinned down
                            }
                        }
                    }
                }
            }
            _ => {
                // tactical_retreat or default
                for u in &mut self.friendlies {
                    if u.health > 0 {
                        u.position_x = (u.position_x - 1).max(0);
                        u.in_cover = true;
                    }
                }
            }
        }

        // Enemy AI counter-attack
        for e in &mut self.enemies {
            if e.health > 0 && e.ammo >= 10 {
                e.ammo -= 10;
                let hit_prob = if e.in_cover { 0.40 } else { 0.30 };
                if self.rng.gen_bool(hit_prob) {
                    if let Some(target) = self.friendlies.iter_mut().find(|f| f.health > 0) {
                        let defense = if target.in_cover { 0.5 } else { 1.0 };
                        let dmg = (self.rng.gen_range(15..30) as f32 * defense) as u32;
                        target.health = target.health.saturating_sub(dmg);
                        step_damage_taken += dmg;
                        if target.health == 0 {
                            self.friendly_casualties += 1;
                        }
                    }
                }
            }
        }

        let reward = (step_damage_dealt as f64 * 1.5)
            - (step_damage_taken as f64 * 2.0)
            + (self.enemies_eliminated as f64 * 50.0)
            - (self.friendly_casualties as f64 * 75.0);

        let terminated = self.tick >= self.max_ticks
            || self.friendlies.iter().all(|u| u.health == 0)
            || self.enemies.iter().all(|u| u.health == 0);

        StepResult {
            observation: self.observation(),
            reward,
            terminated,
            info: serde_json::json!({
                "damage_dealt": step_damage_dealt,
                "damage_taken": step_damage_taken,
                "friendly_casualties": self.friendly_casualties,
                "enemies_eliminated": self.enemies_eliminated,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tactical_env_determinism() {
        let mut env1 = TacticalEnvironment::new(42, 10);
        let mut env2 = TacticalEnvironment::new(42, 10);

        let action = Action {
            name: "advance_to_cover".to_string(),
            parameters: serde_json::json!({}),
        };

        let res1 = env1.step(&action);
        let res2 = env2.step(&action);

        assert_eq!(res1.reward, res2.reward);
        assert_eq!(env1.friendlies[0].in_cover, env2.friendlies[0].in_cover);
    }
}
