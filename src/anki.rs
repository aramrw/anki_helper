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

