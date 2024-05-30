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

