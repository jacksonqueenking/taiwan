use std::collections::HashMap;
use crate::units::{
    UnitStatus, UnitStats, Arsenal, Position,
    MissileType, Missile, UnitError, UnitResult,
};

/// Base trait for all military units in the game
pub trait MilitaryUnit {
    /// Get the unit's current position
    fn get_position(&self) -> &Position;
    
    /// Get the unit's current stats
    fn get_stats(&self) -> &UnitStats;
    
    /// Get the unit's arsenal
    fn get_arsenal(&self) -> &Arsenal;
    
    /// Get the unit's current status
    fn get_status(&self) -> UnitStatus;
    
    /// Check if the unit can attack a specific target
    fn can_attack(&self, target: &dyn MilitaryUnit) -> bool;
    
    /// Calculate potential damage against a target
    fn calculate_damage(&self, target: &dyn MilitaryUnit) -> f32;
    
    /// Handle receiving damage
    fn receive_damage(&mut self, damage: f32);
    
    /// Update unit's supply level
    fn update_supply(&mut self, supply_rate: f32);
}

/// Trait for unit movement capabilities
pub trait Movable {
    /// Move unit to a new position
    fn move_to(&mut self, x: f64, y: f64);
    
    /// Get unit's current speed
    fn get_speed(&self) -> f32;
    
    /// Default implementation for getting operational range
    fn get_range(&self) -> f32 {
        self.get_speed() * 24.0 // Default 24-hour range
    }
}

/// Trait for heavy units that need special transport consideration
pub trait Heavy {
    /// Get the unit's weight in tonnes
    fn weight(&self) -> f32;
}

/// Trait for units that can engage in air operations
pub trait Flyable: Movable {
    /// Set the unit's altitude
    fn set_altitude(&mut self, altitude: f32);
    
    /// Get the unit's current altitude
    fn get_altitude(&self) -> f32;
    
    /// Get the unit's maximum operational ceiling
    fn get_ceiling(&self) -> f32;
}

/// Trait for naval units that can operate at different depths
pub trait Sailable: Movable {
    /// Set the unit's depth
    fn set_depth(&mut self, depth: Option<f32>);
    
    /// Get the unit's current depth
    fn get_depth(&self) -> Option<f32>;
    
    /// Get the unit's maximum operational depth
    fn get_max_depth(&self) -> Option<f32>;
}

/// Trait for tracking unit damage states
pub trait Damageable {
    /// Get unit's current health as a percentage
    fn get_health(&self) -> f32;
    
    /// Check if unit is disabled
    fn is_disabled(&self) -> bool;
    
    /// Check if unit is destroyed
    fn is_destroyed(&self) -> bool;
}

/// Trait for handling unit attrition
pub trait Attritable {
    /// Get the unit's current attrition rate
    fn get_attrition_rate(&self) -> f32;
    
    /// Apply attrition effects to the unit
    fn apply_attrition(&mut self);
}

/// Trait for units that can be targeted by bombing attacks
pub trait Bombable {
    /// Get the unit's hardness rating (resistance to bombing)
    fn get_hardness(&self) -> f32;
    
    /// Calculate damage from a bombing attack
    /// Default implementation provided
    fn calculate_bomb_damage(&self, warhead_size: f32) -> f32 {
        warhead_size * (1.0 - self.get_hardness())
    }
}

/// Trait for units that can be detected by various sensor systems
pub trait Detectable {
    /// Get the unit's signature strength
    fn get_signature(&self) -> f32;
    
    /// Get unit's current visibility
    fn get_visibility(&self) -> f32;
    
    /// Check if unit can be detected by another unit
    fn can_be_detected_by(&self, detector: &dyn MilitaryUnit) -> bool;
}

/// Trait for units with stealth capabilities
pub trait StealthCapable {
    /// Get the unit's stealth rating
    fn get_stealth_level(&self) -> f32;
    
    /// Check if stealth systems are currently active
    fn is_stealth_active(&self) -> bool;
    
    /// Activate stealth systems
    fn activate_stealth(&mut self);
    
    /// Deactivate stealth systems
    fn deactivate_stealth(&mut self);
}

/// Trait for handling combat between units
pub trait Combatable {
    /// Get unit's attack strength
    fn attack_strength(&self) -> f32;
    
    /// Get unit's defense strength
    fn defense_strength(&self) -> f32;
    
    /// Calculate probability of hitting target
    fn calculate_hit_chance(&self, target: &dyn MilitaryUnit) -> f32;
}

/// Trait for units with anti-aircraft capabilities
pub trait AntiAir {
    /// Get unit's anti-aircraft range
    fn aa_range(&self) -> f32;
    
    /// Get unit's anti-aircraft combat strength
    fn aa_strength(&self) -> f32;
    
