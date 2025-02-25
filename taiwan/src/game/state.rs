use std::collections::{HashMap, HashSet};
use crate::map::{
    tiles::{HexTile, TileParameters},
    render::MapRenderer,
    terrain::TerrainRules,
};
use crate::units::{
    LandUnit, Ship, AirUnit, UnitStatus, Arsenal,
    MilitaryUnit, UnitError, UnitResult,
};
use crate::infrastructure::{
    City, Cities, Road, Roads,
    Port, AirBase,
};
use super::{
    GamePhase, Weather, TimeOfDay,
    GameError, GameResult,
    GameState, GameActions, GameQueries, GameEnvironment,
    GameConfig, VictoryConditions, GameEvent, GameStatistics, CombatResult,
    EventListener,
};

pub struct Game {
    // Core game state
    turn: u32,
    phase: GamePhase,
    weather: Weather,
    time_of_day: TimeOfDay,
    
    // Map and terrain
    tiles: Vec<HexTile>,
    terrain_rules: TerrainRules,
    
    // Units and infrastructure
    land_units: HashMap<usize, LandUnit>,
    ships: HashMap<usize, Ship>,
    air_units: HashMap<usize, AirUnit>,
    cities: HashMap<usize, City>,
    roads: HashMap<usize, Road>,
    ports: HashMap<usize, Port>,
    airbases: HashMap<usize, AirBase>,
    
    // Control and visibility
    controlled_territories: HashMap<String, HashSet<usize>>, // Faction -> Set of tile IDs
    visibility_map: Vec<bool>, // Parallel to tiles vec, tracks fog of war
    
    // Supply and logistics
    supply_network: HashMap<usize, Vec<usize>>, // City ID -> Connected unit IDs
    supply_cache: HashMap<usize, f32>, // Unit ID -> Current supply level
    
    // Event system
    event_listeners: Vec<EventListener>,
    event_history: Vec<GameEvent>,
    
    // Statistics
    statistics: GameStatistics,
    
    // Configuration
    config: GameConfig,
}

impl Game {
    fn generate_map(&self, params: &TileParameters) -> Vec<HexTile> {
        params.generate()
    }

    fn update_supply_network(&mut self) {
        self.supply_network.clear();
        
        // Build graph of connected cities and units
        for (city_id, city) in &self.cities {
            let mut connected_units = Vec::new();
            
            // Find units within supply range and with valid supply lines
            for (unit_id, unit) in self.land_units.iter()
                .chain(self.ships.iter().map(|(id, ship)| (id, ship as &dyn MilitaryUnit)))
                .chain(self.air_units.iter().map(|(id, air)| (id, air as &dyn MilitaryUnit)))
            {
                if self.can_supply_unit(*city_id, *unit_id) {
                    connected_units.push(*unit_id);
                }
            }
            
            self.supply_network.insert(*city_id, connected_units);
        }
    }

    fn can_supply_unit(&self, city_id: usize, unit_id: usize) -> bool {
        if let Some(city) = self.cities.get(&city_id) {
            if let Some(unit) = self.get_unit(unit_id) {
                let distance = calculate_distance(
                    city.base.x, city.base.y,
                    unit.get_position().x, unit.get_position().y
                );
                
                // Check if within supply range
                if distance > self.config.max_supply_range {
                    return false;
                }
                
                // Check if supply line is blocked
                return self.is_supply_line_clear(city_id, unit_id);
            }
        }
        false
    }

    fn is_supply_line_clear(&self, city_id: usize, unit_id: usize) -> bool {
        if let (Some(city), Some(unit)) = (self.cities.get(&city_id), self.get_unit(unit_id)) {
            // Check for blocked or mined roads along supply route
            for road in self.get_roads_between(
                city.base.x, city.base.y,
                unit.get_position().x, unit.get_position().y
            ) {
                if road.is_mined || road.condition < 0.2 {
                    return false;
                }
            }
            
            // Check for enemy units blocking supply line
            for enemy_unit in self.get_enemy_units_between(
                city.base.x, city.base.y,
                unit.get_position().x, unit.get_position().y
            ) {
                if self.can_block_supply(enemy_unit) {
                    return false;
                }
            }
            
            true
        } else {
            false
        }
    }

