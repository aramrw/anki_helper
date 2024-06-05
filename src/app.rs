use anki_bridge::AnkiClient;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use std::io;
//use std::sync::{Arc, Mutex};

#[derive(Default)]
pub enum Pages {
    #[default]
    Main,
    Help,
    Split
}

#[derive(Default)]
pub enum SelectMode {
    #[default]
    Expressions,
    Sentences,
    Input,
}

#[derive(Default, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Search,
    //Grep,
    FindID,
}

#[derive(Default)]
pub struct InputBox {
    pub text: String,
    pub char_index: usize,
    pub mode: InputMode,
}

#[derive(Default)]
pub struct Info {
    pub msg: Option<String>,
    pub found: Option<usize>,
}

#[derive(Clone, Debug)]
pub(crate) struct Sentence {
    pub sentence: String,
    pub audio_url: String,
    pub audio_data: Option<Vec<u8>>,
    pub img_url: Option<String>,
    pub media_title: String,
}

pub(crate) struct Expression {
    pub dict_word: String,
    pub readings: Vec<String>,
    pub sentences: Option<Vec<Sentence>>,
    pub sentences_state: ListState,
    pub selected_sentence: Option<usize>,
    pub definitions: Vec<String>,
    pub exact_search: bool,
}

#[derive(Default)]
pub(crate) struct AppState {
    pub expressions: Vec<Expression>,
    pub expressions_state: ListState,
    pub selected_expression: Option<usize>,
    pub select_mode: SelectMode,
    pub err_msg: Option<String>,
    pub info: Info,
    pub input: InputBox,
    pub client: AnkiClient<'static>
}

impl AppState {
    pub(crate) fn new() -> Self {
        Self {
            expressions: Vec::new(),
            expressions_state: ListState::default(),
            selected_expression: Some(0),
            select_mode: SelectMode::Expressions,
            err_msg: None,
            info: Info::default(),
            input: InputBox::default(),
            client: AnkiClient::default(),
        }
    }
}

impl AppState {
    pub async fn run( &mut self, mut term: Terminal<impl Backend>) -> io::Result<()> {
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
                self.handle_keybinds(key).await?
            }
        }
    }

    fn draw(&mut self, term: &mut Terminal<impl Backend>) -> io::Result<()> {
        term.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn update_error_msg(&mut self, title: &str, err: String) {
        self.err_msg = Some(format!("{}: {}", title, err));
    }
}

impl Sentence {
    pub fn from(
        sentence: &str,
        audio_url: &str,
        audio_data: Option<Vec<u8>>,
        img_url: Option<String>,
        media_title: &str,
    ) -> Self {
        Self {
            sentence: sentence.to_string(),
            audio_url: audio_url.to_string(),
            audio_data,
            img_url,
            media_title: media_title.to_string(),
        }
    }

    pub fn to_list_item(&self, i: usize) -> ListItem {
        let line = Line::styled(format!("{}. {}", i, self.sentence), Color::White);
        ListItem::new(line)
    }
}

impl Expression {
    pub fn from(dict_word: String, reading: Option<Vec<String>>, sentences: Option<Vec<Sentence>>) -> Self {
        Self {
            dict_word,
            readings: Vec::new(),
            sentences,
            sentences_state: ListState::default(),
            selected_sentence: Some(0),
            definitions: Vec::new(),
            exact_search: false,
        }
    }

    pub fn to_list_item(&self, i: usize) -> ListItem {
        let line = Line::styled(format!("{}. {}", i, self.dict_word), Color::White);
        ListItem::new(line)
    }
}
