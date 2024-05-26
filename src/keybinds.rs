use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyEvent};
//use ratatui::prelude::*;
use std::io;

use crate::app::AppState;

impl AppState {
    pub fn handle_keybinds(&mut self, key: KeyEvent) -> io::Result<()> {
                match key.code {
                    // add keybinds here
                    _ => {}
        }

        Ok(())
    }
}