    fn update_visibility(&mut self) {
        self.visibility_map = vec![false; self.tiles.len()];
        
        // Update visibility based on unit positions and vision ranges
        for (unit_id, _) in self.land_units.iter()
            .chain(self.ships.iter())
            .chain(self.air_units.iter())
        {
            if let Some(unit) = self.get_unit(*unit_id) {
                let vision_range = self.calculate_vision_range(unit);
                let visible_tiles = self.get_tiles_in_range(
                    unit.get_position().x,
                    unit.get_position().y,
                    vision_range
                );
                
                for tile_id in visible_tiles {
                    self.visibility_map[tile_id] = true;
                }
            }
        }
    }

    fn calculate_vision_range(&self, unit: &dyn MilitaryUnit) -> f64 {
        let base_range = unit.get_stats().vision_range;
        
        // Apply weather modifications
        let weather_modifier = match self.weather {
            Weather::Clear => 1.0,
            Weather::Rain => 0.7,
            Weather::Storm => 0.4,
            Weather::Fog => 0.3,
        };
        
        // Apply time of day modifications
        let time_modifier = match self.time_of_day {
            TimeOfDay::Day => 1.0,
            TimeOfDay::Dawn | TimeOfDay::Dusk => 0.7,
            TimeOfDay::Night => 0.4,
        };
        
        base_range * weather_modifier * time_modifier
    }

    fn emit_event(&mut self, event: GameEvent) {
        // Store event in history
        self.event_history.push(event.clone());
        
        // Notify all listeners
        for listener in &self.event_listeners {
            listener(&event);
        }
    }
}

impl GameState for Game {
    fn new() -> Self {
        let config = GameConfig::default();
        let tiles = config.map_parameters.generate();
        
        Game {
            turn: 1,
            phase: GamePhase::Planning,
            weather: config.initial_weather,
            time_of_day: TimeOfDay::Dawn,
            tiles,
            terrain_rules: TerrainRules::new(),
            land_units: HashMap::new(),
            ships: HashMap::new(),
            air_units: HashMap::new(),
            cities: HashMap::new(),
            roads: HashMap::new(),
            ports: HashMap::new(),
            airbases: HashMap::new(),
            controlled_territories: HashMap::new(),
            visibility_map: Vec::new(),
            supply_network: HashMap::new(),
            supply_cache: HashMap::new(),
            event_listeners: Vec::new(),
            event_history: Vec::new(),
            statistics: GameStatistics {
                turns_played: 0,
                units_lost: HashMap::new(),
                damage_dealt: HashMap::new(),
                cities_captured: Vec::new(),
                supply_consumed: 0.0,
                combat_results: Vec::new(),
            },
            config,
        }
    }

    fn next_turn(&mut self) -> GameResult<()> {
        // Process end of turn effects
        self.update_supply_network();
        self.update_visibility();
        
        // Update time of day
        self.time_of_day = match self.time_of_day {
            TimeOfDay::Dawn => TimeOfDay::Day,
            TimeOfDay::Day => TimeOfDay::Dusk,
            TimeOfDay::Dusk => TimeOfDay::Night,
            TimeOfDay::Night => TimeOfDay::Dawn,
        };
        
        // Randomly update weather with some persistence
        if rand::random::<f32>() < 0.3 {
            self.weather = match rand::random::<f32>() {
                x if x < 0.4 => Weather::Clear,
                x if x < 0.7 => Weather::Rain,
                x if x < 0.9 => Weather::Storm,
                _ => Weather::Fog,
            };
        }
        
        self.turn += 1;
        self.phase = GamePhase::Planning;
        
        self.emit_event(GameEvent::TurnCompleted { turn_number: self.turn });
        
        Ok(())
    }

