use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainBonus {
    pub offensive_multiplier: f32,
    pub defensive_multiplier: f32,
    pub speed_multiplier: f32,
    pub concealment_bonus: f32,
    pub entrenchment_multiplier: f32,
}

impl Default for TerrainBonus {
    fn default() -> Self {
        TerrainBonus {
            offensive_multiplier: 1.0,
            defensive_multiplier: 1.0,
            speed_multiplier: 1.0,
            concealment_bonus: 0.0,
            entrenchment_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnitClass {
    Infantry,
    Mechanized,
    Armor,
    Artillery,
    AntiAir,
    NavalShip,
    Aircraft,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainAttributes {
    pub movement_cost: HashMap<UnitClass, f32>,
    pub visibility_range: f32,
    pub supply_modifier: f32,
    pub air_defense_bonus: f32,
    pub naval_access: bool,
}

#[derive(Debug, Clone)]
pub struct TerrainType {
    pub name: String,
    pub base_bonus: TerrainBonus,
    pub attributes: TerrainAttributes,
    pub elevation: f32,
    pub is_coastal: bool,
}

impl TerrainType {
    pub fn new(name: &str) -> Self {
        TerrainType {
            name: name.to_string(),
            base_bonus: TerrainBonus::default(),
            attributes: TerrainAttributes {
                movement_cost: HashMap::new(),
                visibility_range: 10.0,
                supply_modifier: 1.0,
                air_defense_bonus: 0.0,
                naval_access: false,
            },
            elevation: 0.0,
            is_coastal: false,
        }
    }

    pub fn get_movement_cost(&self, unit_class: UnitClass) -> f32 {
        *self.attributes.movement_cost.get(&unit_class).unwrap_or(&1.0)
    }

    pub fn calculate_combat_modifiers(&self, attacker_class: UnitClass, defender_class: UnitClass, 
                                    time_of_day: TimeOfDay, weather: Weather) -> CombatModifiers {
        let mut modifiers = CombatModifiers::default();
        
        // Apply base terrain bonuses
        modifiers.attack_modifier *= self.base_bonus.offensive_multiplier;
        modifiers.defense_modifier *= self.base_bonus.defensive_multiplier;
        
        // Apply time of day effects
        modifiers.apply_time_of_day(time_of_day);
        
        // Apply weather effects
        modifiers.apply_weather(weather);
        
        // Special case modifiers based on unit class combinations
        match (attacker_class, defender_class) {
            (UnitClass::Infantry, _) if self.elevation > 1000.0 => {
                modifiers.attack_modifier *= 1.2; // Infantry bonus in high elevation
            },
            (UnitClass::Armor, _) if self.name == "Urban" => {
                modifiers.attack_modifier *= 0.7; // Tanks penalty in urban terrain
            },
            _ => {}
        }
        
        modifiers
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CombatModifiers {
    pub attack_modifier: f32,
    pub defense_modifier: f32,
    pub spotting_range: f32,
    pub supply_consumption: f32,
}

impl Default for CombatModifiers {
    fn default() -> Self {
        CombatModifiers {
            attack_modifier: 1.0,
            defense_modifier: 1.0,
            spotting_range: 1.0,
            supply_consumption: 1.0,
        }
    }
}

impl CombatModifiers {
    fn apply_time_of_day(&mut self, time: TimeOfDay) {
        match time {
            TimeOfDay::Dawn | TimeOfDay::Dusk => {
                self.spotting_range *= 0.7;
                self.attack_modifier *= 0.9;
            },
            TimeOfDay::Night => {
                self.spotting_range *= 0.4;
                self.attack_modifier *= 0.7;
                self.defense_modifier *= 1.2;
            },
            TimeOfDay::Day => {} // No modifiers for daytime
        }
    }

    fn apply_weather(&mut self, weather: Weather) {
        match weather {
            Weather::Clear => {},
            Weather::Rain => {
                self.spotting_range *= 0.8;
                self.attack_modifier *= 0.9;
                self.supply_consumption *= 1.2;
            },
            Weather::Storm => {
                self.spotting_range *= 0.5;
                self.attack_modifier *= 0.7;
                self.defense_modifier *= 0.9;
                self.supply_consumption *= 1.5;
            },
            Weather::Fog => {
                self.spotting_range *= 0.3;
                self.attack_modifier *= 0.8;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimeOfDay {
    Dawn,
    Day,
    Dusk,
    Night,
}

#[derive(Debug, Clone, Copy)]
pub enum Weather {
    Clear,
    Rain,
    Storm,
    Fog,
}

pub fn create_terrain_types() -> HashMap<String, TerrainType> {
    let mut terrains = HashMap::new();
    
    // Plains
    let mut plains = TerrainType::new("Plains");
    plains.base_bonus = TerrainBonus {
        offensive_multiplier: 1.0,
        defensive_multiplier: 0.8,
        speed_multiplier: 1.2,
        concealment_bonus: 0.1,
        entrenchment_multiplier: 1.0,
    };
    plains.attributes.movement_cost.insert(UnitClass::Infantry, 1.0);
    plains.attributes.movement_cost.insert(UnitClass::Mechanized, 0.8);
    plains.attributes.movement_cost.insert(UnitClass::Armor, 0.8);
    terrains.insert("Plains".to_string(), plains);
    
    // Urban
    let mut urban = TerrainType::new("Urban");
    urban.base_bonus = TerrainBonus {
        offensive_multiplier: 0.7,
        defensive_multiplier: 1.5,
        speed_multiplier: 0.8,
        concealment_bonus: 0.4,
        entrenchment_multiplier: 1.3,
    };
    urban.attributes.movement_cost.insert(UnitClass::Infantry, 1.0);
    urban.attributes.movement_cost.insert(UnitClass::Mechanized, 1.5);
    urban.attributes.movement_cost.insert(UnitClass::Armor, 2.0);
    urban.attributes.air_defense_bonus = 0.3;
    terrains.insert("Urban".to_string(), urban);
    
    // Forest
    let mut forest = TerrainType::new("Forest");
    forest.base_bonus = TerrainBonus {
        offensive_multiplier: 0.8,
        defensive_multiplier: 1.3,
        speed_multiplier: 0.7,
        concealment_bonus: 0.5,
        entrenchment_multiplier: 1.1,
    };
    forest.attributes.movement_cost.insert(UnitClass::Infantry, 1.2);
    forest.attributes.movement_cost.insert(UnitClass::Mechanized, 1.8);
    forest.attributes.movement_cost.insert(UnitClass::Armor, 2.5);
    forest.attributes.visibility_range = 5.0;
    terrains.insert("Forest".to_string(), forest);
    
    // Mountain
    let mut mountain = TerrainType::new("Mountain");
    mountain.base_bonus = TerrainBonus {
        offensive_multiplier: 0.6,
        defensive_multiplier: 1.8,
        speed_multiplier: 0.5,
        concealment_bonus: 0.3,
        entrenchment_multiplier: 1.4,
    };
    mountain.elevation = 2000.0;
    mountain.attributes.movement_cost.insert(UnitClass::Infantry, 2.0);
    mountain.attributes.movement_cost.insert(UnitClass::Mechanized, 3.0);
    mountain.attributes.movement_cost.insert(UnitClass::Armor, 4.0);
    mountain.attributes.visibility_range = 15.0;
    mountain.attributes.supply_modifier = 0.7;
    terrains.insert("Mountain".to_string(), mountain);
    
    // Coastal
    let mut coastal = TerrainType::new("Coastal");
    coastal.is_coastal = true;
    coastal.base_bonus = TerrainBonus {
        offensive_multiplier: 0.9,
        defensive_multiplier: 1.1,
        speed_multiplier: 0.9,
        concealment_bonus: 0.2,
        entrenchment_multiplier: 0.8,
    };
    coastal.attributes.naval_access = true;
    coastal.attributes.movement_cost.insert(UnitClass::Infantry, 1.2);
    coastal.attributes.movement_cost.insert(UnitClass::Mechanized, 1.5);
    coastal.attributes.movement_cost.insert(UnitClass::Armor, 1.8);
    terrains.insert("Coastal".to_string(), coastal);
    
    terrains
}

pub struct TerrainRules {
    terrain_types: HashMap<String, TerrainType>,
    pub current_time: TimeOfDay,
    pub current_weather: Weather,
}

impl TerrainRules {
    pub fn new() -> Self {
        TerrainRules {
            terrain_types: create_terrain_types(),
            current_time: TimeOfDay::Day,
            current_weather: Weather::Clear,
        }
    }

    pub fn get_terrain(&self, name: &str) -> Option<&TerrainType> {
        self.terrain_types.get(name)
    }

    pub fn calculate_movement_cost(&self, terrain_name: &str, unit_class: UnitClass) -> f32 {
        if let Some(terrain) = self.terrain_types.get(terrain_name) {
            let base_cost = terrain.get_movement_cost(unit_class);
            
            // Apply weather modifications
            let weather_modifier = match self.current_weather {
                Weather::Clear => 1.0,
                Weather::Rain => 1.3,
                Weather::Storm => 1.8,
                Weather::Fog => 1.1,
            };
            
            // Apply time of day modifications
            let time_modifier = match self.current_time {
                TimeOfDay::Day => 1.0,
                TimeOfDay::Night => 1.5,
                TimeOfDay::Dawn | TimeOfDay::Dusk => 1.2,
            };
            
            base_cost * weather_modifier * time_modifier
        } else {
            1.0 // Default cost if terrain type not found
        }
    }
}