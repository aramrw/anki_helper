//use crate::cmds::write_media;
use rayon::prelude::*;
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
        parent_expression: Expression,
        index: usize,
        format_url: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resp = reqwest::get(&format_url)
            .await?
            .json::<MassifJsonSchema>()
            .await?;

        let sentences: Vec<Sentence> = resp
            .results
            .par_iter()
            .map(|item| {
                let wbst_link = format!("https://massif.la/ja/search?q={}", &item.text);
                Sentence::from(
                    &item.text,
                    None,
                    None,
                    None,
                    &item.sample_source.title,
                    &wbst_link,
                    &parent_expression,
                )
            })
            .collect();

        if sentences.is_empty() {
            return Err("0 Sentences Found!".into());
        }

        self.expressions[index].sentences = Some(sentences);
        Ok(())
    }

    pub async fn fetch_ik_api(
        &mut self,
        parent_expression: Expression,
        index: usize,
        format_url: String,
        is_massif: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if is_massif {
            self.fetch_massif_sentences().await?;
            return Ok(());
        }

        let resp = reqwest::get(&format_url)
            .await?
            .json::<IKJsonSchema>()
            .await?;

        //let mut titles: Vec<String> = Vec::new();
        let mut sentences: Vec<Sentence> = Vec::new();

        for item in resp.data {
            for empty_vec in item.dictionary {
                for section in empty_vec {
                    self.expressions[index]
                        .definitions
                        .extend(section.glossary_list);

                    if !parent_expression.readings.contains(&section.reading)
                        && self.expressions[index].dict_word != section.reading
                    {
                        self.expressions[index].readings.push(section.reading);
                    }
                }
            }

            let item_sents: Vec<Sentence> = item
                .examples
                .into_iter()
                .filter_map(|ex| {
                    let image_url = if !ex.image_url.is_empty() {
                        Some(ex.image_url.to_string())
                    } else {
                        None
                    };

                    let wbst_link = format!(
                        "https://www.immersionkit.com/dictionary?keyword={}",
                        &ex.sentence
                    );

                    if self.config.priority.is_empty() {
                        return Some(Sentence::from(
                            &ex.sentence,
                            Some(ex.sound_url),
                            None,
                            image_url,
                            &ex.deck_name,
                            &wbst_link,
                            &parent_expression,
                        ));
                    }

                    if self.config.priority.contains(&ex.deck_name) {
                        return Some(Sentence::from(
                            &ex.sentence,
                            Some(ex.sound_url),
                            None,
                            image_url,
                            &ex.deck_name,
                            &wbst_link,
                            &parent_expression,
                        ));
                    }

                    None
                })
                .collect();

            sentences = item_sents;
        }

        if sentences.is_empty() {
            self.fetch_massif_sentences().await?;
            return Ok(());
        }

        //write_media(titles).unwrap();
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