    fn is_over(&self) -> bool {
        // Check turn limit
        if let Some(limit) = self.config.victory_conditions.turn_limit {
            if self.turn >= limit {
                return true;
            }
        }
        
        // Check city control victory condition
        for faction in self.controlled_territories.keys() {
            let controlled_key_cities = self.config.victory_conditions.key_cities
                .iter()
                .filter(|city_name| {
                    self.cities.values()
                        .any(|city| &city.base.name == *city_name && city.controller == *faction)
                })
                .count();
                
            if (controlled_key_cities as f32 / self.config.victory_conditions.key_cities.len() as f32)
                >= self.config.victory_conditions.required_city_control {
                return true;
            }
        }
        
        // Check casualties victory condition
        for (faction, casualties) in &self.statistics.units_lost {
            let total_units = self.count_faction_units(faction);
            if *casualties as f32 / (total_units + casualties) as f32
                >= self.config.victory_conditions.enemy_casualties_threshold {
                return true;
            }
        }
        
        false
    }

    fn get_winner(&self) -> Option<String> {
        if !self.is_over() {
            return None;
        }

        // Check city control victory condition
        for faction in self.controlled_territories.keys() {
            let controlled_key_cities = self.config.victory_conditions.key_cities
                .iter()
                .filter(|city_name| {
                    self.cities.values()
                        .any(|city| &city.base.name == *city_name && city.controller == *faction)
                })
                .count();
                
            // If faction controls required percentage of key cities, they win
            if (controlled_key_cities as f32 / self.config.victory_conditions.key_cities.len() as f32)
                >= self.config.victory_conditions.required_city_control {
                return Some(faction.clone());
            }
        }

        // Check casualties victory condition
        for faction in self.controlled_territories.keys() {
            // Look at casualties of other factions
            for (enemy_faction, casualties) in &self.statistics.units_lost {
                if enemy_faction == faction {
                    continue; // Skip checking own casualties
                }
                
                let enemy_total_units = self.count_faction_units(enemy_faction);
                let casualty_rate = *casualties as f32 / (enemy_total_units + casualties) as f32;
                
                // If any enemy has suffered casualties above threshold, this faction wins
                if casualty_rate >= self.config.victory_conditions.enemy_casualties_threshold {
                    return Some(faction.clone());
                }
            }
        }

        None
    }
  
    fn get_current_phase(&self) -> GamePhase {
        self.phase
    }

    fn get_turn_number(&self) -> u32 {
        self.turn
    }
}

impl GameActions for Game {
    fn move_unit(&mut self, unit_id: usize, x: f64, y: f64) -> GameResult<()> {
        // Check if we're in the movement phase
        if self.phase != GamePhase::Movement {
            return Err(GameError::PhaseError);
        }

        // Validate movement
        if !self.can_move_to(unit_id, x, y) {
            return Err(GameError::InvalidMove);
        }

        // Get and move the unit
        if let Some(unit) = self.get_unit_mut(unit_id) {
            let current_pos = unit.get_position().clone();
            
            // Calculate supply cost for movement
            let distance = calculate_distance(current_pos.x, current_pos.y, x, y);
            let supply_cost = self.calculate_movement_supply_cost(unit, distance);
            
            // Check if unit has enough supply
            if unit.get_stats().supply_level < supply_cost {
                return Err(GameError::InsufficientSupplies);
            }
            
            // Execute movement
            unit.move_to(x, y);
            self.consume_movement_supply(unit_id, supply_cost)?;
            
            // Emit movement event
            self.emit_event(GameEvent::UnitMoved { unit_id, x, y });
            Ok(())
        } else {
            Err(GameError::UnitNotFound)
        }
    }

