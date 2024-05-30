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

fn url_into_file_name(url: &str) -> String {
    url.rsplit_once('/').unwrap().1.to_string()
}


fn read_config() -> Result<UserNoteFields, std::io::Error> {
    let config_path = "./config.json";
    let file = std::fs::File::open(config_path)?;
    let config: ConfigJson = serde_json::from_reader(file)?;

    Ok(config.fields)
}
