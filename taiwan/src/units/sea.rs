use std::collections::HashMap;
use crate::units::{
    UnitStatus, UnitStats, Arsenal, Position, MilitaryUnit,
    Movable, Heavy, Damageable, Attritable, Bombable, Sailable,
    MissileType, Missile, UnitError, UnitResult,
};
use crate::map::terrain::TerrainBonus;

#[derive(Debug, Clone)]
pub struct Ship {
    pub name: String,
    pub faction: String,
    pub position: Position,
    pub stats: UnitStats,
    pub arsenal: Arsenal,
    pub status: UnitStatus,
    pub hull_integrity: f32,    // 0.0 to 1.0
    pub damage_control: f32,    // Rate of hull repair
    pub capabilities: ShipCapabilities,
    pub sensors: SensorSuite,
}

#[derive(Debug, Clone)]
pub struct ShipCapabilities {
    pub max_speed: f32,         // In knots
    pub lift_capacity: f32,     // In tonnes
    pub fuel_capacity: f32,     // In tonnes
    pub missile_defense: f32,   // Defense effectiveness
    pub gun_range: f32,         // In km
    pub gun_damage: f32,        // Base damage
    pub torpedo_range: f32,     // In km
    pub torpedo_damage: f32,    // Base damage
    pub aa_range: f32,          // In km
    pub aa_ceiling: f32,        // In meters
}

#[derive(Debug, Clone)]
pub struct SensorSuite {
    pub radar_range: f32,       // Detection range in km
    pub radar_accuracy: f32,    // 0.0 to 1.0
    pub sonar_range: f32,       // Detection range in km
    pub sonar_accuracy: f32,    // 0.0 to 1.0
    pub visibility: f32,        // Visual signature
    pub radar_signature: f32,   // Radar cross-section
    pub sonar_signature: f32,   // Acoustic signature
}

#[derive(Debug, Clone)]
pub struct AircraftCarrier {
    pub base: Ship,
    pub aircraft_capacity: i32,
    pub flight_deck_status: f32,    // 0.0 to 1.0
    pub launch_capability: f32,     // 0.0 to 1.0
}

#[derive(Debug, Clone)]
pub struct Submarine {
    pub base: Ship,
    pub depth_rating: f32,          // Maximum depth in meters
    pub is_nuclear: bool,
    pub stealth_rating: f32,        // 0.0 to 1.0
    pub current_depth: f32,
}

#[derive(Debug, Clone)]
pub struct Cruiser {
    pub base: Ship,
    pub air_defense_rating: f32,    // 0.0 to 1.0
    pub command_capability: f32,    // Command & control effectiveness
}

#[derive(Debug, Clone)]
pub struct Destroyer {
    pub base: Ship,
    pub asw_capability: f32,        // Anti-submarine warfare effectiveness
    pub escort_rating: f32,         // Escort effectiveness
}

#[derive(Debug, Clone)]
pub struct SAG {
    pub ships: Vec<Ship>,
    pub formation_status: FormationStatus,
    pub command_ship_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormationStatus {
    Cruise,
    Combat,
    Dispersed,
    Defensive,
}

impl Ship {
    pub fn new(name: String, faction: String, ship_type: &str) -> Self {
        let (stats, capabilities, sensors) = match ship_type {
            "Destroyer" => (
                UnitStats {
                    strength: 100,
                    morale: 1.0,
                    training: 0.8,
                    fatigue: 0.0,
                    supply_level: 1.0,
                },
                ShipCapabilities {
                    max_speed: 30.0,
                    lift_capacity: 1000.0,
                    fuel_capacity: 500.0,
                    missile_defense: 0.7,
                    gun_range: 20.0,
                    gun_damage: 50.0,
                    torpedo_range: 15.0,
                    torpedo_damage: 80.0,
                    aa_range: 10.0,
                    aa_ceiling: 10000.0,
                },
                SensorSuite {
                    radar_range: 100.0,
                    radar_accuracy: 0.8,
                    sonar_range: 25.0,
                    sonar_accuracy: 0.7,
                    visibility: 0.8,
                    radar_signature: 0.7,
                    sonar_signature: 0.6,
                }
            ),
            "Cruiser" => (
                UnitStats {
                    strength: 150,
                    morale: 1.0,
                    training: 0.8,
                    fatigue: 0.0,
                    supply_level: 1.0,
                },
                ShipCapabilities {
                    max_speed: 32.0,
                    lift_capacity: 1500.0,
                    fuel_capacity: 800.0,
                    missile_defense: 0.8,
                    gun_range: 25.0,
                    gun_damage: 70.0,
                    torpedo_range: 15.0,
                    torpedo_damage: 80.0,
                    aa_range: 15.0,
                    aa_ceiling: 15000.0,
                },
                SensorSuite {
                    radar_range: 150.0,
                    radar_accuracy: 0.9,
                    sonar_range: 20.0,
                    sonar_accuracy: 0.6,
                    visibility: 0.9,
                    radar_signature: 0.8,
                    sonar_signature: 0.7,
                }
            ),
            // Add other ship type configurations...
            _ => Default::default(),
        };

        Ship {
            name,
            faction,
            position: Position {
                x: 0.0,
                y: 0.0,
                heading: 0.0,
                altitude: None,
                depth: Some(0.0),
            },
            stats,
            arsenal: Arsenal::default(),
            status: UnitStatus::Active,
            hull_integrity: 1.0,
            damage_control: 1.0,
            capabilities,
            sensors,
        }
    }