    fn attack_unit(&mut self, attacker_id: usize, defender_id: usize) -> GameResult<()> {
        // Check if we're in the combat phase
        if self.phase != GamePhase::Combat {
            return Err(GameError::PhaseError);
        }

        // Validate attack
        if !self.can_attack(attacker_id, defender_id) {
            return Err(GameError::InvalidAttack);
        }

        let combat_result = {
            let attacker = self.get_unit(attacker_id)
                .ok_or(GameError::UnitNotFound)?;
            let defender = self.get_unit(defender_id)
                .ok_or(GameError::UnitNotFound)?;
            
            let terrain = self.get_terrain_at(
                defender.get_position().x,
                defender.get_position().y
            ).ok_or(GameError::InvalidMove)?;

            // Calculate combat result
            self.combat_resolver.resolve_combat(
                attacker,
                defender,
                terrain,
                self.weather,
                self.time_of_day,
                None,
            )?
        };

        // Update statistics
        self.statistics.damage_dealt.entry(combat_result.attacker_id.to_string())
            .and_modify(|d| *d += combat_result.attacker_damage_dealt)
            .or_insert(combat_result.attacker_damage_dealt);

        // Emit combat events
        for event in combat_result.combat_events {
            self.emit_event(GameEvent::Combat(event));
        }

        Ok(())
    }

    fn repair_unit(&mut self, unit_id: usize) -> GameResult<()> {
        let unit = self.get_unit_mut(unit_id)
            .ok_or(GameError::UnitNotFound)?;

        // Check if unit is in a valid repair location
        let (x, y) = (unit.get_position().x, unit.get_position().y);
        let can_repair = self.cities.values()
            .chain(self.ports.values())
            .chain(self.airbases.values())
            .any(|facility| {
                let (fx, fy) = facility.get_position();
                calculate_distance(x, y, fx, fy) < DEFAULT_SUPPLY_RANGE
            });

        if !can_repair {
            return Err(GameError::InvalidMove);
        }

        // Check if unit has sufficient supplies for repair
        let supply_level = self.get_supply_level(unit_id)?;
        if supply_level < 0.5 {
            return Err(GameError::InsufficientSupplies);
        }

        // Calculate repair amount based on supply level and unit type
        let repair_points = 20.0 * supply_level;
        
        // Apply repairs
        match unit {
            Unit::Ship(ship) => ship.repair_hull(repair_points),
            Unit::Land(land) => land.repair(repair_points),
            Unit::Air(air) => air.repair(repair_points),
        };

        // Emit repair event
        self.emit_event(GameEvent::UnitRepaired { unit_id });

        Ok(())
    }

    fn resupply_unit(&mut self, unit_id: usize) -> GameResult<()> {
        // Check if we're in the supply phase
        if self.phase != GamePhase::Supply {
            return Err(GameError::PhaseError);
        }

        let unit = self.get_unit_mut(unit_id)
            .ok_or(GameError::UnitNotFound)?;

        // Find nearest supply source
        let nearest_source = self.find_nearest_supply_source(unit_id)?;
        
        // Calculate supply amount based on distance and source capacity
        let supply_amount = self.calculate_supply_amount(unit_id, nearest_source);
        
        // Create supply package
        let supplies = Arsenal {
            ammunition: (supply_amount * 100.0) as i32,
            fuel: supply_amount * 50.0,
            supplies: (supply_amount * 75.0) as i32,
            missiles: HashMap::new(), // Specific missile resupply would be handled separately
        };

        // Apply resupply
        unit.resupply(&supplies);
        self.statistics.supply_consumed += supply_amount;

        // Emit resupply event
        self.emit_event(GameEvent::UnitResupplied { 
            unit_id,
            amount: supply_amount 
        });

        Ok(())
    }

    fn bomb_road(&mut self, road_id: usize, damage: f64) -> GameResult<()> {
        let road = self.roads.get_mut(&road_id)
            .ok_or(GameError::RoadNotFound)?;

        // Apply damage to road
        road.condition = (road.condition - damage).max(0.0);
        
        self.emit_event(GameEvent::RoadDamaged {
            road_id,
            damage,
        });

        // Update supply network if road becomes unusable
        if road.condition < 0.2 {
            self.update_supply_network();
        }

        Ok(())
    }

    fn mine_road(&mut self, road_id: usize) -> GameResult<()> {
        let road = self.roads.get_mut(&road_id)
            .ok_or(GameError::RoadNotFound)?;

        road.is_mined = true;
        
        // Update supply network
        self.update_supply_network();

        self.emit_event(GameEvent::RoadMined { road_id });

        Ok(())
    }
}

