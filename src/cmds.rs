use crate::app::*;
use arboard::Clipboard;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};

impl AppState {
    pub fn get_current_sentence(&self) -> Option<Sentence> {
        if let Some(exp_index) = self.selected_expression {
            let expression = &self.expressions[exp_index];
            if let Some(sentence_index) = expression.selected_sentence {
                if let Some(sentences) = &expression.sentences {
                    return Some(sentences[sentence_index].clone());
                }
            }
        }
        None
    }

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

    pub async fn fetch_sentences(&mut self) {
        if let Some(i) = self.selected_expression {
            let current_word = self.expressions[i].dict_word.clone();

            match self.fetch_api(current_word.clone(), i).await {
                Ok(_) => {}
                Err(err) => {
                    self.err_msg = Some(format!("Error Fetching {}: {}", &current_word, err));
                    self.info.msg = None;
                }
            }
        }
    }

    pub fn handle_paste(&mut self) {
        let mut clipboard = Clipboard::new().unwrap();
        self.input.text += &clipboard.get_text().unwrap();
        self.input.char_index = self.input.text.len();
    }
}
