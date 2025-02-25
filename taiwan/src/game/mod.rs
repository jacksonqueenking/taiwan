use std::collections::HashMap;
use crate::map::{
    tiles::{HexTile, TileParameters},
    render::MapRenderer,
    terrain::TerrainRules,
};
use crate::units::{
    LandUnit, Ship, AirUnit,
    UnitStatus, UnitStats, Arsenal,
    MilitaryUnit, UnitError, UnitResult
};
use crate::infrastructure::{
    City, Cities, Road, Roads,
    Port, AirBase
};

pub mod state;
pub mod combat;
pub mod turns;

pub use state::Game;
pub use combat::resolve_combat;
pub use turns::Phase;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamePhase {
    Planning,
    Movement,
    Combat,
    Supply,
    EndTurn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Weather {
    Clear,
    Rain,
    Storm,
    Fog,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeOfDay {
    Dawn,
    Day,
    Dusk,
    Night,
}

#[derive(Debug)]
pub enum GameError {
    InvalidMove,
    InvalidAttack,
    InsufficientSupplies,
    UnitNotFound,
    CityNotFound,
    RoadNotFound,
    PhaseError,
    UnitError(UnitError),
}

pub type GameResult<T> = Result<T, GameError>;

pub trait GameState {
    fn new() -> Self;
    fn next_turn(&mut self) -> GameResult<()>;
    fn is_over(&self) -> bool;
    fn get_winner(&self) -> Option<String>;
    fn get_current_phase(&self) -> GamePhase;
    fn get_turn_number(&self) -> u32;
}

pub trait GameActions {
    fn move_unit(&mut self, unit_id: usize, x: f64, y: f64) -> GameResult<()>;
    fn attack_unit(&mut self, attacker_id: usize, defender_id: usize) -> GameResult<()>;
    fn repair_unit(&mut self, unit_id: usize) -> GameResult<()>;
    fn resupply_unit(&mut self, unit_id: usize) -> GameResult<()>;
    fn bomb_road(&mut self, road_id: usize, damage: f64) -> GameResult<()>;
    fn mine_road(&mut self, road_id: usize) -> GameResult<()>;
}

pub trait GameQueries {
    fn get_unit(&self, unit_id: usize) -> Option<&dyn MilitaryUnit>;
    fn get_city(&self, city_id: usize) -> Option<&City>;
    fn get_road(&self, road_id: usize) -> Option<&Road>;
    fn get_units_in_range(&self, x: f64, y: f64, range: f64) -> Vec<usize>;
    fn get_supply_level(&self, unit_id: usize) -> GameResult<f32>;
    fn can_move_to(&self, unit_id: usize, x: f64, y: f64) -> bool;
    fn can_attack(&self, attacker_id: usize, defender_id: usize) -> bool;
}

pub trait GameEnvironment {
    fn get_weather(&self) -> Weather;
    fn get_time_of_day(&self) -> TimeOfDay;
    fn get_terrain_at(&self, x: f64, y: f64) -> Option<&HexTile>;
    fn is_position_visible(&self, x: f64, y: f64) -> bool;
}

pub struct GameConfig {
    pub map_parameters: TileParameters,
    pub starting_cities: Cities,
    pub starting_roads: Roads,
    pub initial_weather: Weather,
    pub victory_conditions: VictoryConditions,
}

pub struct VictoryConditions {
    pub turn_limit: Option<u32>,
    pub key_cities: Vec<String>,
    pub required_city_control: f32,
    pub enemy_casualties_threshold: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            map_parameters: TileParameters::default(),
            starting_cities: Cities::default(),
            starting_roads: Roads::default(),
            initial_weather: Weather::Clear,
            victory_conditions: VictoryConditions {
                turn_limit: Some(30),
                key_cities: vec!["Taipei".to_string(), "Kaohsiung".to_string()],
                required_city_control: 0.7,
                enemy_casualties_threshold: 0.5,
            },
        }
    }
}

pub trait SaveLoad {
    fn save_game(&self, filename: &str) -> std::io::Result<()>;
    fn load_game(filename: &str) -> std::io::Result<Self> where Self: Sized;
}

// Events system for game state changes
#[derive(Debug, Clone)]
pub enum GameEvent {
    UnitMoved { unit_id: usize, x: f64, y: f64 },
    UnitAttacked { attacker_id: usize, defender_id: usize, damage: f32 },
    UnitDestroyed { unit_id: usize },
    CityControleChanged { city_id: usize, new_controller: String },
    RoadDamaged { road_id: usize, damage: f64 },
    SupplyLineDisrupted { from_id: usize, to_id: usize },
    WeatherChanged { new_weather: Weather },
    PhaseChanged { new_phase: GamePhase },
    TurnCompleted { turn_number: u32 },
}

pub type EventListener = Box<dyn Fn(&GameEvent)>;

// Statistics tracking
pub struct GameStatistics {
    pub turns_played: u32,
    pub units_lost: HashMap<String, u32>,
    pub damage_dealt: HashMap<String, f32>,
    pub cities_captured: Vec<String>,
    pub supply_consumed: f32,
    pub combat_results: Vec<CombatResult>,
}

#[derive(Debug, Clone)]
pub struct CombatResult {
    pub turn: u32,
    pub attacker: String,
    pub defender: String,
    pub damage_dealt: f32,
    pub units_lost: u32,
}