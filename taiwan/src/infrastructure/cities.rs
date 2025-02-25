use std::collections::HashMap;
use crate::infrastructure::{Hub, Targetable, SupplyNode, StorageCapable, DamageType};
use crate::units::{Arsenal, Heavy};
use crate::game::GameError;

#[derive(Debug, Clone)]
pub struct City {
    pub base: Hub,
    pub population: i32,          // Current population
    pub base_population: i32,     // Pre-war population
    pub significance: f32,        // Strategic importance (0.0 to 1.0)
    pub industrial_capacity: f32, // Industrial output (0.0 to 1.0)
    pub civilian_morale: f32,     // Civilian morale (0.0 to 1.0)
    pub defenses: CityDefenses,
    pub facilities: CityFacilities,
    pub damage_state: DamageState,
}

#[derive(Debug, Clone)]
pub struct CityDefenses {
    pub fortification_level: f32,    // 0.0 to 1.0
    pub air_defense_level: f32,      // 0.0 to 1.0
    pub civilian_shelter_level: f32,  // 0.0 to 1.0
    pub early_warning_active: bool,
    pub has_underground_facilities: bool,
}

#[derive(Debug, Clone)]
pub struct CityFacilities {
    pub industrial_zones: i32,
    pub hospitals: i32,
    pub power_plants: i32,
    pub water_treatment: i32,
    pub command_centers: i32,
    pub transportation_hubs: i32,
}

#[derive(Debug, Clone)]
pub struct DamageState {
    pub infrastructure_damage: f32,  // 0.0 to 1.0
    pub industrial_damage: f32,      // 0.0 to 1.0
    pub residential_damage: f32,     // 0.0 to 1.0
    pub civilian_casualties: i32,
    pub power_grid_status: f32,      // 0.0 to 1.0
    pub water_system_status: f32,    // 0.0 to 1.0
}

pub type Cities = HashMap<String, City>;

impl Default for Cities {
    fn default() -> Self {
        let mut cities = HashMap::new();
        
        // Add major Taiwanese cities with realistic data
        cities.insert(
            "Taipei".to_string(),
            City::new(
                "Taipei".to_string(),
                320.0,  // x coordinate
                380.0,  // y coordinate
                100000, // storage capacity
                2600000, // population
                "Taiwan".to_string(),
                1.0,    // significance
                1.0,    // industrial capacity
            )
        );
        
        cities.insert(
            "Kaohsiung".to_string(),
            City::new(
                "Kaohsiung".to_string(),
                540.0,
                100.0,
                100000,
                2700000,
                "Taiwan".to_string(),
                0.7,
                0.8,
            )
        );
        
        cities.insert(
            "Tainan".to_string(),
            City::new(
                "Tainan".to_string(),
                100.0,
                100.0,
                100000,
                1900000,
                "Taiwan".to_string(),
                0.5,
                0.6,
            )
        );
        
        cities
    }
}

impl City {
    pub fn new(
        name: String,
        x: f64,
        y: f64,
        storage: i32,
        population: i32,
        controller: String,
        significance: f32,
        industrial_capacity: f32,
    ) -> Self {
        City {
            base: Hub::new(name, x, y, storage, controller),
            population,
            base_population: population,
            significance,
            industrial_capacity,
            civilian_morale: 1.0,
            defenses: CityDefenses {
                fortification_level: 0.5,
                air_defense_level: 0.5,
                civilian_shelter_level: 0.7,
                early_warning_active: true,
                has_underground_facilities: false,
            },
            facilities: CityFacilities {
                industrial_zones: 5,
                hospitals: 3,
                power_plants: 2,
                water_treatment: 2,
                command_centers: 1,
                transportation_hubs: 2,
            },
            damage_state: DamageState {
                infrastructure_damage: 0.0,
                industrial_damage: 0.0,
                residential_damage: 0.0,
                civilian_casualties: 0,
                power_grid_status: 1.0,
                water_system_status: 1.0,
            },
        }
    }

    pub fn update_population(&mut self) {
        // Calculate population changes based on damage and conditions
        let casualty_rate = self.calculate_casualty_rate();
        let evacuation_rate = self.calculate_evacuation_rate();
        
        let population_change = -(
            (self.population as f32 * casualty_rate) +
            (self.population as f32 * evacuation_rate)
        ) as i32;
        
        self.population = (self.population + population_change).max(0);
        self.update_civilian_morale();
    }

    pub fn calculate_strategic_value(&self) -> f32 {
        let population_factor = self.population as f32 / self.base_population as f32;
        let industry_factor = self.industrial_capacity * (1.0 - self.damage_state.industrial_damage);
        let infrastructure_factor = 1.0 - self.damage_state.infrastructure_damage;
        
        self.significance * (
            0.4 * population_factor +
            0.3 * industry_factor +
            0.3 * infrastructure_factor
        )
    }

    pub fn calculate_supply_generation(&self) -> i32 {
        let base_supply = (self.industrial_capacity * self.base.storage as f32) as i32;
        let damage_modifier = 1.0 - self.damage_state.industrial_damage;
        let power_modifier = self.damage_state.power_grid_status;
        
        (base_supply as f32 * damage_modifier * power_modifier) as i32
    }

