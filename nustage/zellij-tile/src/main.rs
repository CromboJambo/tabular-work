//! Entry point for Nustage Zellij Tile
//!
//! This binary provides the main loop for the Zellij tile interface.

use nustage_zellij_tile::NustageTile;
use std::time::Duration;
use zellij_tile_utils::{Event, EventWrapper, Key, Pane, Tile, TileConfig};

fn main() {
    let mut tile = NustageTile::new();
    let mut last_tick = 0;

    loop {
        // Process events
        tile.process_event(EventWrapper::Event(Event::Tick));

        // Render the tile
        let config = tile.config();
        let mut pane = Pane::new(config);
        tile.render(&pane);

        // Handle keyboard input
        if let Some(input) = zellij_tile_utils::read_key() {
            match input {
                EventWrapper::Event(Event::Key(key)) => {
                    tile.process_event(EventWrapper::Event(Event::Key(key)));
                }
                _ => {}
            }
        }

        // Add a small delay to prevent CPU hogging
        std::thread::sleep(Duration::from_millis(10));
    }
}
