use std::collections::HashMap;
use rand::Rng;

use crate::game::{
    GameError, GameResult, GameEvent,
    Weather, TimeOfDay,
};
use crate::units::{
    MilitaryUnit, UnitStatus, UnitStats,
    LandUnit, Ship, AirUnit,
    Flyable, Sailable, AntiAir,
    FighterSquadron, BomberSquadron,
};
use crate::map::terrain::TerrainType;

#[derive(Debug, Clone)]
pub struct CombatResult {
    pub attacker_damage_dealt: f32,
    pub defender_damage_dealt: f32,
    pub attacker_losses: i32,
    pub defender_losses: i32,
    pub combat_events: Vec<CombatEvent>,
}

#[derive(Debug, Clone)]
pub enum CombatEvent {
    Hit {
        attacker_id: usize,
        defender_id: usize,
        damage: f32,
    },
    UnitDestroyed {
        unit_id: usize,
    },
    UnitRetreated {
        unit_id: usize,
    },
    MissileIntercepted {
        missile_id: usize,
        interceptor_id: usize,
    },
    SupplyDisrupted {
        unit_id: usize,
    },
}

pub struct CombatModifiers {
    pub weather: f32,
    pub time_of_day: f32,
    pub terrain: f32,
    pub supply: f32,
    pub morale: f32,
    pub entrenchment: f32,
    pub air_superiority: f32,
    pub naval_superiority: f32,
}

impl Default for CombatModifiers {
    fn default() -> Self {
        CombatModifiers {
            weather: 1.0,
            time_of_day: 1.0,
            terrain: 1.0,
            supply: 1.0,
            morale: 1.0,
            entrenchment: 1.0,
            air_superiority: 1.0,
            naval_superiority: 1.0,
        }
    }
}

pub struct CombatResolver {
    rng: rand::rngs::ThreadRng,
    combat_log: Vec<CombatEvent>,
}

impl CombatResolver {
    pub fn new() -> Self {
        CombatResolver {
            rng: rand::thread_rng(),
            combat_log: Vec::new(),
        }
    }

    pub fn resolve_combat(
        &mut self,
        attacker: &mut dyn MilitaryUnit,
        defender: &mut dyn MilitaryUnit,
        terrain: &TerrainType,
        weather: Weather,
        time: TimeOfDay,
        modifiers: Option<CombatModifiers>,
    ) -> GameResult<CombatResult> {
        // Validate combat conditions
        if !self.validate_combat(attacker, defender)? {
            return Err(GameError::InvalidAttack);
        }

        let mods = modifiers.unwrap_or_default();
        let mut result = CombatResult {
            attacker_damage_dealt: 0.0,
            defender_damage_dealt: 0.0,
            attacker_losses: 0,
            defender_losses: 0,
            combat_events: Vec::new(),
        };

        // Calculate base combat values
        let attack_power = self.calculate_attack_power(attacker, defender, &mods);
        let defense_power = self.calculate_defense_power(defender, attacker, terrain, &mods);

        // Resolve initial strike
        let (att_damage, def_damage) = self.resolve_strike(
            attack_power,
            defense_power,
            attacker,
            defender,
            &mods,
        );

        // Apply damage and track events
        self.apply_damage(attacker, def_damage, &mut result);
        self.apply_damage(defender, att_damage, &mut result);

        // Check for special combat effects
        self.handle_special_effects(attacker, defender, &mut result);

        // Update combat results
        result.attacker_damage_dealt = att_damage;
        result.defender_damage_dealt = def_damage;
        result.combat_events.extend(self.combat_log.drain(..));

        Ok(result)
    }

    fn validate_combat(
        &self,
        attacker: &dyn MilitaryUnit,
        defender: &dyn MilitaryUnit,
    ) -> GameResult<bool> {
        // Check if units are capable of combat
        if attacker.get_status() != UnitStatus::Active ||
           defender.get_status() != UnitStatus::Active {
            return Ok(false);
        }

        // Check if attacker has ammunition
        if attacker.get_arsenal().ammunition <= 0 {
            return Ok(false);
        }

        // Check if units are within range
        if !attacker.can_attack(defender) {
            return Ok(false);
        }

        Ok(true)
    }

    fn calculate_attack_power(
        &self,
        attacker: &dyn MilitaryUnit,
        defender: &dyn MilitaryUnit,
        modifiers: &CombatModifiers,
    ) -> f32 {
        let base_power = attacker.calculate_damage(defender);
        let stats = attacker.get_stats();

        base_power
            * modifiers.weather
            * modifiers.time_of_day
            * modifiers.supply
            * stats.morale
            * stats.training
            * (1.0 - stats.fatigue)
    }

    fn calculate_defense_power(
        &self,
        defender: &dyn MilitaryUnit,
        attacker: &dyn MilitaryUnit,
        terrain: &TerrainType,
        modifiers: &CombatModifiers,
    ) -> f32 {
        let base_defense = match defender.get_status() {
            UnitStatus::Entrenched => 1.5,
            UnitStatus::Active => 1.0,
            _ => 0.7,
        };

        let terrain_bonus = terrain.get_defense_multiplier(defender);
        let stats = defender.get_stats();

        base_defense
            * terrain_bonus
            * modifiers.terrain
            * modifiers.entrenchment
            * stats.training
            * (1.0 - stats.fatigue)
    }

