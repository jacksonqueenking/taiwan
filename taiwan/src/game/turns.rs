use std::fmt;
use std::time::{Duration, Instant};

use crate::game::{
    GameError, GameResult, GameEvent,
    Weather, TimeOfDay,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Planning {
        time_remaining: Duration,
    },
    Movement {
        moves_remaining: u32,
    },
    Combat {
        attacks_remaining: u32,
    },
    Supply {
        supply_points: u32,
    },
    EndTurn,
}

impl Phase {
    pub fn new_planning() -> Self {
        Phase::Planning {
            time_remaining: Duration::from_secs(300), // 5 minutes for planning
        }
    }

    pub fn new_movement(unit_count: u32) -> Self {
        Phase::Movement {
            moves_remaining: unit_count,
        }
    }

    pub fn new_combat(unit_count: u32) -> Self {
        Phase::Combat {
            attacks_remaining: unit_count,
        }
    }

    pub fn new_supply(supply_points: u32) -> Self {
        Phase::Supply {
            supply_points,
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            Phase::Planning { time_remaining } => time_remaining.as_secs() == 0,
            Phase::Movement { moves_remaining } => *moves_remaining == 0,
            Phase::Combat { attacks_remaining } => *attacks_remaining == 0,
            Phase::Supply { supply_points } => *supply_points == 0,
            Phase::EndTurn => true,
        }
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Phase::Planning { time_remaining } => {
                write!(f, "Planning Phase ({:?} remaining)", time_remaining)
            }
            Phase::Movement { moves_remaining } => {
                write!(f, "Movement Phase ({} moves remaining)", moves_remaining)
            }
            Phase::Combat { attacks_remaining } => {
                write!(f, "Combat Phase ({} attacks remaining)", attacks_remaining)
            }
            Phase::Supply { supply_points } => {
                write!(f, "Supply Phase ({} points remaining)", supply_points)
            }
            Phase::EndTurn => write!(f, "End Turn Phase"),
        }
    }
}

pub struct TurnManager {
    current_turn: u32,
    current_phase: Phase,
    phase_start_time: Instant,
    time_of_day: TimeOfDay,
    weather: Weather,
    events: Vec<GameEvent>,
}

impl TurnManager {
    pub fn new() -> Self {
        TurnManager {
            current_turn: 1,
            current_phase: Phase::new_planning(),
            phase_start_time: Instant::now(),
            time_of_day: TimeOfDay::Dawn,
            weather: Weather::Clear,
            events: Vec::new(),
        }
    }

    pub fn advance_phase(&mut self, unit_count: u32) -> GameResult<Phase> {
        let new_phase = match self.current_phase {
            Phase::Planning { .. } => Phase::new_movement(unit_count),
            Phase::Movement { .. } => Phase::new_combat(unit_count),
            Phase::Combat { .. } => Phase::new_supply(calculate_supply_points(unit_count)),
            Phase::Supply { .. } => Phase::EndTurn,
            Phase::EndTurn => {
                self.advance_turn()?;
                Phase::new_planning()
            }
        };

        self.phase_start_time = Instant::now();
        self.current_phase = new_phase;
        
        self.events.push(GameEvent::PhaseChanged { 
            new_phase: self.current_phase 
        });

        Ok(new_phase)
    }

    pub fn advance_turn(&mut self) -> GameResult<()> {
        self.current_turn += 1;
        self.advance_time_of_day();
        self.update_weather();
        
        self.events.push(GameEvent::TurnCompleted { 
            turn_number: self.current_turn 
        });

        Ok(())
    }

    pub fn advance_time_of_day(&mut self) {
        self.time_of_day = match self.time_of_day {
            TimeOfDay::Dawn => TimeOfDay::Day,
            TimeOfDay::Day => TimeOfDay::Dusk,
            TimeOfDay::Dusk => TimeOfDay::Night,
            TimeOfDay::Night => TimeOfDay::Dawn,
        };
    }

