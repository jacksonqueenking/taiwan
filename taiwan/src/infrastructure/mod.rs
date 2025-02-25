use std::collections::HashMap;
use crate::map::tiles::HexTile;
use crate::units::{Arsenal, Movable, Heavy};
use crate::game::GameError;

// Re-export infrastructure components
pub mod cities;
pub mod ports;
pub mod roads;

pub use cities::{City, Cities};
pub use ports::{Port, AirBase};
pub use roads::{Road, RoadType, Roads};

// Base infrastructure type
#[derive(Debug, Clone)]
pub struct Hub {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub storage: i32,  // Storage capacity in tonnes
    pub condition: f32, // 0.0 to 1.0, representing structural integrity
    pub controller: String, // Faction controlling this hub
}

impl Hub {
    pub fn new(name: String, x: f64, y: f64, storage: i32, controller: String) -> Self {
        Hub {
            name,
            x,
            y,
            storage,
            condition: 1.0,
            controller,
        }
    }

    pub fn damage(&mut self, amount: f32) {
        self.condition = (self.condition - amount).max(0.0);
        self.storage = (self.storage as f32 * self.condition) as i32;
    }

    pub fn repair(&mut self, amount: f32) {
        self.condition = (self.condition + amount).min(1.0);
    }
}

// Infrastructure network management
#[derive(Debug)]
pub struct InfrastructureNetwork {
    pub cities: Cities,
    pub ports: HashMap<String, Port>,
    pub airbases: HashMap<String, AirBase>,
    pub roads: Roads,
    supply_cache: HashMap<String, i32>, // Cached supply levels
}

impl InfrastructureNetwork {
    pub fn new(cities: Cities, roads: Roads) -> Self {
        InfrastructureNetwork {
            cities,
            ports: HashMap::new(),
            airbases: HashMap::new(),
            roads,
            supply_cache: HashMap::new(),
        }
    }

    pub fn add_port(&mut self, name: String, port: Port) {
        self.ports.insert(name, port);
    }

    pub fn add_airbase(&mut self, name: String, airbase: AirBase) {
        self.airbases.insert(name, airbase);
    }

    pub fn get_supply_level(&self, location: &str) -> i32 {
        *self.supply_cache.get(location).unwrap_or(&0)
    }

    pub fn update_supply_network(&mut self) {
        self.supply_cache.clear();
        // Calculate supply levels based on infrastructure conditions
        for (city_name, city) in &self.cities {
            let base_supply = city.base.storage as f32 * city.base.condition;
            self.supply_cache.insert(city_name.clone(), base_supply as i32);
        }
    }

    pub fn calculate_path(&self, start: &str, end: &str) -> Option<Vec<String>> {
        // Implement pathfinding between infrastructure points
        // Returns sequence of infrastructure IDs representing the path
        None // Placeholder
    }
}

// Traits for infrastructure elements
pub trait Targetable {
    fn get_position(&self) -> (f64, f64);
    fn get_hardness(&self) -> f32; // Resistance to damage
    fn receive_damage(&mut self, amount: f32);
}

pub trait SupplyNode {
    fn get_supply_capacity(&self) -> i32;
    fn get_current_supply(&self) -> i32;
    fn add_supply(&mut self, amount: i32) -> Result<(), GameError>;
    fn remove_supply(&mut self, amount: i32) -> Result<(), GameError>;
}

pub trait StorageCapable {
    fn get_storage_capacity(&self) -> i32;
    fn get_available_storage(&self) -> i32;
    fn store_resources(&mut self, amount: i32) -> Result<(), GameError>;
    fn retrieve_resources(&mut self, amount: i32) -> Result<(), GameError>;
}

// Implementation for Hub
impl Targetable for Hub {
    fn get_position(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    fn get_hardness(&self) -> f32 {
        0.5 // Base hardness value for infrastructure
    }

    fn receive_damage(&mut self, amount: f32) {
        self.damage(amount);
    }
}

impl SupplyNode for Hub {
    fn get_supply_capacity(&self) -> i32 {
        self.storage
    }

    fn get_current_supply(&self) -> i32 {
        (self.storage as f32 * self.condition) as i32
    }

    fn add_supply(&mut self, amount: i32) -> Result<(), GameError> {
        if amount + self.get_current_supply() <= self.storage {
            self.storage += amount;
            Ok(())
        } else {
            Err(GameError::InsufficientSupplies)
        }
    }

    fn remove_supply(&mut self, amount: i32) -> Result<(), GameError> {
        if self.get_current_supply() >= amount {
            self.storage -= amount;
            Ok(())
        } else {
            Err(GameError::InsufficientSupplies)
        }
    }
}

// Supply route calculation
#[derive(Debug, Clone)]
pub struct SupplyRoute {
    pub path: Vec<String>, // Sequence of infrastructure IDs
    pub capacity: i32,     // Maximum supply throughput
    pub reliability: f32,  // 0.0 to 1.0
    pub distance: f32,     // Total route distance
}

impl SupplyRoute {
    pub fn new(path: Vec<String>, capacity: i32, reliability: f32, distance: f32) -> Self {
        SupplyRoute {
            path,
            capacity,
            reliability,
            distance,
        }
    }

    pub fn get_effective_capacity(&self) -> i32 {
        (self.capacity as f32 * self.reliability) as i32
    }
}

// Infrastructure damage types
#[derive(Debug, Clone, Copy)]
pub enum DamageType {
    Structural,
    Storage,
    Logistics,
    Complete,
}

// Helper functions for infrastructure management
pub fn calculate_supply_throughput(route: &SupplyRoute, infrastructure: &InfrastructureNetwork) -> i32 {
    let mut min_throughput = i32::MAX;
    
    for (i, current) in route.path.iter().enumerate() {
        if i == 0 { continue; } // Skip start node
        
        let prev = &route.path[i - 1];
        if let Some(road) = infrastructure.roads.iter().find(|r| 
            (r.start_city.to_string() == *prev && r.end_city.to_string() == *current) ||
            (r.start_city.to_string() == *current && r.end_city.to_string() == *prev)
        ) {
            let road_capacity = (road.capacity() * road.condition) as i32;
            min_throughput = min_throughput.min(road_capacity);
        }
    }
    
    min_throughput
}