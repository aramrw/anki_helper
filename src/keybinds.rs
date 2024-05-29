use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::event::{/*self, Event */ KeyCode, KeyEvent, KeyEventKind};
//use ratatui::prelude::*;
use crate::app::{AppState, SelectMode};
use std::{io, thread, time::*};

impl AppState {
    pub async fn handle_keybinds(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.select_mode {
            SelectMode::Expressions if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter => self.fetch_sentences().await,
                KeyCode::Down => self.select_next_exp(),
                KeyCode::Up => self.select_prev_exp(),
                _ => {}
            },
            SelectMode::Sentences if key.kind == KeyEventKind::Press => match key.code {
                //KeyCode::Down => self.select_next(),
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }
}
