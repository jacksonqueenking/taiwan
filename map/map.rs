[dependencies]
piston_window = "0.120.0"

extern crate piston_window;

use piston_window::*;

fn main() {
    let (width, height) = (640, 480);
    let mut window: PistonWindow = WindowSettings::new("2D Map", [width, height])
        .exit_on_esc(true)
        .build()
        .unwrap();

    while let Some(event) = window.next() {
        window.draw_2d(&event, |c, g, _| {
            clear([1.0; 4], g);  // Clear the screen with white color

            // Draw tiles
            
            draw_tiles(&tiles, c, g);

            

            // Draw nodes (for simplicity, drawing them as circles)
            

            ellipse(
                [1.0, 0.0, 0.0, 1.0], // red color
                [320.0, 240.0, 50.0, 50.0], // position and size
                c.transform,
                g,
            );

            // Draw roads (for simplicity, drawing them as lines)

            draw_roads(&roads, c, g);
        });
    }
}

trait Drawable {
    fn draw(&self, c: &Context, g: &mut G2d);
}

//Hexagonal Tiles
enum TileTypes{
    WaterTile,
    CoastTile,
    LandTile,
    MountainTile,
}

struct HexTile {
    tile_type: TileType,
    center_x: f64,
    center_y: f64,
    size: f64,  // Distance from center to a corner
    coordinates: (i32, i32),
}
// Drawing the Hexagonal Tile:
// To draw a hexagon, you'll need to calculate the positions of its six corners based on its center and size.

fn generate_tiles(hex_size: f64, tiles_x: usize, tiles_y: usize) -> Vec<HexTile> {
    let mut tiles = Vec::new();
    let spacing_x = hex_size * 1.732;  // 1.732 is 2 times the sine of 60 degrees
    let spacing_y = hex_size * 1.5;

    for i in 0..tiles_x {
        for j in 0..tiles_y {
            let offset_x = if j % 2 == 0 { 0.0 } else { hex_size * 0.866 };  // Offset every other row
            let hex = HexTile {
                center_x: i as f64 * spacing_x + offset_x,
                center_y: j as f64 * spacing_y,
                size: hex_size,
                coordinates: (i, j),
            };
            tiles.push(hex);
        }
    }

    tiles
}

let hex_size = 50.0;
let tiles_x = 10;
let tiles_y = 10;
let tiles = generate_tiles(hex_size, tiles_x, tiles_y);

impl Drawable for HexTile {
    fn draw(&self, c: Context, g: &mut G2d) {
        let points = [
            [self.center_x, self.center_y - self.size],  // Top
            [self.center_x + self.size * 0.866, self.center_y - self.size * 0.5],  // Top right
            [self.center_x + self.size * 0.866, self.center_y + self.size * 0.5],  // Bottom right
            [self.center_x, self.center_y + self.size],  // Bottom
            [self.center_x - self.size * 0.866, self.center_y + self.size * 0.5],  // Bottom left
            [self.center_x - self.size * 0.866, self.center_y - self.size * 0.5],  // Top left
        ];
        polygon([1.0, 0.0, 0.0, 1.0], &points, c.transform, g);  // Drawing the hexagon with a red color
    }
}

fn draw_tiles(tiles: &[HexTile], c: Context, g: &mut G2d) {
    for tile in tiles {
        tile.draw(c, g);
    }
}

//The magic number 0.866 is derived from the sine and cosine of 30 degrees, which are needed to calculate the hexagon's corner positions.

//Rectangles?
            //for i in 0..10 {
                //for j in 0..10 {
                    //rectangle(
                        //[0.5, 0.5, 0.5, 1.0], // gray color
                        //[(i * 64) as f64, (j * 48) as f64, 64.0, 48.0], // position and size
                        //c.transform,
                        //g,
                    //);
                    
                //}
            //}

//Hubs
enum HubType {
    CityHub(City),
    PortHub(Port),
    AirBaseHub(AirBase),
    //Other Hub types
}

struct Arsenal {
    sam_missiles: i32
    //Numbers of all the different types of missiles, supplies, and ammo.
}

struct Hub {
    name: String,
    x: f64,
    y: f64,
    storage: i32, //in tonnes
    //Other generic Hub attributes
}

//Cities

struct City {
    base: Hub,
    significance: f64,  // A value between 0.0 (least significant) to 1.0 (most significant)
    population: i32,
    //Other City attributes
}

let cities = vec![
    City { base.x: 100.0, base.y: 100.0, base.name: "Tainan", significance: 0.5, population: 1900000},
    City { base.x: 540.0, base.y: 100.0, base.name: "Kaohsiung", significance: 0.7, population: 2700000},
    City { base.x: 320.0, base.y: 380.0, base.name: "Taipei", significance: 1.0, population: 2600000 },
    // ... add more cities as needed
];


impl Drawable for City {
    fn draw(&self, c: Context, g: &mut G2d) {
        let radius = self.significance * 25.0 + 5.0;  // Example calculation
        ellipse(
            [1.0, 0.0, 0.0, 1.0],  // Red color
            [self.base.x - radius, self.base.y - radius, 2.0 * radius, 2.0 * radius],
            c.transform,
            g,
        );
    }
}

fn draw_cities(cities: &[City], c: Context, g: &mut G2d) {
    // Draw cities
    for city in cities.iter() {
        city.draw(c, g);
    }

}

//Ports

