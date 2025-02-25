use std::collections::HashMap;
use crate::units::{
    UnitStatus, UnitStats, Arsenal, Position, MilitaryUnit,
    Movable, Heavy, Damageable, Attritable, Bombable,
    MissileType, Missile, UnitError, UnitResult,
};
use crate::map::terrain::TerrainBonus;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LandUnitType {
    Infantry,
    Mechanized,
    Artillery,
    SpecialForces,
    Tank,
    AntiAir,
}

#[derive(Debug, Clone)]
pub struct LandUnit {
    pub name: String,
    pub unit_type: LandUnitType,
    pub faction: String,
    pub position: Position,
    pub stats: UnitStats,
    pub arsenal: Arsenal,
    pub status: UnitStatus,
    pub terrain_bonus: Option<TerrainBonus>,
    pub entrenchment: f32,  // 0.0 to 1.0
    pub visibility: VisibilityProfile,
    pub capabilities: UnitCapabilities,
    pub supply_consumption: SupplyConsumption,
}

#[derive(Debug, Clone)]
pub struct VisibilityProfile {
    pub ground: f32,    // Ground-based visibility
    pub optical: f32,   // Optical/Visual signature
    pub infrared: f32,  // IR signature
    pub radar: f32,     // Radar cross-section
}

#[derive(Debug, Clone)]
pub struct UnitCapabilities {
    pub anti_armor: f32,        // 0.0 to 1.0
    pub close_attack: f32,      // 0-5km effectiveness
    pub medium_attack: f32,     // 5-20km effectiveness
    pub long_attack: f32,       // >20km effectiveness
    pub aa_range: i32,          // Anti-aircraft range in km
    pub aa_altitude: i32,       // Maximum engagement altitude in meters
    pub aa_damage: f32,         // Anti-aircraft damage multiplier
    pub bridging: bool,         // Has bridging equipment
    pub special_forces: bool,   // Special operations capable
}

#[derive(Debug, Clone)]
pub struct SupplyConsumption {
    pub ammo_close: f32,    // Ammo consumption in close combat
    pub ammo_medium: f32,   // Ammo consumption in medium range
    pub ammo_long: f32,     // Ammo consumption in long range
    pub fuel_idle: f32,     // Fuel consumption when stationary
    pub fuel_moving: f32,   // Fuel consumption when moving
}

impl LandUnit {
    pub fn new(
        name: String,
        unit_type: LandUnitType,
        faction: String,
        position: Position,
        strength: i32,
    ) -> Self {
        let (stats, capabilities, consumption) = match unit_type {
            LandUnitType::Infantry => (
                UnitStats {
                    strength,
                    morale: 1.0,
                    training: 0.7,
                    fatigue: 0.0,
                    supply_level: 1.0,
                },
                UnitCapabilities {
                    anti_armor: 0.25,
                    close_attack: 0.4,
                    medium_attack: 0.2,
                    long_attack: 0.1,
                    aa_range: 1,
                    aa_altitude: 1000,
                    aa_damage: 4.26,
                    bridging: false,
                    special_forces: false,
                },
                SupplyConsumption {
                    ammo_close: 2.21,
                    ammo_medium: 1.11,
                    ammo_long: 0.55,
                    fuel_idle: 1.2,
                    fuel_moving: 8.6,
                }
            ),
            LandUnitType::SpecialForces => (
                UnitStats {
                    strength,
                    morale: 2.0,
                    training: 0.9,
                    fatigue: 0.0,
                    supply_level: 1.0,
                },
                UnitCapabilities {
                    anti_armor: 0.25,
                    close_attack: 0.7,
                    medium_attack: 0.4,
                    long_attack: 0.2,
                    aa_range: 2,
                    aa_altitude: 2000,
                    aa_damage: 6.0,
                    bridging: false,
                    special_forces: true,
                },
                SupplyConsumption {
                    ammo_close: 3.5,
                    ammo_medium: 1.75,
                    ammo_long: 0.88,
                    fuel_idle: 0.7,
                    fuel_moving: 5.0,
                }
            ),
            // Add other unit type configurations...
            _ => Default::default(),
        };

        LandUnit {
            name,
            unit_type,
            faction,
            position,
            stats,
            arsenal: Arsenal::default(),
            status: UnitStatus::Active,
            terrain_bonus: None,
            entrenchment: 0.0,
            visibility: VisibilityProfile {
                ground: 1.0,
                optical: 1.0,
                infrared: 1.0,
                radar: 1.0,
            },
            capabilities,
            supply_consumption: consumption,
        }
    }

    pub fn entrench(&mut self, hours: f32) -> f32 {
        let max_entrenchment = match self.unit_type {
            LandUnitType::Infantry => 1.0,
            LandUnitType::Mechanized => 0.7,
            LandUnitType::Artillery => 0.8,
            LandUnitType::SpecialForces => 0.9,
            LandUnitType::Tank => 0.6,
            LandUnitType::AntiAir => 0.7,
        };

        let entrenchment_rate = 0.1 * hours;  // 10% per hour
        self.entrenchment = (self.entrenchment + entrenchment_rate).min(max_entrenchment);
        self.entrenchment
    }