impl GameQueries for Game {
    fn get_unit(&self, unit_id: usize) -> Option<&dyn MilitaryUnit> {
        if let Some(unit) = self.land_units.get(&unit_id) {
            Some(unit as &dyn MilitaryUnit)
        } else if let Some(ship) = self.ships.get(&unit_id) {
            Some(ship as &dyn MilitaryUnit)
        } else if let Some(air) = self.air_units.get(&unit_id) {
            Some(air as &dyn MilitaryUnit)
        } else {
            None
        }
    }
    fn can_attack(&self, attacker_id: usize, defender_id: usize) -> bool {
        if let (Some(attacker), Some(defender)) = (self.get_unit(attacker_id), self.get_unit(defender_id)) {
            // Check if the attacker can reach the defender and if it's a valid target
            let distance = self.calculate_distance(attacker.get_position(), defender.get_position());
            distance <= attacker.get_attack_range() && attacker.can_attack(defender)
        } else {
            false
        }
    }

    fn can_move_to(&self, unit_id: usize, x: f64, y: f64) -> bool {
        if let Some(unit) = self.get_unit(unit_id) {
            let current_position = unit.get_position();
            let destination = (x as i32, y as i32);
            let distance = self.calculate_distance(current_position, destination);
            distance <= unit.get_movement_range() && !self.is_obstructed(destination)
        } else {
            false
        }
    }

    fn get_city(&self, city_id: usize) -> Option<&City> {
        self.cities.get(&city_id)
    }

    fn get_road(&self, road_id: usize) -> Option<&Road> {
        self.roads.get(&road_id)
    }

    fn get_supply_level(&self, unit_id: usize) -> GameResult<f32> {
        if let Some(unit) = self.get_unit(unit_id) {
            Ok(unit.get_supply_level())
        } else {
            Err(GameError::UnitNotFound)
        }
    }

