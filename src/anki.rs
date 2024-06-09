#![allow(non_snake_case)]
use crate::app::*;
use anki_bridge::notes_actions::find_notes::FindNotesRequest;
use anki_bridge::notes_actions::notes_info::NotesInfoRequest;
use anki_bridge::prelude::*;
use anki_direct::notes::NoteAction;
use anki_direct::AnkiClient as AnkiDirectClient;
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

#[derive(Serialize, Deserialize)]
struct Note {
    id: u64,
    fields: HashMap<String, String>,
    audio: Option<Vec<Media>>,
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

#[derive(Serialize, Deserialize)]
struct ConfigOptions {
    del_words: bool,
    tts: bool,
}

// other
#[derive(Serialize, Deserialize)]
pub struct ConfigJson {
    pub fields: UserNoteFields,
    pub media_path: String,
    options: ConfigOptions,
}

#[derive(Serialize, Deserialize)]
pub struct UserNoteFields {
    pub expression: String,
    pub sentence: String,
    pub sentence_audio: String,
    pub image: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReqResult {
    result: Option<Vec<u64>>,
    error: Option<String>,
}

struct AnkiSentence {
    sentence_obj: Sentence,
    filename: Option<String>,
    local_audio_url: Option<String>,
}

impl AnkiSentence {
    fn into_anki_sentence(sentence: Sentence, config: &ConfigJson) -> Self {
        let (filename, local_audio_url) = if let Some(audio_url) = &sentence.audio_url {
            let filename = url_into_file_name(audio_url);
            let local_audio_url = sentence.audio_data.as_ref().map(|audio_data| {
                write_audio_bytes_file(&config.media_path, &filename, audio_data).unwrap()
            });

            (Some(filename), local_audio_url)
        } else {
            (None, None)
        };

        Self {
            sentence_obj: sentence,
            filename,
            local_audio_url,
        }
    }
}

impl AppState {
    pub async fn update_last_anki_card(&mut self) {
        let instant = Instant::now();
        let bad_client = &self.client;
        let client = AnkiDirectClient::default();
        

        let config: ConfigJson = match read_config() {
            Ok(config) => config,
            Err(err) => {
                self.err_msg = Some(format!("Error Reading Config: {}", err));
                return;
            }
        };

        if let Some(i) = self.selected_expression {
            let current_word = &self.expressions[i].dict_word.clone();
        let sentence_objs_vec = &self.notes_to_be_created.sentences;
        let mut note_ids: Vec<usize> = Vec::new();

        for current_sentence in sentence_objs_vec {
            let exp = &current_sentence.parent_expression;

            let id = match self.input.text.trim().parse::<usize>() {
                Ok(id) => id, // if the parsing succeeds, use the parsed id
                Err(_) => match check_note_exists(&client, &exp.dict_word).await {
                    Ok(id) => id,
                    Err(err) => {
                        self.err_msg =
                            Some(format!("Error fetching Note: {}: {}", &exp.dict_word, err));
                        return;
                    }
                },
            };

            note_ids.push(id);
        }

            let (filename, local_audio_url) = if let Some(audio_url) = &sentence.audio_url {
                let filename = url_into_file_name(audio_url);
                let local_audio_url = if let Some(audio_data) = &sentence.audio_data {
                    Some(write_audio_bytes_file(&config.media_path, &filename, audio_data).unwrap())
                } else {
                    None
                };

                (Some(filename), local_audio_url)
            } else {
                (None, None)
            };

            let req: Request<UpdateNoteParams> = match filename {
                Some(filename) => into_update_note_req(
                    note_id as u64,
                    &config.fields,
                    sentence,
                    filename,
                    local_audio_url,
                ),
                None => into_update_only_sentence_req(note_id as u64, &config.fields, sentence),
            };

            match post_note_update(req).await {
                Ok(_) => {
                    let elapsed = instant.elapsed().as_secs();
                    self.info.msg = format!(
                        "Updated Fields for CardID: {} - ({}) in {}s",
                        &note_id, &current_word, elapsed
                    )
                    .into();
                    match open_note_gui(bad_client, note_id) {
                        Ok(_) => match self.delete_word_from_file(current_word) {
                            Ok(_) => {}
                            Err(err) => {
                                self.err_msg =
                                    Some(format!("Error Deleting Word from File: {}", err))
                            }
                        },
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

fn url_into_file_name(url: &str) -> String {
    url.rsplit_once('/')
        .unwrap_or_else(|| panic!("url: {}", url))
        .1
        .to_string()
}

fn open_note_gui(client: &AnkiClient, id: usize) -> Result<(), Box<dyn std::error::Error>> {
    client.request(GuiEditNoteRequest { note: id })?;
    Ok(())
}


async fn direct_find_note_from_word(
    client: &AnkiDirectClient,
    word: &str,
) -> Result<u64, Box<dyn std::error::Error>> {

    let id_vec = NoteAction::find_note_ids(client, word).await?;

    match id_vec.last() {
        Some(id) => Ok(*id),
        None => Err(format!("No notes found for: {}", &word).into()),
    }
}

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
        None => Err(format!("No notes found for: {}", &word).into()),
    }
}

#[allow(dead_code)]
pub fn find_newest_note(client: &AnkiClient) -> Result<usize, Box<dyn std::error::Error>> {
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

fn format_sentence_field(field_name: &str, ik_sentence: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert(field_name.to_string(), ik_sentence.to_string());
    map
}

fn format_local_audio_field(field_name: &str, url: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert(field_name.to_string(), url.to_string());
    map
}

fn write_audio_bytes_file(
    media_dir: &str,
    filename: &str,
    bytes: &Vec<u8>,
) -> std::io::Result<String> {
    let media_dir = Path::new(&media_dir);
    let file_path = media_dir.join(filename);
    std::fs::write(file_path, bytes).unwrap();
    let audio_url = format!("[sound:{}]", filename);
    Ok(audio_url)
}

fn into_update_only_sentence_req(
    id: u64,
    anki_fields: &UserNoteFields,
    sentence: Sentence,
) -> Request<UpdateNoteParams> {
    let sentence_field = format_sentence_field(&anki_fields.sentence, &sentence.sentence);
    let note = Note {
        id,
        fields: { sentence_field },
        audio: None,
        picture: None,
    };

    let params = UpdateNoteParams { note };

    Request {
        action: "updateNoteFields".to_string(),
        version: 6,
        params,
    }
}

fn into_update_note_req(
    id: u64,
    anki_fields: &UserNoteFields,
    sentence: Sentence,
    filename: String,
    local_audio_url: Option<String>,
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

    let local_audio_field: Option<HashMap<String, String>> =
        local_audio_url.map(|local_audio_url| {
            format_local_audio_field(&anki_fields.sentence_audio, &local_audio_url)
        });

    let mut note: Note = match picture {
        Some(picture) => Note {
            id,
            fields: { sentence_field },
            audio: None,
            picture: Some(picture),
        },
        None => Note {
            id,
            fields: { sentence_field },
            audio: None,
            picture,
        },
    };

    if let Some(audio_field) = local_audio_field {
        note.fields.extend(audio_field);
    } else {
        note.audio = Some(vec![Media {
            url: sentence.audio_url.unwrap(),
            filename,
            skipHash: None,
            fields: vec![anki_fields.sentence_audio.clone()],
        }]);
    }

    let params = UpdateNoteParams { note };

    Request {
        action: "updateNoteFields".to_string(),
        version: 6,
        params,
    }
}

 pub async fn check_note_exists(
    client: &AnkiDirectClient,
    current_exp: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let config = read_config()?;

    let note_id = direct_find_note_from_word(client, current_exp).await?;

    let note_infos = NoteAction::get_notes_infos(client, vec![note_id]).await?;

    for n in note_infos {
        let exp_html = match n.fields.get(&config.fields.expression) {
            Some(html) => html,
            None => {
                return Err(format!(
                    "Incorrect Field: `{}`; `expression` field in config has to match Anki Note!",
                    &config.fields.expression
                )
                .into())
            }
        };
        let re = regex::Regex::new(r">([^<]+)<")?;
        let text = &exp_html.value;

        let mut result = String::new();
        for cap in re.captures_iter(text) {
            result.push_str(&cap[1]);
        }

        if *current_exp.trim() == *text || *current_exp.trim() == *result.trim() {
            return Ok(note_id as usize);
        }
    }

    Err(format!("Can't find `{}` in any decks!", current_exp).into())
}

pub fn read_config() -> Result<ConfigJson, std::io::Error> {
    let config_path = "./config.json";
    let data = std::fs::read(config_path)?;
    let config: ConfigJson = serde_json::from_slice(&data)?;

    Ok(config)
}
