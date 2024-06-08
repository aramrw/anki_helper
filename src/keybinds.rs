use crossterm::event::{/*self, Event */ KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
//use ratatui::prelude::*;
use crate::app::{AppState, Pages, SelectMode};
//use crate::audio::{decode_audio_bytes, trim_samples_from_start};
use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, ListState, Paragraph},
};
use std::io;

#[derive(Default, Debug, PartialEq)]
pub enum KeybindSections {
    #[default]
    Expressions,
    Sentences,
    Input,
}

#[derive(Default)]
pub struct Keybinds {
    pub titles: Vec<String>,
    pub selected_kb: usize,
    pub state: ListState,
    pub abouts: Vec<String>,
    pub exp_titles: Vec<String>,
    pub sent_titles: Vec<String>,
    pub input_titles: Vec<String>,
    pub exp_state: ListState,
    pub sent_state: ListState,
    pub input_state: ListState,
    pub exp_abouts: Vec<String>,
    pub sent_abouts: Vec<String>,
    pub input_abouts: Vec<String>,
    pub selected_section: KeybindSections,
}

impl AppState {
    pub async fn handle_keybinds(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.select_mode {
            SelectMode::Expressions if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('I') => self.select_mode = SelectMode::Input,
                KeyCode::Char('Y') => self.handle_copy_to_input(),
                KeyCode::Char('D') => {
                    if let Some(i) = self.selected_expression {
                        let current_wrd = &self.expressions[i].dict_word.clone();
                        match self.delete_word_from_file(current_wrd) {
                            Ok(_) => {
                                self.info.msg =
                                    format!("Deleted: {} from words.txt", &current_wrd).into()
        if self.expressions_state.selected().is_none() {
            self.expressions_state.select(Some(0));
        }
        match self.selected_page {
            Pages::Main => match self.select_mode {
                SelectMode::Expressions if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('I') => self.select_mode = SelectMode::Input,
                    KeyCode::Char('Y') => self.handle_copy_to_input(),
                    KeyCode::Char('D') => {
                        if let Some(i) = self.selected_expression {
                            let current_wrd = &self.expressions[i].dict_word.clone();
                            match self.delete_word_from_file(current_wrd) {
                                Ok(_) => {
                                    self.info.msg =
                                        format!("Deleted: {} from words.txt", &current_wrd).into()
                                }
                                Err(err) => self.update_error_msg(
                                    "Err Deleting {} from words.txt: {}",
                                    err.to_string(),
                                ),
                            }
                            Err(err) => self.update_error_msg(
                                "Err Deleting {} from words.txt: {}",
                                err.to_string(),
                            ),
                        }
                    }
                }
                KeyCode::Enter if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(i) = self.selected_expression {
                        self.expressions[i].sentences = None;
                        self.expressions[i].exact_search = false;
                        self.select_mode = SelectMode::Sentences;
                        if self.expressions[i].sentences.is_none() {
                            self.fetch_sentences().await;
                    KeyCode::Enter if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(i) = self.selected_expression {
                            self.expressions[i].sentences = None;
                            self.expressions[i].exact_search = false;
                            self.select_mode = SelectMode::Sentences;
                            if self.expressions[i].sentences.is_none() {
                                self.expressions[i].sentences_state.select(Some(0));
                                self.fetch_sentences().await;
                            }
                        }
                    }
                }
                KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(i) = self.selected_expression {
                        self.expressions[i].sentences = None;
                        self.expressions[i].exact_search = true;
                        self.select_mode = SelectMode::Sentences;
                        if self.expressions[i].sentences.is_none() {
                            self.fetch_sentences().await;
                    KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if let Some(i) = self.selected_expression {
                            self.expressions[i].sentences = None;
                            self.expressions[i].exact_search = true;
                            self.select_mode = SelectMode::Sentences;
                            if self.expressions[i].sentences.is_none() {
                                self.expressions[i].sentences_state.select(Some(0));
                                self.fetch_sentences().await;
                            }
                        }
                    }
                }
                KeyCode::Down => self.select_next_exp(),
                KeyCode::Up => self.select_prev_exp(),
                _ => {}
            },
            SelectMode::Sentences if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('L') => self.open_website_link(),
                // KeyCode::Right => {
                //     if self.selected_page == Pages::Splice {
                //         if let Some(i) = self.selected_expression {
                //             let current_exp = &self.expressions[i];
                //             if let Some(sentences) = &current_exp.sentences {
                //                 let sent_index = current_exp.selected_sentence.unwrap();
                //                 let audio_data = self.expressions[i].sentences.as_mut().unwrap()
                //                     [sent_index]
                //                     .audio_data
                //                     .as_mut()
                //                     .unwrap();
                //
                //                 let (samples, sample_rate, channels) =
                //                     decode_audio_bytes(audio_data.to_vec()).unwrap();
                //
                //                 let result =
                //                     trim_samples_from_start(samples, sample_rate, channels);
                //
                //                 let trimmed: Vec<u8> = result
                //                     .iter()
                //                     .flat_map(|&sample| {
                //                         let bytes = sample.to_le_bytes();
                //                         vec![bytes[0], bytes[1]]
                //                     })
                //                     .collect();
                //
                //                 *audio_data = trimmed;
                //             }
                //         }
                //     }
                // }
                // KeyCode::Char('S') => {
                //     if let Some(sentence) = self.get_current_sentence() {
                //         if sentence.audio_url.is_some() {
                //             self.push_audio().await.unwrap();
                //             self.selected_page = Pages::Splice;
                //         } else {
                //             self.update_error_msg(
                //                 "Cannot Splice Audio",
                //                 "Sentence does not have an audio file.".to_string(),
                //             );
                //         }
                //     }
                // }
                KeyCode::Char('P') => {
                    if let Err(err) = self.play_audio().await {
                        self.update_error_msg("Error Playing Audio", err.to_string());
                    }
                }
                KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.update_last_anki_card().await
                }
                KeyCode::Esc => self.reset_sentences_index(),
                KeyCode::Up => {
                    if self.selected_page == Pages::Splice {
                        return Ok(());
                    }
                    self.select_prev_sentence()
                }
                KeyCode::Down => {
                    if self.selected_page == Pages::Splice {
                        return Ok(());
                    }
                    self.select_next_sentence()
                }
                _ => {}
            },
            SelectMode::Input if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('P') => self.handle_paste(),
                KeyCode::Esc => self.select_mode = SelectMode::Expressions,
                KeyCode::Enter => self.confirm_search_query().await,
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Char(input_char) => self.enter_char(input_char),
                _ => {}
            },
            _ => match key.code {
                KeyCode::Char('H') => self.selected_page = Pages::Help,
                KeyCode::Char('M') => self.selected_page = Pages::Main,
                KeyCode::Char('R') => self.restart_program(),
                _ => {}
            },
        }

        Ok(())
    }

    pub fn reset_sentences_index(&mut self) {
        if let Some(i) = self.selected_expression {
            self.expressions[i].selected_sentence = None;
            self.expressions[i].sentences_state.select(None);
        }
        self.select_mode = SelectMode::Expressions
    }

    pub fn select_prev_sentence(&mut self) {
        if let Some(exp_index) = self.selected_expression {
            let selected_exp = &self.expressions[exp_index];
            if let Some(sentences) = &selected_exp.sentences {
                let sentence_index = match selected_exp.sentences_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            sentences.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => selected_exp.selected_sentence.unwrap_or(0),
                };
                self.expressions[exp_index].selected_sentence = Some(sentence_index);
                self.expressions[exp_index]
                    .sentences_state
                    .select(Some(sentence_index));
            }
        }
    }

