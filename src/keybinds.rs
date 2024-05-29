use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
//use ratatui::prelude::*;
use std::io;

use crate::app::{AppState, SelectMode};

impl AppState {
    pub fn handle_keybinds(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.select_mode {
            SelectMode::Expressions if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Down => self.select_next_exp(),
                KeyCode::Up => self.select_prev_exp(),
                _ => {},
            }
            SelectMode::Sentences if key.kind == KeyEventKind::Press => match key.code {
                //KeyCode::Down => self.select_next(),
                _ => {},
            }
            _ => {},
        } 

        Ok(())
    }
}