    pub fn update_weather(&mut self) {
        // 30% chance of weather changing each turn
        if rand::random::<f32>() < 0.3 {
            let new_weather = match rand::random::<f32>() {
                x if x < 0.4 => Weather::Clear,
                x if x < 0.7 => Weather::Rain,
                x if x < 0.9 => Weather::Storm,
                _ => Weather::Fog,
            };

            if new_weather != self.weather {
                self.weather = new_weather;
                self.events.push(GameEvent::WeatherChanged { 
                    new_weather: self.weather 
                });
            }
        }
    }

    pub fn consume_action(&mut self) -> GameResult<()> {
        match self.current_phase {
            Phase::Movement { moves_remaining } => {
                if moves_remaining > 0 {
                    self.current_phase = Phase::Movement {
                        moves_remaining: moves_remaining - 1
                    };
                    Ok(())
                } else {
                    Err(GameError::PhaseError)
                }
            }
            Phase::Combat { attacks_remaining } => {
                if attacks_remaining > 0 {
                    self.current_phase = Phase::Combat {
                        attacks_remaining: attacks_remaining - 1
                    };
                    Ok(())
                } else {
                    Err(GameError::PhaseError)
                }
            }
            Phase::Supply { supply_points } => {
                if supply_points > 0 {
                    self.current_phase = Phase::Supply {
                        supply_points: supply_points - 1
                    };
                    Ok(())
                } else {
                    Err(GameError::PhaseError)
                }
            }
            _ => Err(GameError::PhaseError),
        }
    }

    pub fn get_current_turn(&self) -> u32 {
        self.current_turn
    }

    pub fn get_current_phase(&self) -> Phase {
        self.current_phase
    }

    pub fn get_time_of_day(&self) -> TimeOfDay {
        self.time_of_day
    }

    pub fn get_weather(&self) -> Weather {
        self.weather
    }

    pub fn get_events(&self) -> &[GameEvent] {
        &self.events
    }

    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    pub fn update_planning_time(&mut self) {
        if let Phase::Planning { time_remaining } = self.current_phase {
            let elapsed = self.phase_start_time.elapsed();
            self.current_phase = Phase::Planning {
                time_remaining: time_remaining.saturating_sub(elapsed)
            };
        }
    }
}

fn calculate_supply_points(unit_count: u32) -> u32 {
    // Base supply points is 2 per unit
    let base_points = unit_count * 2;
    
    // Add bonus points for larger armies (logistics scale)
    let bonus_points = (unit_count as f32).sqrt() as u32;
    
    base_points + bonus_points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_transitions() {
        let mut manager = TurnManager::new();
        assert_eq!(manager.get_current_turn(), 1);
        
        // Test phase advancement
        let unit_count = 5;
        assert!(matches!(manager.get_current_phase(), Phase::Planning { .. }));
        
        manager.advance_phase(unit_count).unwrap();
        assert!(matches!(manager.get_current_phase(), Phase::Movement { .. }));
        
        manager.advance_phase(unit_count).unwrap();
        assert!(matches!(manager.get_current_phase(), Phase::Combat { .. }));
        
        manager.advance_phase(unit_count).unwrap();
        assert!(matches!(manager.get_current_phase(), Phase::Supply { .. }));
        
        manager.advance_phase(unit_count).unwrap();
        assert!(matches!(manager.get_current_phase(), Phase::EndTurn));
        
        // Test turn advancement
        manager.advance_phase(unit_count).unwrap();
        assert_eq!(manager.get_current_turn(), 2);
        assert!(matches!(manager.get_current_phase(), Phase::Planning { .. }));
    }

    #[test]
    fn test_action_consumption() {
        let mut manager = TurnManager::new();
        manager.advance_phase(2).unwrap(); // Move to Movement phase with 2 moves
        
        assert!(manager.consume_action().is_ok());
        assert!(manager.consume_action().is_ok());
        assert!(manager.consume_action().is_err()); // Should fail - no moves left
    }
}