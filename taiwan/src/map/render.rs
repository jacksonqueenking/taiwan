use piston_window::*;
use crate::map::{
    tiles::{HexTile, TileParameters},
    terrain::TerrainRules,
};
use crate::infrastructure::{City, Road};
use crate::units::{LandUnit, Ship, AirUnit};
use std::collections::HashMap;

pub trait Drawable {
    fn draw(&self, c: &Context, g: &mut G2d);
}

pub struct MapRenderer {
    window_width: u32,
    window_height: u32,
    camera_x: f64,
    camera_y: f64,
    zoom: f64,
    font: Glyphs,
}

impl MapRenderer {
    pub fn new(window_width: u32, window_height: u32, font_path: &str, window: &mut PistonWindow) -> Self {
        let font = window.load_font(font_path).expect("Failed to load font");
        
        MapRenderer {
            window_width,
            window_height,
            camera_x: 0.0,
            camera_y: 0.0,
            zoom: 1.0,
            font,
        }
    }

    pub fn handle_input(&mut self, e: &Event) {
        // Handle camera movement with arrow keys
        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Left => self.camera_x -= 10.0,
                Key::Right => self.camera_x += 10.0,
                Key::Up => self.camera_y -= 10.0,
                Key::Down => self.camera_y += 10.0,
                _ => {}
            }
        }

        // Handle zoom with mouse wheel
        if let Some(args) = e.mouse_scroll_args() {
            self.zoom *= if args[1] > 0.0 { 1.1 } else { 0.9 };
            self.zoom = self.zoom.max(0.5).min(2.0); // Limit zoom range
        }
    }

    pub fn transform(&self) -> Matrix2d {
        // Create transformation matrix for camera and zoom
        [[self.zoom, 0.0, -self.camera_x * self.zoom + self.window_width as f64 / 2.0],
         [0.0, self.zoom, -self.camera_y * self.zoom + self.window_height as f64 / 2.0]]
    }

    pub fn draw_map(
        &mut self,
        tiles: &[HexTile],
        cities: &[City],
        roads: &[Road],
        land_units: &[LandUnit],
        ships: &[Ship],
        air_units: &[AirUnit],
        terrain_rules: &TerrainRules,
        c: Context,
        g: &mut G2d
    ) {
        let transform = self.transform();

        // Draw terrain tiles
        for tile in tiles {
            if self.is_visible_on_screen(tile) {
                tile.draw(&c.trans(transform[0][2], transform[1][2])
                          .scale(transform[0][0], transform[1][1]), g);
            }
        }

        // Draw roads
        self.draw_roads(roads, cities, &c.trans(transform[0][2], transform[1][2])
                                      .scale(transform[0][0], transform[1][1]), g);

        // Draw cities
        for city in cities {
            if self.is_city_visible_on_screen(city) {
                city.draw(&c.trans(transform[0][2], transform[1][2])
                          .scale(transform[0][0], transform[1][1]), g);
                self.draw_city_label(city, &c.trans(transform[0][2], transform[1][2])
                                            .scale(transform[0][0], transform[1][1]), g);
            }
        }

        // Draw units
        self.draw_units(land_units, ships, air_units, 
                       &c.trans(transform[0][2], transform[1][2])
                         .scale(transform[0][0], transform[1][1]), g);

        // Draw weather effects if any
        self.draw_weather_effects(terrain_rules, 
                                &c.trans(transform[0][2], transform[1][2])
                                  .scale(transform[0][0], transform[1][1]), g);

        // Draw UI elements
        self.draw_ui(&c, g);
    }

    fn is_visible_on_screen(&self, tile: &HexTile) -> bool {
        let screen_x = (tile.center_x - self.camera_x) * self.zoom + self.window_width as f64 / 2.0;
        let screen_y = (tile.center_y - self.camera_y) * self.zoom + self.window_height as f64 / 2.0;

        // Add buffer of tile size to ensure partially visible tiles are drawn
        screen_x > -tile.size && screen_x < self.window_width as f64 + tile.size &&
        screen_y > -tile.size && screen_y < self.window_height as f64 + tile.size
    }

    fn is_city_visible_on_screen(&self, city: &City) -> bool {
        let screen_x = (city.base.x - self.camera_x) * self.zoom + self.window_width as f64 / 2.0;
        let screen_y = (city.base.y - self.camera_y) * self.zoom + self.window_height as f64 / 2.0;

        screen_x > 0.0 && screen_x < self.window_width as f64 &&
        screen_y > 0.0 && screen_y < self.window_height as f64
    }

    fn draw_roads(&self, roads: &[Road], cities: &[City], c: &Context, g: &mut G2d) {
        for road in roads {
            let start_city = &cities[road.start_city];
            let end_city = &cities[road.end_city];

            let start_screen_visible = self.is_city_visible_on_screen(start_city);
            let end_screen_visible = self.is_city_visible_on_screen(end_city);

            // Only draw if at least one end is visible
            if start_screen_visible || end_screen_visible {
                let line_width = match road.road_type {
                    RoadType::Highway => 3.0,
                    RoadType::MainRoad => 2.0,
                    RoadType::SecondaryRoad => 1.0,
                };

                // Adjust color based on road condition
                let color = if road.is_mined {
                    [1.0, 0.0, 0.0, 0.8] // Red for mined roads
                } else {
                    let darkness = road.condition;
                    [darkness, darkness, darkness, 1.0]
                };

                line_from_to(
                    color,
                    line_width,
                    [start_city.base.x, start_city.base.y],
                    [end_city.base.x, end_city.base.y],
                    c.transform,
                    g,
                );
            }
        }
    }

    fn draw_city_label(&self, city: &City, c: &Context, g: &mut G2d) {
        let text = Text::new_color([0.0, 0.0, 0.0, 1.0], 12);
        let transform = c.transform
            .trans(city.base.x, city.base.y - 20.0)
            .scale(1.0 / self.zoom, 1.0 / self.zoom); // Scale text inverse to zoom

        text.draw(
            &city.base.name,
            &mut self.font,
            &c.draw_state,
            transform,
            g,
        ).expect("Failed to draw city label");

        // Draw additional city info if zoomed in enough
        if self.zoom > 1.5 {
            let info_text = format!("Pop: {}k", city.population / 1000);
            text.draw(
                &info_text,
                &mut self.font,
                &c.draw_state,
                transform.trans(0.0, 15.0),
                g,
            ).expect("Failed to draw city info");
        }
    }

    fn draw_units(
        &self,
        land_units: &[LandUnit],
        ships: &[Ship],
        air_units: &[AirUnit],
        c: &Context,
        g: &mut G2d
    ) {
        // Draw ships first (lowest layer)
        for ship in ships {
            if self.is_point_visible_on_screen(ship.position_x, ship.position_y) {
                ship.draw(c, g);
            }
        }

        // Draw land units
        for unit in land_units {
            if self.is_point_visible_on_screen(unit.position_x, unit.position_y) {
                unit.draw(c, g);
            }
        }

        // Draw air units last (top layer)
        for unit in air_units {
            if self.is_point_visible_on_screen(unit.position_x, unit.position_y) {
                unit.draw(c, g);
            }
        }
    }

    fn draw_weather_effects(&self, terrain_rules: &TerrainRules, c: &Context, g: &mut G2d) {
        match terrain_rules.current_weather {
            Weather::Rain => self.draw_rain(c, g),
            Weather::Storm => self.draw_storm(c, g),
            Weather::Fog => self.draw_fog(c, g),
            Weather::Clear => {} // No effects for clear weather
        }
    }

    fn draw_rain(&self, c: &Context, g: &mut G2d) {
        // Simple rain effect with diagonal lines
        for i in 0..100 {
            let x = (self.camera_x as i32 + i * 10) as f64;
            let y = (self.camera_y as i32 + i * 10) as f64;
            line(
                [0.5, 0.5, 1.0, 0.3],
                1.0,
                [x, y, x + 5.0, y + 10.0],
                c.transform,
                g
            );
        }
    }

    fn draw_storm(&self, c: &Context, g: &mut G2d) {
        // More intense rain effect plus occasional lightning
        self.draw_rain(c, g);
        
        // Add lightning bolts
        if rand::random::<f64>() < 0.1 {
            let x = self.camera_x + rand::random::<f64>() * self.window_width as f64;
            let y = self.camera_y;
            let points = [
                [x, y],
                [x - 10.0, y + 30.0],
                [x + 5.0, y + 40.0],
                [x - 15.0, y + 70.0],
            ];
            
            for i in 0..points.len()-1 {
                line(
                    [1.0, 1.0, 0.0, 0.8],
                    2.0,
                    [points[i][0], points[i][1], points[i+1][0], points[i+1][1]],
                    c.transform,
                    g
                );
            }
        }
    }

    fn draw_fog(&self, c: &Context, g: &mut G2d) {
        // Draw semi-transparent white overlay
        rectangle(
            [1.0, 1.0, 1.0, 0.3],
            [
                self.camera_x,
                self.camera_y,
                self.window_width as f64,
                self.window_height as f64
            ],
            c.transform,
            g
        );
    }

    fn draw_ui(&self, c: &Context, g: &mut G2d) {
        // Draw mini-map in the corner
        rectangle(
            [0.0, 0.0, 0.0, 0.2],
            [10.0, 10.0, 100.0, 100.0],
            c.transform,
            g
        );

        // Draw zoom level indicator
        let zoom_text = Text::new_color([0.0, 0.0, 0.0, 1.0], 12);
        zoom_text.draw(
            &format!("Zoom: {:.1}x", self.zoom),
            &mut self.font,
            &c.draw_state,
            c.transform.trans(10.0, self.window_height as f64 - 10.0),
            g
        ).expect("Failed to draw zoom text");
    }

    fn is_point_visible_on_screen(&self, x: f64, y: f64) -> bool {
        let screen_x = (x - self.camera_x) * self.zoom + self.window_width as f64 / 2.0;
        let screen_y = (y - self.camera_y) * self.zoom + self.window_height as f64 / 2.0;

        screen_x > 0.0 && screen_x < self.window_width as f64 &&
        screen_y > 0.0 && screen_y < self.window_height as f64
    }
}

// Helper function for drawing lines between points
fn line_from_to(
    color: [f32; 4],
    width: f64,
    from: [f64; 2],
    to: [f64; 2],
    transform: Matrix2d,
    g: &mut G2d
) {
    line(color, width, [from[0], from[1], to[0], to[1]], transform, g);
}