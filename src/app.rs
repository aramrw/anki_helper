use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use std::io;

#[derive(Default)]
pub enum SelectMode {
    #[default]
    Expressions,
    Sentences,
}

#[derive(Default)]
pub struct Info {
    pub msg: Option<String>,
    pub found: Option<usize>,
}

#[derive(Clone)]
pub(crate) struct Sentence {
    pub sentence: String,
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
    pub info: Info,
}

impl AppState {
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
    pub async fn run(&mut self, mut term: Terminal<impl Backend>) -> io::Result<()> {
        match self.read_words_file() {
            Ok(_) => {}
            Err(err) => self.err_msg = Some(format!("Error Reading `words.txt`: {}", err)),
        }

        loop {
            self.draw(&mut term)?;

            // handle key events & keybindings
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('Q') {
                    return Ok(());
                }
                // src/keybinds.rs
                self.handle_keybinds(key)?
                self.handle_keybinds(key).await?
            }
        }
    }

    fn draw(&mut self, term: &mut Terminal<impl Backend>) -> io::Result<()> {
        term.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }
}

impl Expression {
    pub fn from(dict_word: String, sentences: Option<Vec<Sentence>>) -> Self {
        Self {
            dict_word,
            sentences,
            sentences_state: ListState::default(),
            selected_sentence: Some(0),
        }
    }

    pub fn to_list_item(&self, i: usize) -> ListItem {
        let line = Line::styled(format!("{}. {}", i, self.dict_word), Color::LightBlue);
        ListItem::new(line)
    }
}
