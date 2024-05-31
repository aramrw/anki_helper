#![allow(non_snake_case)]
use crate::app::*;
use anki_bridge::notes_actions::find_notes::FindNotesRequest;
use anki_bridge::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

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
struct FindNotesParams {
    query: String,
}

trait AnkiParams {}
impl AnkiParams for UpdateNoteParams {}

#[derive(Serialize, Deserialize)]
struct Request<P: AnkiParams> {
    action: String,
    version: u8,
    params: P,
}

// other
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

impl AppState {
    pub async fn update_last_anki_card(&mut self) {
        let instant = Instant::now();
        let client: AnkiClient<'_> = AnkiClient::default();

        if let Some(i) = self.selected_expression {
            let current_word = &self.expressions[i].dict_word.clone();

            let note_id = match find_newest_note(&client) {
                Ok(id) => id,
                Err(err) => {
                    self.err_msg = Some(format!("Error Finding Card: {}", err));
                    return;
                }
            };

            let fields: UserNoteFields = match read_config() {
                Ok(fields) => fields,
                Err(err) => {
                    self.err_msg = Some(format!("Error Reading Config: {}", err));
                    return;
                }
            };

            let sentence: Sentence = match &self.get_current_sentence() {
                Some(sent) => sent.clone(),
                None => {
                    self.err_msg = Some("Error: Failed to Get Current Sentence".to_string());
                    return;
                }
            };

            let filename = url_into_file_name(&sentence.audio_url.clone());
            let req: Request<UpdateNoteParams> =
                into_update_note_req(note_id as u64, fields, sentence, filename);
            match post_note_update(req).await {
                Ok(_) => {
                    let elapsed = instant.elapsed().as_secs();
                    self.info.msg = format!(
                        "Updated Fields for CardID: {} - ({}) in {}s",
                        &note_id, &current_word, elapsed
                    )
                    .into();
                    match open_note_gui(&client, note_id) {
                        Ok(_) => {}
                        Err(err) => {
                            self.err_msg = Some(format!("Error Opening Note GUI: {}", err));
                        }
                    }
                }
                Err(err) => {
                    let elapsed = instant.elapsed().as_secs();
                    self.err_msg = Some(format!(
                        "POST Error -> Failed to Update Anki Card: {} after {}s",
                        err, elapsed
                    ));
                }
            };
        }
    }
}

fn open_note_gui(client: &AnkiClient, id: usize) -> Result<(), Box<dyn std::error::Error>> {
    client.request(GuiEditNoteRequest { note: id })?;
    Ok(())
}

#[allow(dead_code)]
fn find_note_from_word(
    client: &AnkiClient,
    word: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let id_vec = client
        .request(FindNotesRequest {
            query: word.to_string(),
        })?
        .0;
    match id_vec.last() {
        Some(id) => Ok(*id),
        None => Err("No new cards found".into()),
    }
}

fn find_newest_note(client: &AnkiClient) -> Result<usize, Box<dyn std::error::Error>> {
    let id_vec = client
        .request(FindNotesRequest {
            query: "is:new".to_string(),
        })?
        .0;
    match id_vec.last() {
        Some(id) => Ok(*id),
        None => Err("ID for {} card not found".into()),
    }
}

async fn post_note_update(
    req: Request<UpdateNoteParams>,
) -> Result<(), Box<dyn std::error::Error>> {
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

#[allow(dead_code)]
fn write_audio_bytes_file(filename: &str, bytes: &Vec<u8>) -> std::io::Result<String> {
    let dir = tempfile::tempdir()?;
    let file_path = dir.path().join(filename);
    std::fs::write(&file_path, bytes)?;
    let audio_url = format!("file://{}", &file_path.to_string_lossy());

    Ok(audio_url)
}

fn into_update_note_req(
    id: u64,
    anki_fields: UserNoteFields,
    sentence: Sentence,
    filename: String,
) -> Request<UpdateNoteParams> {
    let sentence_field = format_sentence_field(&anki_fields.sentence, &sentence.sentence);
    let picture: Option<Vec<Media>> = match &sentence.img_url {
        Some(img_url) => vec![Media {
            url: img_url.clone(),
            filename: url_into_file_name(img_url),
            skipHash: None,
            fields: vec![anki_fields.image.clone()],
        }]
        .into(),
        None => None,
    };

    let audio: Vec<Media> = vec![Media {
        url: sentence.audio_url,
        filename,
        skipHash: None,
        fields: vec![anki_fields.sentence_audio.clone()],
    }];

    let note = Note {
        id,
        fields: { sentence_field },
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