    /// Check if unit can engage an air target
    fn can_engage_air(&self, target: &dyn Flyable) -> bool;
}

/// Trait for units that require maintenance
pub trait Maintainable {
    /// Perform repairs on the unit
    fn repair(&mut self, repair_points: f32);
    
    /// Resupply the unit
    fn resupply(&mut self, supplies: &Arsenal);
    
    /// Get unit's maintenance cost
    fn get_maintenance_cost(&self) -> f32;
}

/// Trait for units that can be transported
pub trait Transportable {
    /// Get unit's weight for transport calculations
    fn get_weight(&self) -> f32;
    
    /// Get unit's volume for transport calculations
    fn get_volume(&self) -> f32;
    
    /// Check if unit can be transported by another unit
    fn can_be_transported_by(&self, transporter: &dyn MilitaryUnit) -> bool;
}

/// Helper functions for combat calculations
pub mod combat {
    use super::*;

    /// Calculate base hit chance between attacker and target
    pub fn calculate_base_hit_chance(attacker: &dyn MilitaryUnit, target: &dyn MilitaryUnit) -> f32 {
        let distance = calculate_distance(
            attacker.get_position(),
            target.get_position()
        );
        
        let base_chance = 0.8;
        let distance_modifier = (1000.0 - distance) / 1000.0;
        let weather_modifier = 1.0; // TODO: Implement weather effects
        
        (base_chance * distance_modifier * weather_modifier).max(0.0).min(1.0)
    }

    /// Calculate distance between two positions
    pub fn calculate_distance(pos1: &Position, pos2: &Position) -> f64 {
        let dx = pos2.x - pos1.x;
        let dy = pos2.y - pos1.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate combat effectiveness considering multiple factors
    pub fn calculate_combat_effectiveness(
        unit: &dyn MilitaryUnit,
        terrain_modifier: f32,
        weather_modifier: f32,
    ) -> f32 {
        let stats = unit.get_stats();
        let supply_modifier = if stats.supply_level < 0.5 { 0.7 } else { 1.0 };
        let morale_modifier = stats.morale;
        let training_modifier = stats.training;
        let fatigue_modifier = 1.0 - stats.fatigue;

        terrain_modifier * weather_modifier * supply_modifier * 
        morale_modifier * training_modifier * fatigue_modifier
    }
}

/// Helper functions for movement calculations
pub mod movement {
    use super::*;

    /// Calculate movement cost considering terrain and weather
    pub fn calculate_movement_cost(
        unit: &dyn Movable,
        terrain_cost: f32,
        weather_cost: f32
    ) -> f32 {
        let base_cost = 1.0;
        let speed_modifier = unit.get_speed() / 100.0;
        
        base_cost * terrain_cost * weather_cost * speed_modifier
    }

    /// Check if movement is possible between two points
    pub fn can_move_between(
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
        unit: &dyn Movable,
        obstacles: &[Position]
    ) -> bool {
        let distance = ((end_x - start_x).powi(2) + (end_y - start_y).powi(2)).sqrt();
        
        // Check if distance is within unit's range
        if distance > unit.get_range() {
            return false;
        }
        
        // Check for obstacles
        for obstacle in obstacles {
            let obstacle_distance = calculate_obstacle_interference(
                start_x, start_y,
                end_x, end_y,
                obstacle.x, obstacle.y
            );
            if obstacle_distance < 1.0 {
                return false;
            }
        }
        
        true
    }

    /// Calculate how much an obstacle interferes with movement
    fn calculate_obstacle_interference(
        start_x: f64, start_y: f64,
        end_x: f64, end_y: f64,
        obstacle_x: f64, obstacle_y: f64
    ) -> f64 {
        // Simplified line-point distance calculation
        let line_length = ((end_x - start_x).powi(2) + (end_y - start_y).powi(2)).sqrt();
        if line_length == 0.0 {
            return ((obstacle_x - start_x).powi(2) + (obstacle_y - start_y).powi(2)).sqrt();
        }

        let t = ((obstacle_x - start_x) * (end_x - start_x) + 
                 (obstacle_y - start_y) * (end_y - start_y)) / line_length.powi(2);

        if t < 0.0 {
            ((obstacle_x - start_x).powi(2) + (obstacle_y - start_y).powi(2)).sqrt()
        } else if t > 1.0 {
            ((obstacle_x - end_x).powi(2) + (obstacle_y - end_y).powi(2)).sqrt()
        } else {
            let proj_x = start_x + t * (end_x - start_x);
            let proj_y = start_y + t * (end_y - start_y);
            ((obstacle_x - proj_x).powi(2) + (obstacle_y - proj_y).powi(2)).sqrt()
        }
    }
}