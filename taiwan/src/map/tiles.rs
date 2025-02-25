use piston_window::*;
use crate::map::render::Drawable;
use crate::units::{TerrainBonus, Movable};

#[derive(Debug, Clone, PartialEq)]
pub enum TileType {
    Water { depth: f32 }, // depth in meters
    Coast { beach_length: f32 }, // beach length in km
    Land { elevation: f32 }, // elevation in meters
    Mountain { height: f32, slope: f32 }, // height and slope steepness
    Forest { density: f32 }, // density from 0.0 to 1.0
    Urban { density: f32 }, // urban density from 0.0 to 1.0
}

impl TileType {
    fn get_color(&self) -> [f32; 4] {
        match self {
            TileType::Water { depth } => {
                let intensity = (depth / 1000.0).min(1.0);
                [0.0, 0.0, 1.0 - intensity, 1.0] // Darker blue for deeper water
            },
            TileType::Coast { .. } => [0.8, 0.8, 0.6, 1.0], // Beige for coast
            TileType::Land { elevation } => {
                let intensity = (elevation / 1000.0).min(1.0);
                [0.2 + intensity * 0.4, 0.5 + intensity * 0.3, 0.2, 1.0] // Varying shades of green
            },
            TileType::Mountain { height, .. } => {
                let intensity = (height / 3000.0).min(1.0);
                [0.4 + intensity * 0.4, 0.4 + intensity * 0.4, 0.4 + intensity * 0.4, 1.0] // Gray
            },
            TileType::Forest { density } => [0.0, 0.4 + density * 0.4, 0.0, 1.0], // Dark green
            TileType::Urban { density } => [0.5 + density * 0.3, 0.5 + density * 0.3, 0.5 + density * 0.3, 1.0], // Gray
        }
    }

    fn get_terrain_bonus(&self) -> TerrainBonus {
        match self {
            TileType::Water { depth } => TerrainBonus {
                offensive_multiplier: 0.0,
                defensive_multiplier: 0.0,
                speed_multiplier: if *depth < 10.0 { 0.2 } else { 0.0 },
            },
            TileType::Coast { .. } => TerrainBonus {
                offensive_multiplier: 0.8,
                defensive_multiplier: 1.2,
                speed_multiplier: 0.7,
            },
            TileType::Land { .. } => TerrainBonus {
                offensive_multiplier: 1.0,
                defensive_multiplier: 1.0,
                speed_multiplier: 1.0,
            },
            TileType::Mountain { slope, .. } => TerrainBonus {
                offensive_multiplier: 0.6,
                defensive_multiplier: 1.5 + slope * 0.5,
                speed_multiplier: 0.4,
            },
            TileType::Forest { density } => TerrainBonus {
                offensive_multiplier: 0.8,
                defensive_multiplier: 1.0 + density * 0.5,
                speed_multiplier: 0.7,
            },
            TileType::Urban { density } => TerrainBonus {
                offensive_multiplier: 0.7,
                defensive_multiplier: 1.2 + density * 0.8,
                speed_multiplier: 0.9,
            },
        }
    }
}

pub struct HexTile {
    pub tile_type: TileType,
    pub center_x: f64,
    pub center_y: f64,
    pub size: f64,  // Distance from center to corner
    pub coordinates: (i32, i32), // Axial coordinates for hex grid
    pub is_visible: bool,
    pub is_explored: bool,
}

impl HexTile {
    pub fn new(tile_type: TileType, center_x: f64, center_y: f64, size: f64, coordinates: (i32, i32)) -> Self {
        HexTile {
            tile_type,
            center_x,
            center_y,
            size,
            coordinates,
            is_visible: false,
            is_explored: false,
        }
    }

    pub fn get_corners(&self) -> [(f64, f64); 6] {
        let angles = [0, 60, 120, 180, 240, 300];
        angles.map(|angle| {
            let radian = angle as f64 * std::f64::consts::PI / 180.0;
            (
                self.center_x + self.size * radian.cos(),
                self.center_y + self.size * radian.sin()
            )
        })
    }

    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        // Calculate distance from center
        let dx = x - self.center_x;
        let dy = y - self.center_y;
        let distance = (dx * dx + dy * dy).sqrt();
        
        // Check if point is within the hexagon
        distance <= self.size
    }
}

pub struct TileParameters {
    pub hex_size: f64,
    pub tiles_x: usize,
    pub tiles_y: usize,
    pub offset_type: OffsetType,
}

#[derive(Clone, Copy)]
pub enum OffsetType {
    Even,
    Odd,
}

impl Default for TileParameters {
    fn default() -> Self {
        TileParameters {
            hex_size: 50.0,
            tiles_x: 10,
            tiles_y: 10,
            offset_type: OffsetType::Even,
        }
    }
}

impl TileParameters {
    pub fn generate(&self) -> Vec<HexTile> {
        let mut tiles = Vec::new();
        let width = self.hex_size * 2.0;
        let height = width * 0.866; // height = 2 * size * sin(60°)
        
        for row in 0..self.tiles_y {
            for col in 0..self.tiles_x {
                let offset = match self.offset_type {
                    OffsetType::Even => if row % 2 == 0 { width / 2.0 } else { 0.0 },
                    OffsetType::Odd => if row % 2 == 1 { width / 2.0 } else { 0.0 },
                };
                
                let center_x = col as f64 * width + offset;
                let center_y = row as f64 * height;
                
                // Determine tile type based on position (example implementation)
                let tile_type = Self::determine_tile_type(col, row, self.tiles_x, self.tiles_y);
                
                tiles.push(HexTile::new(
                    tile_type,
                    center_x,
                    center_y,
                    self.hex_size,
                    (col as i32, row as i32),
                ));
            }
        }
        tiles
    }

    fn determine_tile_type(x: usize, y: usize, max_x: usize, max_y: usize) -> TileType {
        // Example terrain generation - could be replaced with more sophisticated algorithms
        let center_distance = {
            let dx = x as f64 - max_x as f64 / 2.0;
            let dy = y as f64 - max_y as f64 / 2.0;
            (dx * dx + dy * dy).sqrt()
        };

        if center_distance < max_x as f64 * 0.2 {
            TileType::Urban { density: 0.8 }
        } else if center_distance < max_x as f64 * 0.4 {
            TileType::Land { elevation: 100.0 }
        } else if center_distance < max_x as f64 * 0.6 {
            TileType::Forest { density: 0.7 }
        } else if center_distance < max_x as f64 * 0.8 {
            TileType::Mountain { height: 2000.0, slope: 0.6 }
        } else {
            TileType::Water { depth: 100.0 }
        }
    }
}

impl Drawable for HexTile {
    fn draw(&self, c: Context, g: &mut G2d) {
        let corners = self.get_corners();
        let color = self.tile_type.get_color();
        
        // Convert corners to the format expected by polygon
        let points: Vec<[f64; 2]> = corners
            .iter()
            .map(|&(x, y)| [x, y])
            .collect();

        // Draw the hexagon
        polygon(
            color,
            &points,
            c.transform,
            g
        );

        // Draw border
        for i in 0..6 {
            let start = corners[i];
            let end = corners[(i + 1) % 6];
            line(
                [0.0, 0.0, 0.0, 0.3], // Black with 0.3 alpha for border
                1.0,  // Line width
                [start.0, start.1, end.0, end.1],
                c.transform,
                g
            );
        }

        // Draw fog of war if not visible
        if !self.is_visible {
            polygon(
                [0.0, 0.0, 0.0, if self.is_explored { 0.3 } else { 0.7 }],
                &points,
                c.transform,
                g
            );
        }
    }
}