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

pub(crate) struct Sentence {
    sentence: String,
}

pub(crate) struct Expression {
    pub dict_word: String,
    pub sentences: Option<Vec<Sentence>>,
    pub sentences_state: ListState,
    pub selected_sentence: Option<usize>,
}

#[derive(Default)]
pub(crate) struct AppState {
    pub expressions: Vec<Expression>,
    pub expressions_state: ListState,
    pub selected_expression: Option<usize>,
    pub select_mode: SelectMode,
    pub err_msg: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self { /* add variables to AppState struct */ }
    pub(crate) fn new() -> Self {
        Self {
            expressions: vec![],
            expressions_state: ListState::default(),
            selected_expression: Some(0),
            select_mode: SelectMode::Expressions,
            err_msg: None,
        }
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