struct Port {
    base: Hub,
    base_capacity: i32,
    condition: f32,
    is_blockaded: bool,
}


struct AirBase {
    base: Hub,
    base_capacity: i32,
    condition: f32,
}
//Roads


while let Some(event) = window.next() {
    window.draw_2d(&event, |c, g, _| {
        clear([1.0; 4], g);  // Clear the screen with white color
        draw_map(&cities, c, g);
    });
}
//This code will draw nodes for each city and roads connecting every pair of cities. The size of the nodes and the thickness of the roads will vary based on the significance of the cities. Adjust the calculations for radius and thickness as needed to get the desired visual effect.

enum RoadType {
    Highway,
    MainRoad,
    SecondaryRoad,
    // ... other types ...
}

struct Road {
    start_city: usize,  // Index to a City in a cities Vec
    end_city: usize,    // Index to a City in a cities Vec
    condition: f64,     // A value between 0.0 (destroyed) to 1.0 (perfect condition)
    is_mined: bool,
    road_type: RoadType,
    // ... add other attributes as needed
}

let mut roads = vec![
    Road { start_city: 0, end_city: 1, condition: .9, is_mined: false, road_type: Highway},
    Road { start_city: 1, end_city: 2, condition: .8, is_mined: false, road_type: MainRoad},
    Road { start_city: 0, end_city: 2, condition: 1.0, is_mined: true, road_type: SecondaryRoad},
    // ... add more roads as needed
];

impl Road {
    fn capacity(&self) -> f64 {
        let base_capacity = match self.road_type {
            RoadType::Highway => 1000.0,       // Example base capacity for highways
            RoadType::MainRoad => 700.0,      // Example base capacity for main roads
            RoadType::SecondaryRoad => 400.0, // Example base capacity for secondary roads
            // ... other types ...
        };

        // Modify the base capacity based on the road's condition
        base_capacity * self.condition
    }
}

fn draw_roads(cities: &[City], roads: &[Road], c: Context, g: &mut G2d) {
    // Draw roads
    for road in roads.iter() {
        let start = &cities[road.start_city];
        let end = &cities[road.end_city];
        let thickness = match self.road_type {
            RoadType::Highway => 10,       // Example base capacity for highways
            RoadType::MainRoad => 7,      // Example base capacity for main roads
            RoadType::SecondaryRoad => 4, // Example base capacity for secondary roads
            // ... other types ...
        };  
        let color = if road.is_mined {
            [0.8, 0.0, 0.0, 1.0]  // Darker red for mined roads
        } else {
            [0.0, 0.0, 0.0, 1.0]  // Black color
        };
        line(
            color,
            thickness,
            [start.x, start.y, end.x, end.y],
            c.transform,
            g,
        );
    }
}

fn bomb_road(road: &mut Road, damage: f64) {
    road.condition -= damage;
    if road.condition < 0.0 {
        road.condition = 0.0;
    }
}

fn mine_road(road: &mut Road) {
    road.is_mined = true;
}

// ... add other interaction functions as needed


while let Some(event) = window.next() {
    window.draw_2d(&event, |c, g, _| {
        clear([1.0; 4], g);  // Clear the screen with white color
        draw_map(&cities, &roads, c, g);
    });

    // Example interactions:
    if some_bombing_event {  // Replace with actual game event logic
        bomb_road(&mut roads[0], 0.2);
    }
    if some_mining_event {  // Replace with actual game event logic
        mine_road(&mut roads[1]);
    }
}
//This setup allows you to have roads with different conditions and states, and you can interact with them based on game events. Adjust the logic and attributes as needed to fit your game's requirements.

//while let Some(event) = window.next() {
    //window.draw_2d(&event, |c, g, _| {
        //clear([1.0; 4], g);  // Clear the screen with white color
        //draw_map(c, g);
    //});
//}

// Drawing City Names:
// To display the names of major cities automatically:

impl City {
    fn draw_name(&self, glyphs: &mut Glyphs, c: Context, g: &mut G2d) {
        if self.significance > 0.5 {  // Example threshold for "major" cities
            let transform = c.transform.trans(self.x, self.y - 10.0);  // Position slightly above the city
            text::Text::new_color([0.0, 0.0, 0.0, 1.0], 20).draw(
                &self.name,
                glyphs,
                &c.draw_state,
                transform,
                g
            ).unwrap();
        }
    }
}
// Hover Interaction:
// To display names when hovering over cities or tiles:

fn city_or_tile_at_point(cities: &[City], x: f64, y: f64) -> Option<&City> {
    for city in cities.iter() {
        let distance = ((city.x - x).powi(2) + (city.y - y).powi(2)).sqrt();
        if distance < SOME_THRESHOLD {  // Define a suitable threshold for hover detection
            return Some(city);
        }
    }
    None
}

let mut hovered_city: Option<&City> = None;

while let Some(event) = window.next() {
    window.draw_2d(&event, |c, g, glyphs| {
        clear([1.0; 4], g);  // Clear the screen with white color

        for city in &cities {
            city.draw(c, g);
            city.draw_name(glyphs, c, g);
        }

        if let Some(city) = &hovered_city {
            // Draw the name of the hovered city, possibly with a different style or color
        }
    });

    // Handle mouse move events to detect hovering
    if let Some(e) = event.mouse_cursor_args() {
        hovered_city = city_or_tile_at_point(&cities, e[0], e[1]);
    }
}

