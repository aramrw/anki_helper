use crate::app::*;
use arboard::Clipboard;
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*, BufReader, BufWriter};
use std::process::Command;
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

    pub fn delete_words_from_file(&mut self, del_vec: &Vec<String>) -> io::Result<()> {
        let file = File::open("words.txt")?;
        let reader = BufReader::new(file);

        let temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("temp.txt")?;
        let mut writer = BufWriter::new(temp_file);

        for line in reader.lines() {
            let mut line = line?;
            for to_del_word in del_vec {
                if line.contains(to_del_word) {
                    line = line.replace(to_del_word, "");
                }
            }
            writeln!(writer, "{}", line)?;
        }

        std::fs::remove_file("words.txt")?;
        std::fs::rename("temp.txt", "words.txt")?;

        Ok(())
    }

    pub fn read_words_file(&mut self) -> io::Result<()> {
        let file = File::open("words.txt")?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;

            for word in line.split_whitespace() {
                self.expressions
                    .push(Expression::from(word.to_string(), None, None));
            }
        }

        Ok(())
    }

    pub async fn fetch_massif_sentences(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(i) = self.selected_expression {
            let current_word = self.expressions[i].dict_word.clone();
            let format_url = format!("https://massif.la/ja/search?q={}&fmt=json", &current_word);
            self.fetch_massif_api(self.expressions[i].clone(), i, format_url)
                .await?;
        }
        Ok(())
    }

    pub async fn fetch_sentences(&mut self) {
        if let Some(i) = self.selected_expression {
            let instant = Instant::now();
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

            match self
                .fetch_ik_api(self.expressions[i].clone(), i, format_url)
                .await
            {
                Ok(_) => {
                    self.err_msg = None;
                    if self.expressions[i].exact_search {
                        self.info.msg = format!(
                            "Fetched Exact Sentences for {} in {}s",
                            &current_word,
                            instant.elapsed().as_secs()
                        )
                        .into();
                    } else {
                        self.info.msg = format!(
                            "Fetched Sentences For {} in {}s",
                            &current_word,
                            instant.elapsed().as_secs()
                        )
                        .into();
                    }
                }
                Err(err) => {
                    self.select_mode = SelectMode::Expressions;
                    self.info.msg = None;
                    self.err_msg = Some(format!("Error Fetching {}: {}", &current_word, err));
                }
            }
        }
    }

    pub fn open_website_link(&mut self) {
        if let Some(sentence) = self.get_current_sentence() {
            if let Err(e) = webbrowser::open(&sentence.wbst_link) {
                self.update_error_msg("Error Opening Link: {}", e.to_string());
            }
        }
    }

    pub fn open_github(&mut self) {
        if let Err(e) = webbrowser::open("https://github.com/aramrw/anki_helper") {
            self.update_error_msg("Error Opening Github Link: {}", e.to_string());
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

    pub fn restart_program(&mut self) {
        match Command::new("anki_helper.exe").spawn() {
            Ok(_) => {
                std::process::exit(0);
            }
            Err(e) => {
                self.info.msg = None;
                self.update_error_msg("Error Restarting", e.to_string());
            }
        };
    }
}
