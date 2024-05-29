use crossterm::event::{/*self, Event */ KeyCode, KeyEvent, KeyEventKind};
//use ratatui::prelude::*;
use crate::app::{AppState, SelectMode};
use std::{io, thread, time::*};
use std::io;

impl AppState {
    pub async fn handle_keybinds(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.select_mode {
            SelectMode::Expressions if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Enter => {
                    self.select_mode = SelectMode::Sentences;
                    self.fetch_sentences().await
                }
                KeyCode::Down => self.select_next_exp(),
                KeyCode::Up => self.select_prev_exp(),
                _ => {}
            },
            SelectMode::Sentences if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc => self.select_mode = SelectMode::Expressions,
                KeyCode::Up => self.select_prev_sentence(),
                KeyCode::Down => self.select_next_sentence(),
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }

    pub async fn fetch_sentences(&mut self) {
        if let Some(i) = self.selected_expression {
            let current_word = self.expressions[i].dict_word.clone();
            let instant = Instant::now();
            match self.fetch_api(current_word.clone(), i).await {
                Ok(_) => {
                    self.err_msg = None;
                    self.info.msg = format!(
                        "Fetched sentences for {} in {}s",
                        &current_word,
                        instant.elapsed().as_secs()
                    )
                    .into()
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

                }
                Err(err) => {
                    self.err_msg = Some(format!("Error Fetching {}: {}", &current_word, err));
                    self.info.msg = None;
                }
            }
        }
    }
}