    pub fn select_next_sentence(&mut self) {
        if let Some(exp_index) = self.selected_expression {
            let selected_exp = &self.expressions[exp_index];
            if let Some(sentences) = &selected_exp.sentences {
                if sentences.is_empty() {
                    return;
                }
                let sentence_index = match selected_exp.sentences_state.selected() {
                    Some(i) => {
                        if i == sentences.len().saturating_sub(1) {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => selected_exp.selected_sentence.unwrap_or(0),
                };
                self.expressions[exp_index].selected_sentence = Some(sentence_index);
                self.expressions[exp_index]
                    .sentences_state
                    .select(Some(sentence_index));
            }
        }
    }
    pub fn select_prev_exp(&mut self) {
        let i = match self.expressions_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.expressions.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => self.selected_expression.unwrap_or(0),
        };

        self.selected_expression = Some(i);
        self.expressions_state.select(Some(i));
    }

    pub fn select_next_exp(&mut self) {
        let i = match self.expressions_state.selected() {
            Some(i) => {
                if i == self.expressions.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => self.selected_expression.unwrap_or(0),
        };

        self.selected_expression = Some(i);
        self.expressions_state.select(Some(i));
    }

    pub fn rend_main_keybinds(&self, area: Rect, buf: &mut Buffer) {
        let (msg, style) = (
            vec![
                "<H> ".green().bold(),
                "Help Page ".into(),
                "<R> ".red().bold(),
                "Restart Program ".into(),
                // "<S> ".light_blue().bold(),
                // "Audio Cutter".into(),
            ],
            Style::default(),
        );

        let text = Text::from(Line::from(msg).patch_style(style));
        Paragraph::new(text)
            .block(Block::bordered().title("Keybinds"))
            .style(Style::default().bold())
            .centered()
            .render(area, buf);
    }

    pub fn rend_keybinds(&mut self, area: Rect, buf: &mut Buffer) {
        let kb_titles: Vec<ListItem> = self
            .keybinds
            .titles
            .iter()
            .enumerate()
            .map(|(i, kb)| Keybinds::to_list_item(kb, i))
            .collect();

        let kbs = List::new(kb_titles)
            .block(
                Block::bordered()
                    .title("Expressions")
                    .style(Style::default().yellow().bold()),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::REVERSED)
                    .fg(Color::White),
            );

        StatefulWidget::render(kbs, area, buf, &mut self.keybinds.state);
    }

    pub fn rend_about(&mut self, area: Rect, buf: &mut Buffer) {
        let i = self.keybinds.selected_kb;
        let (about, style) = (&self.keybinds.abouts[i], Style::default().white());
        let text = Text::from(Line::from(about.clone()).patch_style(style));
        Paragraph::new(text)
            .block(
                Block::bordered()
                    .title("About")
                    .style(Style::default().yellow().bold()),
            )
            .render(area, buf);
    }
}

impl Keybinds {
    pub fn to_list_item(text: &str, i: usize) -> ListItem {
        let line = Line::styled(format!("{}. {}", i, text), Color::White);
        ListItem::new(line)
    }

    pub fn new() -> Self {
        Self {
            titles: ["<I>", "<Y>", "<D>"]
                .iter()
                .map(|kb| kb.to_string())
                .collect(),
            selected_kb: 0,
            state: ListState::default(),
            abouts: [
                "Selects the input box.",
                "Copies selected Expression to Input Box.",
                "Deletes selected Expression",
            ]
            .iter()
            .map(|ab| ab.to_string())
            .collect(),
        }
    }
}