    fn resolve_strike(
        &mut self,
        attack_power: f32,
        defense_power: f32,
        attacker: &dyn MilitaryUnit,
        defender: &dyn MilitaryUnit,
        modifiers: &CombatModifiers,
    ) -> (f32, f32) {
        // Calculate hit probabilities
        let att_hit_chance = self.calculate_hit_chance(attacker, defender, modifiers);
        let def_hit_chance = self.calculate_hit_chance(defender, attacker, modifiers);

        // Roll for hits
        let att_hits = self.rng.gen::<f32>() < att_hit_chance;
        let def_hits = self.rng.gen::<f32>() < def_hit_chance;

        // Calculate damages
        let att_damage = if att_hits {
            attack_power * (1.0 - defense_power.min(0.8))
        } else {
            0.0
        };

        let def_damage = if def_hits {
            defense_power * 0.5 * (1.0 - attack_power.min(0.8))
        } else {
            0.0
        };

        // Log combat events
        if att_hits {
            self.combat_log.push(CombatEvent::Hit {
                attacker_id: 0, // TODO: Add proper ID handling
                defender_id: 1,
                damage: att_damage,
            });
        }

        if def_hits {
            self.combat_log.push(CombatEvent::Hit {
                attacker_id: 1,
                defender_id: 0,
                damage: def_damage,
            });
        }

        (att_damage, def_damage)
    }

    fn calculate_hit_chance(
        &self,
        attacker: &dyn MilitaryUnit,
        defender: &dyn MilitaryUnit,
        modifiers: &CombatModifiers,
    ) -> f32 {
        let base_chance = match attacker {
            _ if defender.get_position().altitude.is_some() => {
                // Air combat
                if let Some(fighter) = attacker as Option<&FighterSquadron> {
                    0.7 + (fighter.air_to_air * 0.3)
                } else {
                    0.5
                }
            }
            _ if defender.get_position().depth.is_some() => {
                // Naval combat
                if let Some(ship) = attacker as Option<&Ship> {
                    0.6 + (ship.capabilities.missile_defense * 0.2)
                } else {
                    0.4
                }
            }
            _ => 0.8, // Ground combat
        };

        let weather_penalty = match modifiers.weather {
            x if x < 0.5 => 0.3,
            x if x < 0.8 => 0.1,
            _ => 0.0,
        };

        let time_penalty = match modifiers.time_of_day {
            x if x < 0.5 => 0.2,
            x if x < 0.8 => 0.1,
            _ => 0.0,
        };

        (base_chance - weather_penalty - time_penalty)
            .max(0.1)
            .min(0.95)
    }

    fn apply_damage(
        &mut self,
        unit: &mut dyn MilitaryUnit,
        damage: f32,
        result: &mut CombatResult,
    ) {
        let initial_strength = unit.get_stats().strength;
        unit.receive_damage(damage);
        let strength_loss = initial_strength - unit.get_stats().strength;

        // Update result statistics
        match unit.get_status() {
            UnitStatus::Destroyed => {
                self.combat_log.push(CombatEvent::UnitDestroyed {
                    unit_id: 0, // TODO: Add proper ID handling
                });
                if unit == result.attacker_losses as _ {
                    result.attacker_losses += strength_loss;
                } else {
                    result.defender_losses += strength_loss;
                }
            }
            UnitStatus::Retreating => {
                self.combat_log.push(CombatEvent::UnitRetreated {
                    unit_id: 0, // TODO: Add proper ID handling
                });
            }
            _ => {}
        }
    }

    fn handle_special_effects(
        &mut self,
        attacker: &mut dyn MilitaryUnit,
        defender: &mut dyn MilitaryUnit,
        result: &mut CombatResult,
    ) {
        // Handle missile interception
        if let Some(interceptor) = attacker as Option<&dyn AntiAir> {
            if defender.get_position().altitude.is_some() &&
               interceptor.can_engage_air(defender as &dyn Flyable) {
                self.combat_log.push(CombatEvent::MissileIntercepted {
                    missile_id: 0, // TODO: Add proper ID handling
                    interceptor_id: 0,
                });
            }
        }

        // Handle supply disruption
        if attacker.calculate_damage(defender) > 0.5 {
            self.combat_log.push(CombatEvent::SupplyDisrupted {
                unit_id: 0, // TODO: Add proper ID handling
            });
        }
    }
}

// Helper functions for combat calculations
fn calculate_terrain_advantage(
    unit: &dyn MilitaryUnit,
    terrain: &TerrainType,
) -> f32 {
    match unit {
        _ if terrain.is_urban() => 1.2,  // Urban combat bonus
        _ if terrain.is_forest() => 1.1,  // Forest combat bonus
        _ if terrain.is_mountain() => 0.8, // Mountain combat penalty
        _ => 1.0,
    }
}

fn calculate_weather_effect(
    weather: Weather,
    unit: &dyn MilitaryUnit,
) -> f32 {
    match weather {
        Weather::Clear => 1.0,
        Weather::Rain => 0.8,
        Weather::Storm => 0.6,
        Weather::Fog => 0.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_resolution() {
        let mut resolver = CombatResolver::new();
        // TODO: Add comprehensive combat resolution tests
    }

    #[test]
    fn test_hit_chance_calculation() {
        let resolver = CombatResolver::new();
        // TODO: Add hit chance calculation tests
    }
}