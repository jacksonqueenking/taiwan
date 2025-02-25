use std::collections::{HashMap, HashSet};
use crate::infrastructure::{Hub, Targetable, SupplyNode, StorageCapable, DamageType};
use crate::units::{
    Ship, AirUnit, Arsenal,
    Movable, Heavy, UnitStatus,
    MilitaryUnit,
};
use crate::game::GameError;

#[derive(Debug, Clone)]
pub struct Port {
    pub base: Hub,
    pub docking_capacity: i32,      // Number of ships that can dock
    pub repair_capacity: f32,       // Repair points per day
    pub loading_capacity: i32,      // Tonnes per day
    pub depth: f32,                 // Harbor depth in meters
    pub is_blockaded: bool,
    pub docked_ships: HashSet<String>, // IDs of currently docked ships
    pub facilities: PortFacilities,
}

#[derive(Debug, Clone)]
pub struct AirBase {
    pub base: Hub,
    pub runway_condition: f32,      // 0.0 to 1.0
    pub hangar_space: i32,          // Number of aircraft that can be housed
    pub maintenance_capacity: f32,   // Maintenance points per day
    pub air_defense_level: f32,     // 0.0 to 1.0
    pub stationed_aircraft: HashSet<String>, // IDs of stationed aircraft
    pub facilities: AirbaseFacilities,
}

#[derive(Debug, Clone)]
pub struct PortFacilities {
    pub has_drydock: bool,
    pub has_crane: bool,
    pub has_fuel_depot: bool,
    pub has_ammunition_depot: bool,
    pub has_radar: bool,
    pub has_missile_defense: bool,
    defense_strength: f32,          // Coastal defense strength
    pub max_ship_size: i32,         // Maximum ship tonnage that can be handled
}

#[derive(Debug, Clone)]
pub struct AirbaseFacilities {
    pub runway_length: i32,         // in meters
    pub has_control_tower: bool,
    pub has_radar: bool,
    pub has_sam_sites: bool,
    pub has_bunkers: bool,
    pub has_fuel_depot: bool,
    pub has_munitions_depot: bool,
    air_defense_strength: f32,      // Air defense capability
    pub night_operations: bool,     // Capability for night operations
}

impl Port {
    pub fn new(
        name: String,
        x: f64,
        y: f64,
        storage: i32,
        controller: String,
        docking_capacity: i32,
        depth: f32,
    ) -> Self {
        Port {
            base: Hub::new(name, x, y, storage, controller),
            docking_capacity,
            repair_capacity: 10.0,   // Base repair rate
            loading_capacity: 1000,  // Base loading capacity
            depth,
            is_blockaded: false,
            docked_ships: HashSet::new(),
            facilities: PortFacilities {
                has_drydock: false,
                has_crane: true,
                has_fuel_depot: true,
                has_ammunition_depot: true,
                has_radar: true,
                has_missile_defense: false,
                defense_strength: 1.0,
                max_ship_size: 50000,
            },
        }
    }

    pub fn can_dock_ship(&self, ship: &Ship) -> bool {
        if self.is_blockaded || self.base.condition < 0.3 {
            return false;
        }

        // Check basic requirements
        if self.docked_ships.len() >= self.docking_capacity as usize {
            return false;
        }

        // Check depth requirements (assuming ship has draft)
        if ship.get_position().depth.unwrap_or(0.0) > self.depth {
            return false;
        }

        // Check if ship is too large for port
        if ship.weight() > self.facilities.max_ship_size as f32 {
            return false;
        }

        true
    }

    pub fn dock_ship(&mut self, ship_id: String) -> Result<(), GameError> {
        if self.docked_ships.len() >= self.docking_capacity as usize {
            return Err(GameError::InsufficientSupplies);
        }
        self.docked_ships.insert(ship_id);
        Ok(())
    }

    pub fn undock_ship(&mut self, ship_id: &str) {
        self.docked_ships.remove(ship_id);
    }

    pub fn repair_ship(&mut self, ship: &mut Ship) -> f32 {
        if !self.docked_ships.contains(&ship.name) {
            return 0.0;
        }

        let repair_amount = self.repair_capacity * self.base.condition;
        ship.hull_integrity = (ship.hull_integrity + repair_amount).min(1.0);
        repair_amount
    }

    pub fn calculate_defense_strength(&self) -> f32 {
        let base_defense = self.facilities.defense_strength;
        let condition_modifier = self.base.condition;
        let blockade_modifier = if self.is_blockaded { 0.5 } else { 1.0 };

        base_defense * condition_modifier * blockade_modifier
    }
}

impl AirBase {
    pub fn new(
        name: String,
        x: f64,
        y: f64,
        storage: i32,
        controller: String,
        hangar_space: i32,
    ) -> Self {
        AirBase {
            base: Hub::new(name, x, y, storage, controller),
            runway_condition: 1.0,
            hangar_space,
            maintenance_capacity: 5.0,
            air_defense_level: 1.0,
            stationed_aircraft: HashSet::new(),
            facilities: AirbaseFacilities {
                runway_length: 3000,
                has_control_tower: true,
                has_radar: true,
                has_sam_sites: true,
                has_bunkers: true,
                has_fuel_depot: true,
                has_munitions_depot: true,
                air_defense_strength: 1.0,
                night_operations: true,
            },
        }
    }

