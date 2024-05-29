use std::fs::File;
use std::io::{self, prelude::*, BufReader};

use crate::app::*;

impl AppState {
    pub fn read_words_file(&mut self) -> io::Result<()> {
        let file = File::open("words.txt")?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;

            for word in line.split_whitespace() {
                self.expressions
                    .push(Expression::from(word.to_string(), None));
            }
        }

        Ok(())
    }

    pub fn select_prev_exp(&mut self) {
        let i = match self.expressions_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.expressions.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.selected_expression.unwrap_or(0),
        };
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

        self.expressions_state.select(Some(i));
    }
}
