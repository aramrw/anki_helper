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
                }
                Err(err) => {
                    self.err_msg = Some(format!("Error Fetching {}: {}", &current_word, err));
                    self.info.msg = None;
                }
            }
        }
    }
}
