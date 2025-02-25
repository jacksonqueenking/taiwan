use std::collections::HashMap;
use crate::units::{
    UnitStatus, UnitStats, Arsenal, Position, MilitaryUnit,
    Movable, Heavy, Damageable, Attritable, Bombable, Flyable,
    MissileType, Missile, UnitError, UnitResult,
};
use crate::map::terrain::TerrainBonus;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FighterGeneration {
    Fourth,
    FourthPointFive,
    Fifth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BomberType {
    Stealth,
    NonStealth,
}

#[derive(Debug, Clone)]
pub struct AirUnit {
    pub name: String,
    pub faction: String,
    pub position: Position,
    pub stats: UnitStats,
    pub arsenal: Arsenal,
    pub status: UnitStatus,
    pub stealth: f32,           // 0.0 to 1.0
    pub radar_signature: f32,   // Lower is better
    pub capabilities: AirCapabilities,
    pub maintenance: MaintenanceProfile,
}

#[derive(Debug, Clone)]
pub struct FighterSquadron {
    pub base: AirUnit,
    pub generation: FighterGeneration,
    pub air_to_air: f32,    // Air combat effectiveness
    pub air_to_ground: f32, // Ground attack effectiveness
}

#[derive(Debug, Clone)]
pub struct BomberSquadron {
    pub base: AirUnit,
    pub bomber_type: BomberType,
    pub payload_capacity: f32,
}

#[derive(Debug, Clone)]
pub struct AirCapabilities {
    pub combat_radius: f32,     // Range in km
    pub max_speed: f32,         // Mach
    pub service_ceiling: f32,   // Maximum altitude in meters
    pub radar_range: f32,       // Detection range in km
    pub ecm_strength: f32,      // Electronic countermeasures
}

#[derive(Debug, Clone)]
pub struct MaintenanceProfile {
    pub flight_hours: f32,          // Hours until maintenance required
    pub maintenance_hours: f32,     // Hours of maintenance needed
    pub reliability: f32,           // 0.0 to 1.0
    pub sortie_rate: f32,          // Missions per day
}

impl FighterSquadron {
    pub fn new(name: String, generation: FighterGeneration, faction: String) -> Self {
        let (stealth, radar_sig, air_air, air_ground, capabilities) = match generation {
            FighterGeneration::Fifth => (
                0.9,    // High stealth
                0.2,    // Low radar signature
                1.0,    // Best air-to-air
                0.8,    // Good air-to-ground
                AirCapabilities {
                    combat_radius: 1000.0,
                    max_speed: 1.8,
                    service_ceiling: 15000.0,
                    radar_range: 150.0,
                    ecm_strength: 0.9,
                }
            ),
            FighterGeneration::FourthPointFive => (
                0.6,    // Moderate stealth
                0.4,    // Moderate radar signature
                0.9,    // Very good air-to-air
                0.7,    // Decent air-to-ground
                AirCapabilities {
                    combat_radius: 800.0,
                    max_speed: 1.6,
                    service_ceiling: 14000.0,
                    radar_range: 120.0,
                    ecm_strength: 0.7,
                }
            ),
            FighterGeneration::Fourth => (
                0.3,    // Low stealth
                0.6,    // High radar signature
                0.7,    // Good air-to-air
                0.5,    // Basic air-to-ground
                AirCapabilities {
                    combat_radius: 600.0,
                    max_speed: 1.4,
                    service_ceiling: 12000.0,
                    radar_range: 100.0,
                    ecm_strength: 0.5,
                }
            ),
        };

        FighterSquadron {
            base: AirUnit {
                name,
                faction,
                position: Position {
                    x: 0.0,
                    y: 0.0,
                    heading: 0.0,
                    altitude: Some(0.0),
                    depth: None,
                },
                stats: UnitStats {
                    strength: 24,  // Standard squadron size
                    morale: 1.0,
                    training: 0.8,
                    fatigue: 0.0,
                    supply_level: 1.0,
                },
                arsenal: Arsenal::default(),
                status: UnitStatus::Active,
                stealth,
                radar_signature: radar_sig,
                capabilities,
                maintenance: MaintenanceProfile {
                    flight_hours: 0.0,
                    maintenance_hours: 0.0,
                    reliability: 0.95,
                    sortie_rate: 3.0,
                },
            },
            generation,
            air_to_air,
            air_to_ground,
        }
    }
}

impl BomberSquadron {
    pub fn new(name: String, bomber_type: BomberType, faction: String) -> Self {
        let (stealth, radar_sig, payload, capabilities) = match bomber_type {
            BomberType::Stealth => (
                0.95,   // Very high stealth
                0.15,   // Very low radar signature
                1.0,    // Standard payload
                AirCapabilities {
                    combat_radius: 2000.0,
                    max_speed: 0.9,
                    service_ceiling: 15000.0,
                    radar_range: 100.0,
                    ecm_strength: 0.95,
                }
            ),
            BomberType::NonStealth => (
                0.2,    // Low stealth
                0.8,    // High radar signature
                1.2,    // Higher payload capacity
                AirCapabilities {
                    combat_radius: 1500.0,
                    max_speed: 0.8,
                    service_ceiling: 13000.0,
                    radar_range: 80.0,
                    ecm_strength: 0.6,
                }
            ),
        };

        BomberSquadron {
            base: AirUnit {
                name,
                faction,
                position: Position {
                    x: 0.0,
                    y: 0.0,
                    heading: 0.0,
                    altitude: Some(0.0),
                    depth: None,
                },
                stats: UnitStats {
                    strength: 12,  // Standard bomber squadron size
                    morale: 1.0,
                    training: 0.8,
                    fatigue: 0.0,
                    supply_level: 1.0,
                },
                arsenal: Arsenal::default(),
                status: UnitStatus::Active,
                stealth,
                radar_signature: radar_sig,
                capabilities,
                maintenance: MaintenanceProfile {
                    flight_hours: 0.0,
                    maintenance_hours: 0.0,
                    reliability: 0.90,
                    sortie_rate: 2.0,
                },
            },
            bomber_type,
            payload_capacity: payload,
        }
    }
}

impl MilitaryUnit for FighterSquadron {
    fn get_position(&self) -> &Position {
        &self.base.position
    }

    fn get_stats(&self) -> &UnitStats {
        &self.base.stats
    }

    fn get_arsenal(&self) -> &Arsenal {
        &self.base.arsenal
    }

    fn get_status(&self) -> UnitStatus {
        self.base.status
    }

    fn can_attack(&self, target: &dyn MilitaryUnit) -> bool {
        if self.base.stats.supply_level < 0.1 || self.base.arsenal.ammunition == 0 {
            return false;
        }

        // Check if target is within combat radius
        let distance = self.calculate_distance(target);
        if distance > self.base.capabilities.combat_radius {
            return false;
        }

        // Check altitude limitations
        if let Some(target_alt) = target.get_position().altitude {
            target_alt <= self.base.capabilities.service_ceiling
        } else {
            true // Can attack surface targets
        }
    }

    fn calculate_damage(&self, target: &dyn MilitaryUnit) -> f32 {
        let effectiveness = if target.get_position().altitude.is_some() {
            self.air_to_air
        } else {
            self.air_to_ground
        };

        let generation_bonus = match self.generation {
            FighterGeneration::Fifth => 1.3,
            FighterGeneration::FourthPointFive => 1.1,
            FighterGeneration::Fourth => 1.0,
        };

        effectiveness * generation_bonus * self.base.stats.training
    }

    fn receive_damage(&mut self, damage: f32) {
        // Fighters are more vulnerable but more agile
        let actual_damage = damage * (1.0 - self.base.stealth * 0.5);
        self.base.stats.strength = (self.base.stats.strength as f32 * (1.0 - actual_damage)) as i32;
        
        if self.base.stats.strength <= 0 {
            self.base.status = UnitStatus::Destroyed;
        } else if self.base.stats.strength < self.base.stats.strength / 4 {
            self.base.status = UnitStatus::Disabled;
        }
    }

    fn update_supply(&mut self, supply_rate: f32) {
        self.base.stats.supply_level = (self.base.stats.supply_level + supply_rate).min(1.0);
    }
}

impl MilitaryUnit for BomberSquadron {
    fn get_position(&self) -> &Position {
        &self.base.position
    }

    fn get_stats(&self) -> &UnitStats {
        &self.base.stats
    }

    fn get_arsenal(&self) -> &Arsenal {
        &self.base.arsenal
    }

    fn get_status(&self) -> UnitStatus {
        self.base.status
    }

    fn can_attack(&self, target: &dyn MilitaryUnit) -> bool {
        if self.base.stats.supply_level < 0.1 || self.base.arsenal.ammunition == 0 {
            return false;
        }

        // Bombers can only attack surface targets
        if target.get_position().altitude.is_some() {
            return false;
        }

        // Check if target is within combat radius
        let distance = self.calculate_distance(target);
        distance <= self.base.capabilities.combat_radius
    }

    fn calculate_damage(&self, target: &dyn MilitaryUnit) -> f32 {
        let base_damage = self.payload_capacity * self.base.stats.training;
        
        let target_hardness = if let Some(bombable) = target as Option<&dyn Bombable> {
            bombable.get_hardness()
        } else {
            0.5 // Default hardness
        };

        base_damage * (1.0 - target_hardness)
    }

    fn receive_damage(&mut self, damage: f32) {
        // Bombers are more vulnerable but have better damage control
        let actual_damage = damage * (1.0 - self.base.stealth * 0.3);
        self.base.stats.strength = (self.base.stats.strength as f32 * (1.0 - actual_damage)) as i32;
        
        if self.base.stats.strength <= 0 {
            self.base.status = UnitStatus::Destroyed;
        } else if self.base.stats.strength < self.base.stats.strength / 4 {
            self.base.status = UnitStatus::Disabled;
        }
    }

    fn update_supply(&mut self, supply_rate: f32) {
        self.base.stats.supply_level = (self.base.stats.supply_level + supply_rate).min(1.0);
    }
}

impl Flyable for FighterSquadron {
    fn set_altitude(&mut self, altitude: f32) {
        self.base.position.altitude = Some(altitude.min(self.base.capabilities.service_ceiling));
    }

    fn get_altitude(&self) -> f32 {
        self.base.position.altitude.unwrap_or(0.0)
    }

    fn get_ceiling(&self) -> f32 {
        self.base.capabilities.service_ceiling
    }
}

impl Flyable for BomberSquadron {
    fn set_altitude(&mut self, altitude: f32) {
        self.base.position.altitude = Some(altitude.min(self.base.capabilities.service_ceiling));
    }

    fn get_altitude(&self) -> f32 {
        self.base.position.altitude.unwrap_or(0.0)
    }

    fn get_ceiling(&self) -> f32 {
        self.base.capabilities.service_ceiling
    }
}

// Helper functions
impl FighterSquadron {
    fn calculate_distance(&self, target: &dyn MilitaryUnit) -> f32 {
        let target_pos = target.get_position();
        let dx = target_pos.x - self.base.position.x;
        let dy = target_pos.y - self.base.position.y;
        ((dx * dx + dy * dy) as f32).sqrt()
    }

    pub fn can_intercept(&self, target: &dyn MilitaryUnit) -> bool {
        let distance = self.calculate_distance(target);
        let target_alt = target.get_position().altitude.unwrap_or(0.0);
        
        distance <= self.base.capabilities.radar_range &&
        target_alt <= self.base.capabilities.service_ceiling &&
        self.base.stats.supply_level >= 0.3
    }
}

impl BomberSquadron {
    fn calculate_distance(&self, target: &dyn MilitaryUnit) -> f32 {
        let target_pos = target.get_position();
        let dx = target_pos.x - self.base.position.x;
        let dy = target_pos.y - self.base.position.y;
        ((dx * dx + dy * dy) as f32).sqrt()
    }

    pub fn calculate_mission_risk(&self, distance: f32, enemy_fighters: &[&FighterSquadron]) -> f32 {
        let mut risk = distance / self.base.capabilities.combat_radius * 0.5;
        
        for fighter in enemy_fighters {
            if fighter.can_intercept(self) {
                risk += match fighter.generation {
                    FighterGeneration::Fifth => 0.4,
                    FighterGeneration::FourthPointFive => 0.3,
                    FighterGeneration::Fourth => 0.2,
                } * (1.0 - self.base.stealth);
            }
        }

        risk.min(1.0)
    }
}