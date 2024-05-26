use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use std::io;

pub struct AppState {}

impl AppState {
    pub fn new() -> Self {
        Self { /* add variables to AppState struct */ }
    }
}


    fn draw(&mut self, term: &mut Terminal<impl Backend>) -> io::Result<()> {
        term.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}