    pub fn repair_infrastructure(&mut self, repair_points: f32) {
        // Prioritize critical infrastructure
        let mut remaining_points = repair_points;
        
        // Repair power grid
        if self.damage_state.power_grid_status < 1.0 {
            let repair = remaining_points.min(1.0 - self.damage_state.power_grid_status);
            self.damage_state.power_grid_status += repair;
            remaining_points -= repair;
        }
        
        // Repair water system
        if remaining_points > 0.0 && self.damage_state.water_system_status < 1.0 {
            let repair = remaining_points.min(1.0 - self.damage_state.water_system_status);
            self.damage_state.water_system_status += repair;
            remaining_points -= repair;
        }
        
        // Repair general infrastructure
        if remaining_points > 0.0 {
            self.damage_state.infrastructure_damage = 
                (self.damage_state.infrastructure_damage - remaining_points).max(0.0);
        }
    }

    pub fn handle_bombing_attack(&mut self, damage: f32, target_type: DamageType) {
        let effective_damage = damage * (1.0 - self.defenses.fortification_level);
        
        match target_type {
            DamageType::Structural => {
                self.damage_state.infrastructure_damage += effective_damage;
                self.base.condition *= 1.0 - (effective_damage * 0.5);
            },
            DamageType::Storage => {
                self.base.storage = (self.base.storage as f32 * (1.0 - effective_damage)) as i32;
            },
            DamageType::Logistics => {
                self.damage_state.industrial_damage += effective_damage;
                self.industrial_capacity *= 1.0 - (effective_damage * 0.3);
            },
            DamageType::Complete => {
                // Widespread damage to all systems
                self.damage_state.infrastructure_damage += effective_damage * 0.8;
                self.damage_state.industrial_damage += effective_damage * 0.7;
                self.damage_state.residential_damage += effective_damage * 0.9;
                self.base.condition *= 1.0 - (effective_damage * 0.6);
            },
        }
        
        self.update_population();
    }

    fn calculate_casualty_rate(&self) -> f32 {
        let base_rate = 0.001; // Base 0.1% casualty rate per update
        let shelter_modifier = 1.0 - self.defenses.civilian_shelter_level;
        let damage_modifier = self.damage_state.residential_damage;
        
        base_rate * shelter_modifier * (1.0 + damage_modifier)
    }

    fn calculate_evacuation_rate(&self) -> f32 {
        let base_rate = 0.005; // Base 0.5% evacuation rate per update
        let morale_modifier = 1.0 - self.civilian_morale;
        let damage_modifier = (
            self.damage_state.infrastructure_damage +
            self.damage_state.residential_damage +
            (1.0 - self.damage_state.power_grid_status) +
            (1.0 - self.damage_state.water_system_status)
        ) / 4.0;
        
        base_rate * (1.0 + morale_modifier) * (1.0 + damage_modifier)
    }

    fn update_civilian_morale(&mut self) {
        let population_factor = self.population as f32 / self.base_population as f32;
        let infrastructure_factor = 1.0 - self.damage_state.infrastructure_damage;
        let services_factor = (
            self.damage_state.power_grid_status +
            self.damage_state.water_system_status
        ) / 2.0;
        
        self.civilian_morale = (
            0.3 * population_factor +
            0.3 * infrastructure_factor +
            0.4 * services_factor
        ).max(0.0).min(1.0);
    }
}

impl Targetable for City {
    fn get_position(&self) -> (f64, f64) {
        (self.base.x, self.base.y)
    }

    fn get_hardness(&self) -> f32 {
        let base_hardness = 0.4;
        let fortification_bonus = self.defenses.fortification_level * 0.3;
        let underground_bonus = if self.defenses.has_underground_facilities { 0.2 } else { 0.0 };
        
        (base_hardness + fortification_bonus + underground_bonus) * self.base.condition
    }

    fn receive_damage(&mut self, amount: f32) {
        self.handle_bombing_attack(amount, DamageType::Complete);
    }
}

impl SupplyNode for City {
    fn get_supply_capacity(&self) -> i32 {
        self.calculate_supply_generation()
    }

    fn get_current_supply(&self) -> i32 {
        let capacity = self.get_supply_capacity();
        (capacity as f32 * (1.0 - self.damage_state.infrastructure_damage)) as i32
    }

    fn add_supply(&mut self, amount: i32) -> Result<(), GameError> {
        if self.damage_state.infrastructure_damage > 0.8 {
            return Err(GameError::InsufficientSupplies);
        }
        self.base.add_supply(amount)
    }

    fn remove_supply(&mut self, amount: i32) -> Result<(), GameError> {
        self.base.remove_supply(amount)
    }
}

impl StorageCapable for City {
    fn get_storage_capacity(&self) -> i32 {
        self.base.storage
    }

    fn get_available_storage(&self) -> i32 {
        let base_storage = self.base.storage as f32;
        let damage_modifier = 1.0 - self.damage_state.infrastructure_damage;
        (base_storage * damage_modifier) as i32
    }

    fn store_resources(&mut self, amount: i32) -> Result<(), GameError> {
        if amount <= self.get_available_storage() {
            self.base.storage += amount;
            Ok(())
        } else {
            Err(GameError::InsufficientSupplies)
        }
    }

    fn retrieve_resources(&mut self, amount: i32) -> Result<(), GameError> {
        if self.get_current_supply() >= amount {
            self.base.storage -= amount;
            Ok(())
        } else {
            Err(GameError::InsufficientSupplies)
        }
    }
}