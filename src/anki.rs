#![allow(non_snake_case)]
use crate::app::*;
use anki_bridge::prelude::*;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Note {
    id: u64,
    fields: HashMap<String, String>,
    audio: Vec<Media>,
    picture: Option<Vec<Media>>,
}

#[derive(Serialize, Deserialize)]
struct Media {
    url: String,
    filename: String,
    skipHash: Option<String>,
    fields: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct UpdateNoteParams {
    note: Note,
}

#[derive(Serialize, Deserialize)]
struct Request {
    action: String,
    version: u8,
    params: UpdateNoteParams,
}

#[derive(Serialize, Deserialize)]
struct ConfigJson {
    fields: UserNoteFields,
}

#[derive(Serialize, Deserialize)]
struct UserNoteFields {
    sentence: String,
    sentence_audio: String,
    image: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReqResult {
    result: Option<Vec<u64>>,
    error: Option<String>,
}

    pub async fn update_last_anki_card(&mut self) {
        let client = AnkiClient::default();

        let card_id = match client.request(FindCardsRequest {
            query: "is:new".to_string(),
        }) {
            Ok(res) => {
                let id = match res.last().copied() {
                    Some(id) => id,
                    None => {
                        self.err_msg = Some(
                            "Error: Failed to Fetch Card ID\n    -> No Cards Found...".to_string(),
                        );
                        return;
                    }
                };
                id
            }
            Err(err) => {
                self.err_msg = Some(format!("Error Making Card: {}", err));
                return;
            }
        };
async fn post_note_update(req: Request) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().build()?;
    let res = client
        .post("http://localhost:8765")
        .json(&req)
        .send()
        .await?;

    let res: ReqResult = res.json().await?;

    match res {
        ReqResult {
            result: Some(_),
            error: None,
        } => {}
        ReqResult {
            result: None,
            error: Some(err),
        } => return Err(err.into()),
        _ => return Ok(()),
    }

    Ok(())
}

fn url_into_file_name(url: &str) -> String {
    url.rsplit_once('/').unwrap().1.to_string()
}

fn format_sentence_field(field_name: &str, ik_sentence: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert(field_name.to_string(), ik_sentence.to_string());
    map
}

fn into_update_note_req(id: u64, anki_fields: UserNoteFields, sentence: Sentence) -> Request {
    let sentence_field = format_sentence_field(&anki_fields.sentence, &sentence.sentence);
    let picture: Option<Vec<Media>> = match sentence.img_url {
        Some(img_url) => vec![Media {
            url: img_url.clone(),
            filename: url_into_file_name(&img_url),
            skipHash: None,
            fields: vec![anki_fields.image.clone()],
        }]
        .into(),
        None => None,
    };
    let audio: Vec<Media> = vec![Media {
        url: sentence.audio_url.clone(),
        filename: url_into_file_name(&sentence.audio_url.clone()),
        skipHash: None,
        fields: vec![anki_fields.sentence_audio.clone()],
    }];

    let note = Note {
        id,
        fields: {sentence_field},
        audio,
        picture,
    };
    let params = UpdateNoteParams { note };

    Request {
        action: "updateNoteFields".to_string(),
        version: 6,
        params,
    }
}

fn read_config() -> Result<UserNoteFields, std::io::Error> {
    let config_path = "./config.json";
    let file = std::fs::File::open(config_path)?;
    let config: ConfigJson = serde_json::from_reader(file)?;

    Ok(config.fields)
}
