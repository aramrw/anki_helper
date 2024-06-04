use crate::app::*;
use arboard::Clipboard;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*, BufReader, BufWriter};
use std::time::Instant;

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

    pub fn delete_word_from_file(&mut self, to_del_word: &str) -> io::Result<()> {
        let file = File::open("words.txt")?;
        let reader = BufReader::new(file);

        let temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("temp.txt")?;
        let mut writer = BufWriter::new(temp_file);

        for line in reader.lines() {
            let line = line?;
            let new_line = if line.contains(to_del_word) {
                line.replace(to_del_word, "")
            } else {
                line
            };
            writeln!(writer, "{}", new_line)?;
        }

        std::fs::remove_file("words.txt")?;
        std::fs::rename("temp.txt", "words.txt")?;
        if let Some(i) = self.selected_expression {
            if self.expressions[i].dict_word.trim() == to_del_word.trim() {
                self.expressions.remove(i);
            }
        }

        Ok(())
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

            let format_url = if self.expressions[i].exact_search {
                format!(
                "https://api.immersionkit.com/look_up_dictionary?keyword={}&exact=true&sort=shortness",
                &current_word
            )
            } else {
                format!(
                    "https://api.immersionkit.com/look_up_dictionary?keyword={}&sort=shortness",
                    &current_word
                )
            };

            match self.fetch_api(current_word.clone(), i, format_url).await {
                Ok(_) => {}
                Err(err) => {
                    self.err_msg = Some(format!("Error Fetching {}: {}", &current_word, err));
                    self.info.msg = None;
                }
            }
        }
    }

    pub fn handle_copy_to_input(&mut self) {
        if let Some(i) = self.selected_expression {
            let selected_word = &self.expressions[i].dict_word;
            self.select_mode = SelectMode::Input;
            self.input.text += selected_word.trim();
            self.input.char_index = self.input.text.len() - 1;
        }
    }

    pub fn handle_paste(&mut self) {
        let mut clipboard = Clipboard::new().unwrap();
        self.input.text += &clipboard.get_text().unwrap();
        self.input.char_index = self.input.text.len();
    }
}
