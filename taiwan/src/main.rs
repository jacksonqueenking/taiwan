use piston_window::*;
use taiwan_strait_conflict::game::{Game, GameState, GameActions, GameQueries};
use taiwan_strait_conflict::map::render::MapRenderer;

fn main() {
    // Initialize game state
    let mut game = Game::new();
    
    // Create window
    let mut window: PistonWindow = WindowSettings::new(
        "Taiwan Strait Conflict",
        [640, 480]
    )
    .exit_on_esc(true)
    .build()
    .unwrap();

    // Create renderer
    let mut renderer = MapRenderer::new(
        640,
        480,
        "assets/DejaVuSans.ttf",
        &mut window
    );

    // Game loop
    while let Some(event) = window.next() {
        // Handle input
        renderer.handle_input(&event);

        // Update game state
        if !game.is_over() {
            // Process turns and phases here
        }

        // Render
        window.draw_2d(&event, |c, g, _| {
            // Clear screen
            clear([0.0, 0.0, 0.0, 1.0], g);

            // Draw game state
            renderer.draw_map(
                game.get_tiles(),
                game.get_cities(),
                game.get_roads(),
                game.get_land_units(),
                game.get_ships(),
                game.get_air_units(),
                game.get_terrain_rules(),
                c,
                g
            );
        });
    }
}