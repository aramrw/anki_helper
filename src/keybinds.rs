use crossterm::event::{/*self, Event */ KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
//use ratatui::prelude::*;
use crate::app::{AppState, SelectMode};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};
use std::io;

impl AppState {
    pub async fn handle_keybinds(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.select_mode {
            SelectMode::Expressions if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('I') => self.select_mode = SelectMode::Input,
                KeyCode::Char('Y') => self.handle_copy_to_input(),
                KeyCode::Enter if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(i) = self.selected_expression {
                        self.expressions[i].sentences = None;
                        self.expressions[i].exact_search = false;
                        self.select_mode = SelectMode::Sentences;
                        if self.expressions[i].sentences.is_none() {
                            self.fetch_sentences().await;
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
                        }
                    }
                }
                KeyCode::Down => self.select_next_exp(),
                KeyCode::Up => self.select_prev_exp(),
                _ => {}
            },
            SelectMode::Sentences if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('P') => {
                    if let Err(err) = self.play_audio().await {
                        self.update_error_msg("Error Playing Audio", err.to_string());
                    }
                }
                KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.update_last_anki_card().await
                }
                KeyCode::Esc => self.reset_sentences_index(),
                KeyCode::Up => self.select_prev_sentence(),
                KeyCode::Down => self.select_next_sentence(),
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
            _ => {}
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

    pub fn rend_keybinds(&self, area: Rect, buf: &mut Buffer) {
        let (msg, style) = match self.select_mode {
            SelectMode::Expressions => (
                vec![
                    "(".into(),
                    "<Up> ".light_yellow().bold(),
                    "Prev ".yellow(),
                    "| ".into(),
                    "<Down> ".light_yellow().bold(),
                    "Next".yellow(),
                    ") ".into(),
                    "<Enter> ".light_green().bold(),
                    "Sentence Selection ".green(),
                ],
                Style::default(),
            ),
            SelectMode::Sentences => (
                vec![
                    "<Esc> ".light_red().bold(),
                    "Back ".red(),
                    "(".into(),
                    "<Up> ".light_yellow().bold(),
                    "Prev ".yellow(),
                    "| ".into(),
                    "<Down> ".light_yellow().bold(),
                    "Next ".yellow(),
                    ") ".into(),
                    "<P> ".light_blue().bold(),
                    "Play Audio ".blue(),
                    "<C> ".light_green().bold(),
                    "Update Card".green(),
                ],
                Style::default(),
            ),
            SelectMode::Input => (
                vec![
                    "<Esc> ".light_red().bold(),
                    "Back ".red(),
                    "(".into(),
                    "<Left> ".light_yellow().bold(),
                    "Prev ".yellow(),
                    "| ".into(),
                    "<Right> ".light_yellow().bold(),
                    "Next ".yellow(),
                    ") ".into(),
                    "<P> ".light_blue().bold(),
                    "Paste ".into(),
                    "<Enter> ".light_blue().bold(),
                    "Confirm ".into(),
                ],
                Style::default(),
            ),
        };

        let text = Text::from(Line::from(msg).patch_style(style));
        Paragraph::new(text)
            .block(Block::bordered().title("Keybinds"))
            .centered()
            .render(area, buf);
    }
}