    pub fn repair_hull(&mut self, hours: f32) {
        let repair_rate = 0.1 * self.damage_control * hours;
        self.hull_integrity = (self.hull_integrity + repair_rate).min(1.0);
    }
}

impl AircraftCarrier {
    pub fn new(name: String, faction: String) -> Self {
        AircraftCarrier {
            base: Ship::new(name, faction, "Carrier"),
            aircraft_capacity: 75,
            flight_deck_status: 1.0,
            launch_capability: 1.0,
        }
    }

    pub fn calculate_launch_effectiveness(&self) -> f32 {
        self.flight_deck_status * self.launch_capability * self.base.stats.training
    }
}

impl Submarine {
    pub fn new(name: String, faction: String, is_nuclear: bool) -> Self {
        let mut sub = Submarine {
            base: Ship::new(name, faction, "Submarine"),
            depth_rating: 400.0,
            is_nuclear,
            stealth_rating: 0.9,
            current_depth: 0.0,
        };

        if is_nuclear {
            sub.base.capabilities.max_speed += 5.0;
            sub.base.capabilities.fuel_capacity *= 3.0;
        }

        sub
    }

    pub fn set_depth(&mut self, depth: f32) {
        self.current_depth = depth.min(self.depth_rating);
        self.base.position.depth = Some(self.current_depth);
    }
}

impl MilitaryUnit for Ship {
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
        let target_altitude = target.get_position().altitude.unwrap_or(0.0);

        if target_altitude > 0.0 {
            // Air target
            distance <= self.capabilities.aa_range && target_altitude <= self.capabilities.aa_ceiling
        } else if target.get_position().depth.is_some() {
            // Submarine target
            distance <= self.capabilities.torpedo_range
        } else {
            // Surface target
            distance <= self.capabilities.gun_range
        }
    }

    fn calculate_damage(&self, target: &dyn MilitaryUnit) -> f32 {
        let base_damage = if target.get_position().altitude.unwrap_or(0.0) > 0.0 {
            // Air target
            self.capabilities.aa_range * 0.5
        } else if target.get_position().depth.is_some() {
            // Submarine target
            self.capabilities.torpedo_damage
        } else {
            // Surface target
            self.capabilities.gun_damage
        };

        base_damage * self.hull_integrity * self.stats.training
    }

    fn receive_damage(&mut self, damage: f32) {
        let effective_damage = damage * (1.0 - self.capabilities.missile_defense * 0.5);
        self.hull_integrity *= 1.0 - effective_damage;
        
        // Update status based on damage
        if self.hull_integrity <= 0.0 {
            self.status = UnitStatus::Destroyed;
        } else if self.hull_integrity < 0.25 {
            self.status = UnitStatus::Disabled;
        }

        // Morale impact from damage
        self.stats.morale *= 1.0 - (effective_damage * 0.3);
    }

    fn update_supply(&mut self, supply_rate: f32) {
        self.stats.supply_level = (self.stats.supply_level + supply_rate).min(1.0);
    }
}

impl Sailable for Ship {
    fn set_depth(&mut self, depth: Option<f32>) {
        self.position.depth = depth;
    }

    fn get_depth(&self) -> Option<f32> {
        self.position.depth
    }

    fn get_max_depth(&self) -> Option<f32> {
        Some(0.0) // Surface ships can't submerge
    }
}

impl Heavy for Ship {
    fn weight(&self) -> f32 {
        self.capabilities.lift_capacity * 2.0 // Approximate ship weight
    }
}

impl SAG {
    pub fn new(command_ship: Ship) -> Self {
        SAG {
            ships: vec![command_ship],
            formation_status: FormationStatus::Cruise,
            command_ship_index: 0,
        }
    }

    pub fn add_ship(&mut self, ship: Ship) {
        self.ships.push(ship);
    }

    pub fn get_combat_effectiveness(&self) -> f32 {
        let command_ship = &self.ships[self.command_ship_index];
        let base_effectiveness = self.ships.iter()
            .map(|ship| ship.hull_integrity * ship.stats.training)
            .sum::<f32>() / self.ships.len() as f32;

        let formation_modifier = match self.formation_status {
            FormationStatus::Combat => 1.2,
            FormationStatus::Defensive => 1.1,
            FormationStatus::Cruise => 1.0,
            FormationStatus::Dispersed => 0.8,
        };

        base_effectiveness * formation_modifier * command_ship.stats.training
    }

    pub fn set_formation(&mut self, status: FormationStatus) {
        self.formation_status = status;
    }
}

// Helper functions
impl Ship {
    fn calculate_distance(&self, target: &dyn MilitaryUnit) -> f32 {
        let target_pos = target.get_position();
        let dx = target_pos.x - self.position.x;
        let dy = target_pos.y - self.position.y;
        ((dx * dx + dy * dy) as f32).sqrt()
    }

    pub fn detect_submarine(&self, submarine: &Submarine) -> bool {
        let distance = self.calculate_distance(submarine);
        if distance > self.sensors.sonar_range {
            return false;
        }

        let depth_modifier = (submarine.current_depth / 100.0).min(1.0);
        let detection_chance = self.sensors.sonar_accuracy 
            * (1.0 - submarine.stealth_rating)
            * (1.0 - depth_modifier)
            * self.stats.training;

        rand::random::<f32>() < detection_chance
    }

    pub fn can_intercept_missile(&self, distance: f32, missile_speed: f32) -> bool {
        let reaction_time = distance / missile_speed;
        let intercept_chance = self.capabilities.missile_defense 
            * self.stats.training 
            * (1.0 - reaction_time / 10.0).max(0.0);

        rand::random::<f32>() < intercept_chance
    }
}