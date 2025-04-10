use crate::anki::return_new_anki_words;
use crate::app::*;
use arboard::Clipboard;
use rayon::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*, BufReader, BufWriter};
use std::process::Command;
use std::time::Instant;

impl AppState {
    pub fn get_current_sentence(&self) -> Option<Sentence> {
        if self.expressions.is_empty() {
            return None;
        }
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

    pub fn delete_exps_from_app_data(&mut self, del_vec: &[String]) {
        let mut indexes: Vec<usize> = self
            .expressions
            .par_iter()
            .enumerate()
            .filter_map(|(i, exp)| {
                if del_vec.contains(&exp.dict_word) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        // Sort the indexes in reverse order
        indexes.sort_unstable_by(|a, b| b.cmp(a));

        for &i in &indexes {
            self.expressions.remove(i);
        }

        // Set the selected expression to the one before the last one that was deleted
        let final_index = indexes.last().and_then(|&last_index| {
            if last_index > 0 {
                Some(last_index - 1)
            } else {
                None
            }
        });
        self.selected_expression = final_index;
        self.expressions_state.select(final_index);
    }

    pub fn delete_words_from_file(&mut self, del_vec: &Vec<String>) -> io::Result<()> {
        {
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
            self.delete_exps_from_app_data(del_vec);
        }

        self.clean_up_words_file()?;

        Ok(())
    }

    pub fn clean_up_words_file(&mut self) -> io::Result<()> {
        let file = File::open("words.txt")?;
        let reader = BufReader::new(file);

        let mut words = Vec::new();

        for line in reader.lines() {
            let line = line?;
            for word in line.split_whitespace() {
                words.push(word.to_string());
            }
        }

        let cleaned_words = words.join(" ");

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("words.txt")?;

        writeln!(file, "{}", cleaned_words)?;

        Ok(())
    }

    pub async fn read_words_file(&mut self) -> io::Result<()> {
        let file = File::open("words.txt")?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;

            for word in line.split_whitespace() {
                self.expressions
                    .push(Expression::from(word.to_string(), None, None, None));
            }
        }

        if self.config.options.auto_load_new_notes {
            match return_new_anki_words(&self.client, &self.config, "is:new -is:suspended").await {
                Ok(exps) => {
                    for exp in exps {
                        if self.expressions.contains(&exp) {
                            continue;
                        }
                        self.expressions.push(exp);
                    }
                }
                Err(e) => {
                    self.update_error_msg("New Anki Notes Err", e.to_string());
                }
            };
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

    pub async fn fetch_sentences(&mut self, is_massif: bool) {
        if let Some(i) = self.selected_expression {
            let instant = Instant::now();
            let current_word = self.expressions[i].dict_word.clone();

            let format_url = format!(
                "https://api.immersionkit.com/look_up_dictionary?keyword={}&sort=shortness",
                &current_word
            );

            match self
                .fetch_ik_api(self.expressions[i].clone(), i, format_url, is_massif)
                .await
            {
                Ok(_) => {
                    if self.expressions[i].exact_search {
                        self.info.msg = format!(
                            "Fetched Massif Sentences for {} in {}s",
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
                    self.errors
                        .push(format!("Error Fetching {}: {}", &current_word, err));
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

pub fn _write_media(titles: Vec<String>) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("data/media.txt")?;
    let mut new_titles: Vec<String> = Vec::new();

    let reader = BufReader::new(&file);
    for line in reader.lines() {
        let line = line?;
        if !titles.contains(&line.trim().to_string()) {
            continue;
        }

        new_titles.push(line);
    }

    let mut writer = BufWriter::new(&file);
    for title in titles {
        writeln!(writer, "{}", title)?;
    }

    writer.flush()?;
    Ok(())
}

pub fn write_to_errs_log(err_vec: &Vec<String>) -> std::io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("data/err_log.txt")?;

    let mut writer = BufWriter::new(file);

    for err in err_vec {
        writeln!(writer, "{}", err)?;
    }

    writer.flush()?;

    Ok(())
}
