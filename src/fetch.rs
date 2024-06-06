use crate::anki::{find_newest_note, read_config};
use anki_bridge::notes_actions::notes_info::NotesInfoRequest;
use anki_bridge::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

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
    image_url: String,
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
struct IKJsonSchema {
    data: Vec<Main>,
}

// Massif Schema (shoutout massif... way simpler lol)
#[derive(Serialize, Deserialize, Debug)]
struct MassifSampleSource {
    publish_date: String,
    title: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MassifResults {
    highlighted_html: String,
    sample_source: MassifSampleSource,
    source_count: u16,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MassifJsonSchema {
    hits: u16,
    hits_limited: bool,
    results: Vec<MassifResults>,
}

use crate::app::*;

impl AppState {
    pub async fn fetch_massif_api(
        &mut self,
        _word: String,
        index: usize,
        format_url: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::get(&format_url)
            .await?
            .json::<MassifJsonSchema>()
            .await?;

        let mut sentences: Vec<Sentence> = Vec::new();
        for item in resp.results {
            let wbst_link = format!("https://massif.la/ja/search?q={}", &item.text);
            sentences.push(Sentence::from(
                &item.text,
                None,
                None,
                None,
                &item.sample_source.title,
                &wbst_link
            ));
        }

        if sentences.is_empty() {
            return Err("0 Sentences Found!".into());
        }

        self.expressions[index].sentences = Some(sentences);
        Ok(())
    }

    pub async fn fetch_ik_api(
        &mut self,
        _word: String,
        index: usize,
        format_url: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::get(&format_url)
            .await?
            .json::<IKJsonSchema>()
            .await?;

        let mut sentences: Vec<Sentence> = Vec::new();
        for item in resp.data {
            for empty_vec in item.dictionary {
                for section in empty_vec {
                    self.expressions[index]
                        .definitions
                        .extend(section.glossary_list);
                    if !self.expressions[index].readings.contains(&section.reading)
                        && self.expressions[index].dict_word != section.reading
                    {
                        self.expressions[index].readings.push(section.reading);
                    }
                }
            }
            for ex in item.examples {
                let image_url = if !ex.image_url.is_empty() {
                    Some(ex.image_url.to_string())
                } else {
                    None
                };

                let wbst_link = format!("https://www.immersionkit.com/dictionary?keyword={}", &ex.sentence);

                sentences.push(Sentence::from(
                    &ex.sentence,
                    Some(ex.sound_url),
                    None,
                    image_url,
                    &ex.deck_name,
                    &wbst_link
                ));
            }
        }

        if sentences.is_empty() {
            self.fetch_massif_sentences().await?;
            return Ok(());
        }

        self.expressions[index].sentences = Some(sentences);
        Ok(())
    }

    // pub async fn push_audio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    //     if let Some(exp_index) = self.selected_expression {
    //         if let Some(sent_index) = self.expressions[exp_index].selected_sentence {
    //             let sentence =
    //                 &mut self.expressions[exp_index].sentences.as_mut().unwrap()[sent_index];
    //
    //             if sentence.audio_data.is_some() {
    //                 return Ok(());
    //             }
    //
    //             let audio_url = sentence.audio_url.clone().ok_or("Audio URL not found")?;
    //             let resp = reqwest::get(&audio_url).await?;
    //             let audio_data = resp.bytes().await?.to_vec();
    //             sentence.audio_data = Some(audio_data.clone());
    //         }
    //     }
    //     Ok(())
    // }

    pub async fn play_audio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(exp_index) = self.selected_expression {
            if let Some(sent_index) = self.expressions[exp_index].selected_sentence {
                let sentence =
                    &mut self.expressions[exp_index].sentences.as_mut().unwrap()[sent_index];

                let audio_data = if let Some(audio_data) = &sentence.audio_data {
                    audio_data.clone()
                } else {
                    let audio_url = sentence.audio_url.clone().ok_or("Audio URL not found")?;
                    let resp = reqwest::get(&audio_url).await?;
                    let audio_data = resp.bytes().await?.to_vec();
                    sentence.audio_data = Some(audio_data.clone());
                    audio_data
                };

                tokio::task::spawn_blocking(move || {
                    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
                    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                    let cursor = Cursor::new(audio_data);
                    let source = rodio::Decoder::new(cursor).unwrap();
                    sink.append(source);
                    sink.sleep_until_end();
                })
                .await
                .unwrap();
            }
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub fn check_note_exists(
    client: AnkiClient,
    current_exp: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = read_config()?;

    let note_id = find_newest_note(&client)?;

    let note_info = client.request(NotesInfoRequest {
        notes: vec![note_id],
    });

    let note = note_info?;
    let exp_html = note
        .last()
        .unwrap()
        .fields
        .get(&config.fields.expression)
        .unwrap();

    // extract text from html and join them

    let re = regex::Regex::new(r">([^<]+)<").unwrap();
    let text = &exp_html.value;
    let mut result = String::new();
    for cap in re.captures_iter(text) {
        result.push_str(&cap[1]);
    }

    if *current_exp != result {
        return Err(format!(
            "Warning: Last Made Note: {} != {}; It Will Get Overwritten!",
            result, current_exp
        )
        .into());
    }

    Ok(())
}
