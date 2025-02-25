//! Taiwan Strait Conflict - A strategic military simulation
//! 
//! This crate implements a turn-based strategy game simulating modern warfare
//! in the Taiwan Strait region, with combined arms warfare across land, sea, and air.

use piston_window::*;

// Public module declarations
pub mod map;
pub mod units;
pub mod game;
pub mod infrastructure;

// Re-export main game components
pub use game::{
    Game, GameState, GameActions, GameQueries, GameEnvironment,
    GamePhase, Weather, TimeOfDay,
    GameError, GameResult, GameEvent,
    GameConfig, VictoryConditions,
};

pub use map::{
    tiles::{HexTile, TileType, TileParameters},
    render::{MapRenderer, Drawable},
    terrain::{TerrainBonus, TerrainRules},
};

pub use units::{
    // Land units
    LandUnit, LandUnitType,
    
    // Naval units
    Ship, SAG, AircraftCarrier, Submarine, Cruiser, 
    Destroyer, CivilianRoRoShip, AmphibiousLandingShip,
    
    // Air units
    AirUnit, FighterSquadron, BomberSquadron,
    FighterGeneration, BomberType,
    
    // Common types
    UnitStatus, UnitStats, Arsenal, Position,
    MissileType, Missile,
    
    // Core traits
    MilitaryUnit, UnitError, UnitResult,
    Movable, Heavy, Damageable, Attritable, 
    Bombable, Flyable, Sailable,
};

pub use infrastructure::{
    // Core types
    Hub, InfrastructureNetwork,
    
    // Cities
    City, Cities,
    
    // Military facilities
    Port, AirBase,
    
    // Transportation
    Road, RoadType, Roads,
    
    // Infrastructure traits
    Targetable, SupplyNode, StorageCapable,
    
    // Supply system
    SupplyRoute, DamageType,
};

// Window settings and constants
pub const WINDOW_WIDTH: u32 = 640;
pub const WINDOW_HEIGHT: u32 = 480;
pub const DEFAULT_TURN_LIMIT: u32 = 30;
pub const DEFAULT_SUPPLY_RANGE: f32 = 50.0; // In kilometers
pub const DEFAULT_CITY_CONTROL_THRESHOLD: f32 = 0.7;
pub const DEFAULT_CASUALTIES_THRESHOLD: f32 = 0.5;

/// Creates a new game instance with default configuration
pub fn new_game() -> Game {
    Game::new()
}

/// Creates a new game instance with custom configuration
pub fn new_game_with_config(config: GameConfig) -> Game {
    Game::new_with_config(config)
}

// Internal helper functions
mod helpers {
    use super::*;
    
    pub(crate) fn calculate_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
    }

    pub(crate) fn is_valid_coordinate(x: f64, y: f64) -> bool {
        x >= 0.0 && x < WINDOW_WIDTH as f64 && 
        y >= 0.0 && y < WINDOW_HEIGHT as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_creation() {
        let game = new_game();
        assert_eq!(game.get_turn_number(), 1);
        assert_eq!(game.get_current_phase(), GamePhase::Planning);
    }

    #[test]
    fn test_custom_game_config() {
        let config = GameConfig {
            map_parameters: TileParameters::default(),
            starting_cities: Cities::default(),
            starting_roads: Roads::default(),
            initial_weather: Weather::Clear,
            victory_conditions: VictoryConditions {
                turn_limit: Some(20),
                key_cities: vec!["Taipei".to_string(), "Kaohsiung".to_string()],
                required_city_control: 0.8,
                enemy_casualties_threshold: 0.6,
            },
        };
        let game = new_game_with_config(config);
        assert_eq!(game.get_turn_number(), 1);
    }

    #[test]
    fn test_distance_calculation() {
        let distance = helpers::calculate_distance(0.0, 0.0, 3.0, 4.0);
        assert_eq!(distance, 5.0);
    }
}