    pub fn can_base_aircraft(&self, aircraft: &AirUnit) -> bool {
        if self.runway_condition < 0.3 {
            return false;
        }

        // Check basic requirements
        if self.stationed_aircraft.len() >= self.hangar_space as usize {
            return false;
        }

        // Check runway length requirements (placeholder logic)
        let required_runway = match aircraft {
            _ if aircraft.weight() > 50000.0 => 3000,
            _ if aircraft.weight() > 20000.0 => 2500,
            _ => 2000,
        };

        if required_runway > self.facilities.runway_length {
            return false;
        }

        true
    }

    pub fn station_aircraft(&mut self, aircraft_id: String) -> Result<(), GameError> {
        if self.stationed_aircraft.len() >= self.hangar_space as usize {
            return Err(GameError::InsufficientSupplies);
        }
        self.stationed_aircraft.insert(aircraft_id);
        Ok(())
    }

    pub fn remove_aircraft(&mut self, aircraft_id: &str) {
        self.stationed_aircraft.remove(aircraft_id);
    }

    pub fn maintain_aircraft(&mut self, aircraft: &mut AirUnit) -> f32 {
        if !self.stationed_aircraft.contains(&aircraft.name) {
            return 0.0;
        }

        let maintenance_amount = self.maintenance_capacity * self.runway_condition;
        // Apply maintenance effects to aircraft
        // (Specific implementation would depend on AirUnit maintenance system)
        maintenance_amount
    }

    pub fn calculate_air_defense(&self) -> f32 {
        let base_defense = self.facilities.air_defense_strength;
        let condition_modifier = self.base.condition;
        let sam_modifier = if self.facilities.has_sam_sites { 1.5 } else { 1.0 };
        let radar_modifier = if self.facilities.has_radar { 1.2 } else { 1.0 };

        base_defense * condition_modifier * sam_modifier * radar_modifier
    }
}

impl Targetable for Port {
    fn get_position(&self) -> (f64, f64) {
        (self.base.x, self.base.y)
    }

    fn get_hardness(&self) -> f32 {
        // Ports are generally harder targets than basic infrastructure
        0.7 * self.base.condition
    }

    fn receive_damage(&mut self, amount: f32) {
        self.base.damage(amount);
        // Additional damage effects on port-specific systems
        self.facilities.defense_strength *= 1.0 - (amount * 0.5);
        if amount > 0.5 {
            self.facilities.has_radar = false;
            self.facilities.has_missile_defense = false;
        }
    }
}

impl Targetable for AirBase {
    fn get_position(&self) -> (f64, f64) {
        (self.base.x, self.base.y)
    }

    fn get_hardness(&self) -> f32 {
        // Airbases can be hardened with bunkers
        if self.facilities.has_bunkers {
            0.8 * self.base.condition
        } else {
            0.6 * self.base.condition
        }
    }

    fn receive_damage(&mut self, amount: f32) {
        self.base.damage(amount);
        self.runway_condition *= 1.0 - amount;
        // Additional damage effects on airbase-specific systems
        if amount > 0.5 {
            self.facilities.has_radar = false;
            self.facilities.has_sam_sites = false;
        }
    }
}

impl SupplyNode for Port {
    fn get_supply_capacity(&self) -> i32 {
        (self.base.storage as f32 * self.base.condition) as i32
    }

    fn get_current_supply(&self) -> i32 {
        self.base.get_current_supply()
    }

    fn add_supply(&mut self, amount: i32) -> Result<(), GameError> {
        if self.is_blockaded {
            return Err(GameError::InsufficientSupplies);
        }
        self.base.add_supply(amount)
    }

    fn remove_supply(&mut self, amount: i32) -> Result<(), GameError> {
        self.base.remove_supply(amount)
    }
}

impl SupplyNode for AirBase {
    fn get_supply_capacity(&self) -> i32 {
        (self.base.storage as f32 * self.base.condition) as i32
    }

    fn get_current_supply(&self) -> i32 {
        self.base.get_current_supply()
    }

    fn add_supply(&mut self, amount: i32) -> Result<(), GameError> {
        if self.runway_condition < 0.3 {
            return Err(GameError::InsufficientSupplies);
        }
        self.base.add_supply(amount)
    }

    fn remove_supply(&mut self, amount: i32) -> Result<(), GameError> {
        self.base.remove_supply(amount)
    }
}

impl StorageCapable for Port {
    fn get_storage_capacity(&self) -> i32 {
        self.base.storage
    }

    fn get_available_storage(&self) -> i32 {
        (self.base.storage as f32 * (1.0 - self.base.condition)) as i32
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
        if self.base.get_current_supply() >= amount {
            self.base.storage -= amount;
            Ok(())
        } else {
            Err(GameError::InsufficientSupplies)
        }
    }
}

impl StorageCapable for AirBase {
    fn get_storage_capacity(&self) -> i32 {
        self.base.storage
    }

    fn get_available_storage(&self) -> i32 {
        (self.base.storage as f32 * (1.0 - self.base.condition)) as i32
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
        if self.base.get_current_supply() >= amount {
            self.base.storage -= amount;
            Ok(())
        } else {
            Err(GameError::InsufficientSupplies)
        }
    }
}