    pub fn calculate_combat_effectiveness(&self) -> f32 {
        let morale_factor = self.stats.morale;
        let supply_factor = self.stats.supply_level;
        let entrenchment_factor = 1.0 + self.entrenchment * 0.5;
        let terrain_factor = self.terrain_bonus.as_ref()
            .map(|bonus| bonus.defensive_multiplier)
            .unwrap_or(1.0);

        morale_factor * supply_factor * entrenchment_factor * terrain_factor
    }
}

impl MilitaryUnit for LandUnit {
    fn get_position(&self) -> &Position {
        &self.position
    }

    fn get_stats(&self) -> &UnitStats {
        &self.stats
    }

    fn get_arsenal(&self) -> &Arsenal {
        &self.arsenal
    }

    fn get_status(&self) -> UnitStatus {
        self.status
    }

    fn can_attack(&self, target: &dyn MilitaryUnit) -> bool {
        if self.stats.supply_level < 0.1 || self.arsenal.ammunition == 0 {
            return false;
        }

        let distance = self.calculate_distance(target);
        match self.unit_type {
            LandUnitType::Infantry | LandUnitType::Mechanized =>
                distance <= 5.0,  // Close combat range
            LandUnitType::Artillery =>
                distance <= 30.0, // Artillery range
            LandUnitType::SpecialForces =>
                distance <= 10.0, // Special forces range
            _ => distance <= 15.0,
        }
    }

    fn calculate_damage(&self, target: &dyn MilitaryUnit) -> f32 {
        let distance = self.calculate_distance(target);
        let base_damage = if distance <= 5.0 {
            self.capabilities.close_attack
        } else if distance <= 20.0 {
            self.capabilities.medium_attack
        } else {
            self.capabilities.long_attack
        };

        let effectiveness = self.calculate_combat_effectiveness();
        base_damage * effectiveness
    }

    fn receive_damage(&mut self, damage: f32) {
        self.stats.strength = (self.stats.strength as f32 * (1.0 - damage)) as i32;
        
        // Update status based on damage
        if self.stats.strength <= 0 {
            self.status = UnitStatus::Destroyed;
        } else if self.stats.strength < self.stats.strength / 4 {
            self.status = UnitStatus::Disabled;
        }

        // Morale impact from damage
        self.stats.morale *= 1.0 - (damage * 0.5);
    }

    fn update_supply(&mut self, supply_rate: f32) {
        self.stats.supply_level = (self.stats.supply_level + supply_rate).min(1.0);
    }
}

impl Movable for LandUnit {
    fn move_to(&mut self, x: f64, y: f64) {
        self.position.x = x;
        self.position.y = y;
        self.entrenchment = 0.0;  // Reset entrenchment when moving
    }

    fn get_speed(&self) -> f32 {
        let base_speed = match self.unit_type {
            LandUnitType::Infantry => 6.44,
            LandUnitType::Mechanized => 32.0,
            LandUnitType::SpecialForces => 10.0,
            LandUnitType::Artillery => 20.0,
            LandUnitType::Tank => 25.0,
            LandUnitType::AntiAir => 22.0,
        };

        let terrain_modifier = self.terrain_bonus.as_ref()
            .map(|bonus| bonus.speed_multiplier)
            .unwrap_or(1.0);

        let supply_modifier = if self.stats.supply_level < 0.5 { 0.7 } else { 1.0 };

        base_speed * terrain_modifier * supply_modifier
    }
}

impl Heavy for LandUnit {
    fn weight(&self) -> f32 {
        match self.unit_type {
            LandUnitType::Infantry => 945.0,
            LandUnitType::Mechanized => 1598.0,
            LandUnitType::SpecialForces => 200.0,
            LandUnitType::Artillery => 1200.0,
            LandUnitType::Tank => 2000.0,
            LandUnitType::AntiAir => 1000.0,
        }
    }
}

impl Damageable for LandUnit {
    fn get_health(&self) -> f32 {
        self.stats.strength as f32 / 100.0
    }

    fn is_disabled(&self) -> bool {
        self.status == UnitStatus::Disabled
    }

    fn is_destroyed(&self) -> bool {
        self.status == UnitStatus::Destroyed
    }
}

impl Attritable for LandUnit {
    fn get_attrition_rate(&self) -> f32 {
        let base_rate = 0.01;  // 1% base attrition
        let supply_modifier = if self.stats.supply_level < 0.3 { 2.0 } else { 1.0 };
        let terrain_modifier = self.terrain_bonus.as_ref()
            .map(|bonus| if bonus.defensive_multiplier < 1.0 { 1.5 } else { 1.0 })
            .unwrap_or(1.0);

        base_rate * supply_modifier * terrain_modifier
    }

    fn apply_attrition(&mut self) {
        let attrition = self.get_attrition_rate();
        self.stats.strength = (self.stats.strength as f32 * (1.0 - attrition)) as i32;
    }
}

impl Bombable for LandUnit {
    fn get_hardness(&self) -> f32 {
        match self.unit_type {
            LandUnitType::Tank => 0.8,
            LandUnitType::Mechanized => 0.6,
            LandUnitType::Artillery | LandUnitType::AntiAir => 0.5,
            LandUnitType::Infantry | LandUnitType::SpecialForces => 0.3,
        }
    }
}

impl LandUnit {
    fn calculate_distance(&self, target: &dyn MilitaryUnit) -> f32 {
        let target_pos = target.get_position();
        let dx = target_pos.x - self.position.x;
        let dy = target_pos.y - self.position.y;
        ((dx * dx + dy * dy) as f32).sqrt()
    }
}