use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyEvent};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
//use ratatui::prelude::*;
use std::io;

use crate::app::{AppState, SelectMode};

impl AppState {
    pub fn handle_keybinds(&mut self, key: KeyEvent) -> io::Result<()> {
                match key.code {
                    // add keybinds here
                    _ => {}
        }

        Ok(())
    }
}
