

enum TerrainType {
    Mountain,
    Hill,
    Forest,
    // ... other terrains
}

struct Tile {
    terrain: TerrainType,
    visibility_multiplier: f32,
    concealment_multiplier: f32,
    // ... other properties
}

//land units

enum LandUnitType {
    Infantry,
    Armor,
    Artillery,
    SpecialForces,
    // ... add other types as needed
}

struct TerrainBonus {
    offensive_multiplier: f32,
    defensive_multiplier: f32,
    speed_multiplier: f32,
}

struct LandUnit {
    unit_type: UnitType,
    strength: i32,  // Represents the number of personnel or "health" of the unit.
    morale: f32,    // A value between 0 and 1, where 1 is high morale.
    visibility: f32,
    ammunition: i32,
    supply: i32,
    terrain_bonus: Option<TerrainBonus>,  // This can be None if the unit is in a neutral terrain.
    entrenchment: f32,  // A value between 0 and 1, where 1 is fully entrenched.
    aa_range: i32,  // Anti-aircraft range.
    aa_ammo: i32,
    missile_capabilities: Vec<Missile>,
    // ... other properties
}

impl LandUnit {
    fn attack(&self, target: &LandUnit) -> i32 {
        // Calculate damage based on unit type, terrain, morale, etc.
        // Return the damage value.
        0  // Placeholder
    }

    fn defend(&mut self, damage: i32) {
        // Reduce strength based on damage and other factors.
        // Modify morale based on casualties.
    }

    // ... other methods
}

//sea units
struct SAG { //SAG = Suface Action Group 
    ships: vec<Ship>,
}

struct Ship {
    name: String,
    ordnance: Vec<Ordnance>,  // A list of ordnance the ship can fire
    lift: f64,  // Lift capacity in tonnes
    arsenal: Arsenal //stuff it is carrying, whether or not it can fire it or use it.
    hull_integrity: f32,
    fuel: i32,
    missile_capabilities: Vec<Missile>,
    // ... other properties
}

struct AircraftCarrier {
    base: Ship,
    aircraft: Vec<Airplane>,  // List of airplanes on the carrier
    // ... other attributes specific to aircraft carriers ...
}

struct Submarine {
    base: Ship,
    nuclear_powered: bool,  // true for nuclear submarines, false for diesel
    // ... other attributes specific to submarines ...
}

struct Cruiser {
    base: Ship,
    // ... attributes specific to cruisers ...
}

struct Destroyer {
    base: Ship,
    // ... attributes specific to destroyers ...
}

struct CivilianRoRoShip {
    base: Ship,
    // ... attributes specific to civilian roll-on-roll-off ships ...
}

struct AmphibiousLandingShip {
    base: Ship,
    // ... attributes specific to amphibious landing ships ...
}

//air units

struct AirUnit {
    name: String,
    strength: usize,  // Number of planes or other entities in the unit
    missile_capabilities: Vec<Missile>,
    // ... other common attributes ...
}

enum FighterGeneration {
    Fourth,
    FourthPointFive,
    Fifth,
}

struct FighterSquadron {
    base: AirUnit,
    generation: FighterGeneration,
    // ... other attributes specific to fighter squadrons ...
}

impl FighterSquadron {
    fn new(generation: FighterGeneration) -> Self {
        let (name, strength) = match generation {
            FighterGeneration::Fourth => ("4th Generation Fighter Squadron".to_string(), 24),
            FighterGeneration::FourthPointFive => ("4.5th Generation Fighter Squadron".to_string(), 24),
            FighterGeneration::Fifth => ("5th Generation Fighter Squadron".to_string(), 24),
        };

        FighterSquadron {
            base: AirUnit { name, strength },
            generation,
        }
    }
}

enum BomberType {
    Stealth,
    NonStealth,
}

struct BomberSquadron {
    base: AirUnit,
    bomber_type: BomberType,
    // ... other attributes specific to bomber squadrons ...
}

impl BomberSquadron {
    fn new(bomber_type: BomberType) -> Self {
        let (name, strength) = match bomber_type {
            BomberType::Stealth => ("Stealth Bomber Squadron".to_string(), 12),
            BomberType::NonStealth => ("Non-Stealth Bomber Squadron".to_string(), 12),
        };

        BomberSquadron {
            base: AirUnit { name, strength },
            bomber_type,
        }
    }
}

// Missiles

enum MissileTarget {
    Air,
    Ship,
    Land,
    Multiple(Vec<MissileTarget>),  // For missiles that can strike multiple types of targets
}

struct Missile {
    name: String,
    range: f64,  // Range in kilometers
    weight: f64,  // Weight in tonnes
    target: MissileTarget,
    // ... other attributes ...
}

trait Movable {
    fn move_to(&mut self, x: f64, y: f64);
    fn current_position(&self) -> (f64, f64);
    // ... other movement-related methods ...
}

impl Movable for LandUnit {
    fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    fn current_position(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    // ... other method implementations ...
}

trait Heavy {
    fn weight(&self);
}

trait Damageable {
    fn damage(&self, damage: i32);
}

trait Attritable {
    fn attrite(&self, attrition: i32);
}

trait Bombable {

}

trait Flyable {

}

trait Sailable {

}

