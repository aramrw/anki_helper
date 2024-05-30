use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::time::Instant;

#[derive(Serialize, Deserialize, Debug)]
struct CategoryCount {
    anime: u16,
    drama: u16,
    games: u16,
    literature: u16,
    news: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeckCount {
    anime: HashMap<String, u16>,
    drama: HashMap<String, u16>,
    games: HashMap<String, u16>,
    literature: HashMap<String, u16>,
    news: HashMap<String, u16>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Glossary {
    glossary_list: Vec<String>,
    headword: String,
    reading: String,
    sound: Option<String>,
    tags: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum StringOru64 {
    Empty(String),
    Number(u64),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum VecStringOrString {
    Empty(String),
    Vec(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug)]
struct Example {
    author_japanese: Option<String>,
    category: String,
    channel: Option<String>,
    deck_id: u16,
    deck_name: String,
    deck_name_japanese: Option<String>,
    episode: StringOru64,
    id: u64,
    image_url: Option<String>,
    position: u64,
    sentence: String,
    sentence_id: String,
    sentence_with_furigana: String,
    sound_begin: StringOru64,
    sound_end: StringOru64,
    sound_url: String,
    tags: Vec<String>,
    timestamp: Option<String>,
    translation: String,
    translation_word_index: Vec<usize>,
    translation_word_list: VecStringOrString,
    word_index: Vec<usize>,
    word_list: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Main {
    category_count: CategoryCount,
    deck_count: DeckCount,
    dictionary: Vec<Vec<Glossary>>,
    exact_match: Option<String>,
    examples: Vec<Example>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonSchema {
    data: Vec<Main>,
}

use crate::app::*;

impl AppState {
    pub async fn fetch_api(
        &mut self,
        word: String,
        index: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let instant = Instant::now();
        let format_url = format!(
            "https://api.immersionkit.com/look_up_dictionary?keyword={}&sort=shortness",
            &word
        );
        let resp = reqwest::get(&format_url)
            .await?
            .json::<JsonSchema>()
            .await
            .unwrap();

        let mut sentences: Vec<Sentence> = Vec::new();

        for item in resp.data {
            for ex in item.examples {
                if let Some(image_url) = ex.image_url {
                    sentences.push(Sentence::from(ex.sentence, ex.sound_url, image_url));
                }
            }
        }

        if sentences.is_empty() {
            self.select_mode = SelectMode::Expressions;
            self.info.msg = format!("No Sentences found for {}", &word).into();
            return Ok(());
        }

        self.expressions[index].sentences = Some(sentences);
        self.err_msg = None;
        self.info.msg = format!(
            "Fetched sentences for {} in {}s",
            &word,
            instant.elapsed().as_secs()
        )
        .into();
        Ok(())
    }

    pub async fn play_audio(&mut self) {
        if let Some(sentence) = self.get_current_sentence() {
            tokio::task::spawn_blocking(move || {
                let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
                let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                let resp = reqwest::blocking::get(sentence.audio_url).unwrap();
                let cursor = Cursor::new(resp.bytes().unwrap());
                let source = rodio::Decoder::new(cursor).unwrap();
                sink.append(source);
                sink.sleep_until_end();
            })
            .await
            .unwrap();
        }
    }
}
