

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

struct Ship {
    hull_size: i32,
    hull_integrity: f32,
    fuel: i32,
    // ... other properties
}

//air units