    fn get_units_in_range(&self, x: f64, y: f64, range: f64) -> Vec<usize> {
        let origin = (x as i32, y as i32);
        self.land_units
            .iter()
            .chain(self.ships.iter())
            .chain(self.air_units.iter())
            .filter_map(|(&id, unit)| {
                let distance = self.calculate_distance(origin, unit.get_position());
                if distance <= range {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl GameEnvironment for Game {
    fn get_weather(&self) -> Weather {
        self.weather
    }

    fn get_time_of_day(&self) -> TimeOfDay {
        self.time_of_day
    }

    fn get_terrain_at(&self, x: f64, y: f64) -> Option<&HexTile> {
        self.tiles.iter().find(|tile| tile.contains_point(x, y))
    }

    fn is_position_visible(&self, x: f64, y: f64) -> bool {
        if let Some(tile_index) = self.tiles.iter().position(|tile| tile.contains_point(x, y)) {
            self.visibility_map[tile_index]
        } else {
            false
        }
    }
}

// Helper functions
fn calculate_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

impl Game {
    fn count_faction_units(&self, faction: &str) -> u32 {
        let mut count = 0;
        
        for unit in self.land_units.values()
            .chain(self.ships.values())
            .chain(self.air_units.values())
        {
            if unit.get_faction() == faction && unit.get_status() != UnitStatus::Destroyed {
                count += 1;
            }
        }
        
        count
    }

    fn get_roads_between(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Vec<&Road> {
        let mut relevant_roads = Vec::new();
        let path_bbox = BoundingBox::from_points(start_x, start_y, end_x, end_y);

        for road in self.roads.values() {
            if let (Some(start_city), Some(end_city)) = (
                self.cities.get(&road.start_city),
                self.cities.get(&road.end_city)
            ) {
                let road_bbox = BoundingBox::from_points(
                    start_city.base.x, start_city.base.y,
                    end_city.base.x, end_city.base.y
                );

                if path_bbox.intersects(&road_bbox) {
                    relevant_roads.push(road);
                }
            }
        }

        relevant_roads
    }

    fn get_enemy_units_between(&self, start_x: f64, start_y: f64, end_x: f64, end_y: f64) -> Vec<&dyn MilitaryUnit> {
        let mut enemy_units = Vec::new();
        let path_bbox = BoundingBox::from_points(start_x, start_y, end_x, end_y);
        let buffer_distance = 50.0; // Detection range for supply line interference

        for unit in self.land_units.values()
            .chain(self.ships.values())
            .chain(self.air_units.values())
        {
            let pos = unit.get_position();
            if path_bbox.contains_with_buffer(pos.x, pos.y, buffer_distance) {
                enemy_units.push(unit as &dyn MilitaryUnit);
            }
        }

        enemy_units
    }

    fn can_block_supply(&self, unit: &dyn MilitaryUnit) -> bool {
        match unit.get_status() {
            UnitStatus::Active | UnitStatus::Entrenched => true,
            _ => false,
        }
    }

    pub fn get_tiles_in_range(&self, x: f64, y: f64, range: f64) -> Vec<usize> {
        let mut tiles_in_range = Vec::new();
        
        for (index, tile) in self.tiles.iter().enumerate() {
            let distance = calculate_distance(x, y, tile.center_x, tile.center_y);
            if distance <= range {
                tiles_in_range.push(index);
            }
        }
        
        tiles_in_range
    }

    pub fn add_event_listener(&mut self, listener: EventListener) {
        self.event_listeners.push(listener);
    }

    pub fn get_event_history(&self) -> &[GameEvent] {
        &self.event_history
    }
}

impl GameActions for Game {
    fn attack_unit(&mut self, attacker_id: usize, defender_id: usize) -> GameResult<()> {
        if self.phase != GamePhase::Combat {
            return Err(GameError::PhaseError);
        }

        let can_attack = self.can_attack(attacker_id, defender_id);
        if !can_attack {
            return Err(GameError::InvalidAttack);
        }

        let damage = {
            let attacker = self.get_unit(attacker_id)
                .ok_or(GameError::UnitNotFound)?;
            let defender = self.get_unit(defender_id)
                .ok_or(GameError::UnitNotFound)?;
            
            attacker.calculate_damage(defender)
        };

        // Apply damage to defender
        if let Some(defender) = self.get_unit_mut(defender_id) {
            defender.receive_damage(damage);
            
            // Check if unit was destroyed
            if defender.get_status() == UnitStatus::Destroyed {
                if let Some(faction) = defender.get_faction().map(String::from) {
                    *self.statistics.units_lost.entry(faction).or_insert(0) += 1;
                }
                self.emit_event(GameEvent::UnitDestroyed { unit_id: defender_id });
            }
        }

        self.emit_event(GameEvent::UnitAttacked {
            attacker_id,
            defender_id,
            damage,
        });

        Ok(())
    }

    fn repair_unit(&mut self, unit_id: usize) -> GameResult<()> {
        let unit = self.get_unit_mut(unit_id)
            .ok_or(GameError::UnitNotFound)?;

        if unit.get_status() == UnitStatus::Destroyed {
            return Err(GameError::UnitError(UnitError::Disabled));
        }

        // Check if unit has sufficient supplies for repair
        let supply_level = self.get_supply_level(unit_id)?;
        if supply_level < 0.5 {
            return Err(GameError::InsufficientSupplies);
        }

        // Calculate repair amount based on supply level and unit type
        let repair_points = 20.0 * supply_level;
        unit.repair(repair_points);

        Ok(())
    }

    fn resupply_unit(&mut self, unit_id: usize) -> GameResult<()> {
        let unit = self.get_unit_mut(unit_id)
            .ok_or(GameError::UnitNotFound)?;

        // Find nearest supply source
        let nearest_source = self.find_nearest_supply_source(unit_id)?;
        
        // Calculate supply amount based on distance and source capacity
        let supply_amount = self.calculate_supply_amount(unit_id, nearest_source);
        
        // Create supply package
        let supplies = Arsenal {
            ammunition: (supply_amount * 100.0) as i32,
            fuel: supply_amount * 50.0,
            supplies: (supply_amount * 75.0) as i32,
            missiles: HashMap::new(), // Specific missile resupply would be handled separately
        };

        unit.resupply(&supplies);
        self.statistics.supply_consumed += supply_amount;

        Ok(())
    }

    fn bomb_road(&mut self, road_id: usize, damage: f64) -> GameResult<()> {
        let road = self.roads.get_mut(&road_id)
            .ok_or(GameError::RoadNotFound)?;

        road.condition = (road.condition - damage).max(0.0);
        
        self.emit_event(GameEvent::RoadDamaged {
            road_id,
            damage,
        });

        // Update supply network if road becomes unusable
        if road.condition < 0.2 {
            self.update_supply_network();
        }

        Ok(())
    }

    fn mine_road(&mut self, road_id: usize) -> GameResult<()> {
        let road = self.roads.get_mut(&road_id)
            .ok_or(GameError::RoadNotFound)?;

        road.is_mined = true;
        
        // Update supply network
        self.update_supply_network();

        Ok(())
    }
    fn move_unit(&mut self, unit_id: usize, x: f64, y: f64) -> GameResult<()> {
        if self.phase != GamePhase::Movement {
            return Err(GameError::PhaseError);
        }

        let unit = self.get_unit_mut(unit_id)
            .ok_or(GameError::UnitNotFound)?;

        if unit.get_status() == UnitStatus::Destroyed {
            return Err(GameError::UnitError(UnitError::Disabled));
        }

        // Validate if the unit can reach the destination
        let can_move = self.can_move_to(unit_id, x, y);
        if !can_move {
            return Err(GameError::InvalidMovement);
        }

        // Update the unit's position
        let destination = (x as i32, y as i32);
        unit.set_position(destination);
        self.emit_event(GameEvent::UnitMoved {
            unit_id,
            x,
            y,
        });

        Ok(())
    }
}

// Helper structs
struct BoundingBox {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl BoundingBox {
    fn from_points(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        BoundingBox {
            min_x: x1.min(x2),
            min_y: y1.min(y2),
            max_x: x1.max(x2),
            max_y: y1.max(y2),
        }
    }

    fn intersects(&self, other: &BoundingBox) -> bool {
        self.min_x <= other.max_x &&
        self.max_x >= other.min_x &&
        self.min_y <= other.max_y &&
        self.max_y >= other.min_y
    }

    fn contains_with_buffer(&self, x: f64, y: f64, buffer: f64) -> bool {
        x >= (self.min_x - buffer) &&
        x <= (self.max_x + buffer) &&
        y >= (self.min_y - buffer) &&
        y <= (self.max_y + buffer)
    }
}

impl Game {
    fn find_nearest_supply_source(&self, unit_id: usize) -> GameResult<usize> {
        let unit = self.get_unit(unit_id)
            .ok_or(GameError::UnitNotFound)?;
        let unit_pos = unit.get_position();

        self.cities.iter()
            .filter(|(_, city)| city.base.storage > 0)
            .min_by_key(|(_, city)| {
                let distance = calculate_distance(
                    unit_pos.x, unit_pos.y,
                    city.base.x, city.base.y
                );
                (distance * 100.0) as i32 // Convert to integer for comparison
            })
            .map(|(id, _)| *id)
            .ok_or(GameError::InsufficientSupplies)
    }

    fn calculate_supply_amount(&self, unit_id: usize, source_id: usize) -> f32 {
        let unit = match self.get_unit(unit_id) {
            Some(u) => u,
            None => return 0.0,
        };

        let source = match self.cities.get(&source_id) {
            Some(c) => c,
            None => return 0.0,
        };

        let distance = calculate_distance(
            unit.get_position().x, unit.get_position().y,
            source.base.x, source.base.y
        );

        // Base amount depends on source capacity
        let base_amount = source.base.storage as f32 * 0.01;

        // Apply distance penalty
        let distance_factor = (1000.0 - distance) / 1000.0;
        let amount = base_amount * distance_factor;

        // Apply weather penalty
        let weather_modifier = match self.weather {
            Weather::Clear => 1.0,
            Weather::Rain => 0.8,
            Weather::Storm => 0.5,
            Weather::Fog => 0.7,
        };

        amount * weather_modifier
    }
}