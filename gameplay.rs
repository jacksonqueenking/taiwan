struct Game {
    tiles: Vec<Tile>,
    units: Vec<Unit>,
    ships: Vec<Ship>,
    turn: i32,
    phase: Phase,
    // ... other properties
    
}

impl Game {
    fn next_turn(&mut self) {
        // ... logic for advancing the game by one turn
    }

    fn resolve_combat(&mut self) {
        // ... logic for resolving combat
    }

    // ... other methods
}

fn main() {
    let mut game = Game::new();

    while !game.is_over() {
        game.display();
        game.get_player_input();
        game.next_turn();
    }
}

