use crate::keybinds::Keybinds;
use anki_bridge::AnkiClient;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{prelude::*, widgets::*};
use std::io;
//use std::sync::{Arc, Mutex};

#[derive(Default, PartialEq)]
pub enum Pages {
    #[default]
    Main,
    Help,
    Splice,
}

#[derive(Default, PartialEq)]
pub enum SelectMode {
    #[default]
    Expressions,
    Sentences,
    Input,
    Ntbm,
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Sentence {
    pub sentence: String,
    pub audio_url: Option<String>,
    pub audio_data: Option<Vec<u8>>,
    pub img_url: Option<String>,
    pub media_title: String,
    pub wbst_link: String,
    pub parent_expression: Expression,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Expression {
    pub dict_word: String,
    pub readings: Vec<String>,
    pub sentences: Option<Vec<Sentence>>,
    pub sentences_state: ListState,
    pub selected_sentence: Option<usize>,
    pub definitions: Vec<String>,
    pub exact_search: bool,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct NotesToBeCreated {
    pub sentences: Vec<Sentence>,
    pub state: ListState,
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
    pub keybinds: Keybinds,
    pub selected_page: Pages,
    pub notes_to_be_created: NotesToBeCreated,
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
            keybinds: Keybinds::new(),
            selected_page: Pages::Main,
            notes_to_be_created: NotesToBeCreated::default(),
        }
    }
}

impl AppState {
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
                if self.expressions_state.selected().is_none() {
                    self.expressions_state.select(Some(0));
                }
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
        audio_url: Option<String>,
        audio_data: Option<Vec<u8>>,
        img_url: Option<String>,
        media_title: &str,
        wbst_link: &str,
        parent_expression: &Expression,
    ) -> Self {
        Self {
            sentence: sentence.to_string(),
            audio_url,
            audio_data,
            img_url,
            media_title: media_title.to_string(),
            wbst_link: wbst_link.to_string(),
            parent_expression: parent_expression.clone(),
        }
    }
    pub fn to_be_created_list_item(&self, sentence: &Sentence, i: usize) -> ListItem {
        let mixed_line = Line::from(vec![
            //Span::styled("|", Color::Green),
            Span::styled(i.to_string(), Style::default().yellow()),
            Span::styled(". ", Color::Green),
            Span::styled(sentence.sentence.clone(), Color::White),
        ]);

        ListItem::new(mixed_line)
    }
}

impl Expression {
    pub fn from(
        dict_word: String,
        _reading: Option<Vec<String>>,
        sentences: Option<Vec<Sentence>>,
    ) -> Self {
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
        let mixed_line = Line::from(vec![
            //Span::styled("|", Color::Green),
            Span::styled(i.to_string(), Style::default().yellow()),
            Span::styled(". ", Color::Green),
            Span::styled(&self.dict_word, Color::White),
        ]);

        ListItem::new(mixed_line)
    }
}
