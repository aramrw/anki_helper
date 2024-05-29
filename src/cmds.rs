use crate::app::*;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::{thread, time::*};

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

    pub async fn fetch_sentences(&mut self) {
        if let Some(i) = self.selected_expression {
            if let Some(sentences) = &self.expressions[i].sentences {
                if !sentences.is_empty() {
                    return;
                };
            }
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
