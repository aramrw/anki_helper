use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use std::io;

pub struct AppState {}
#[derive(Default)]
pub enum SelectMode {
    #[default]
    Expressions,
    Sentences,
}


impl AppState {
    pub fn new() -> Self {
        Self { /* add variables to AppState struct */ }
    }
}

impl AppState {
    pub fn run(&mut self, mut term: Terminal<impl Backend>) -> io::Result<()> {
        loop {
            self.draw(&mut term)?;

            // handle key events & keybindings
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('Q') {
                    return Ok(());
                }
                // src/keybinds.rs
                self.handle_keybinds(key)?
            }
        }
    }

    fn draw(&mut self, term: &mut Terminal<impl Backend>) -> io::Result<()> {
        term.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}
