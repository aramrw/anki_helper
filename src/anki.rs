#![allow(non_snake_case)]
use crate::app::*;
use crate::cmds::write_to_errs_log;
use anki_direct::notes::NoteAction;
use anki_direct::AnkiClient as AnkiDirectClient;
use futures_util::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Note {
    id: u128,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigOptions {
    pub del_words: bool,
    pub tts: bool,
}

// other
#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigJson {
    pub fields: UserNoteFields,
    pub media_path: String,
    pub options: ConfigOptions,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserNoteFields {
    pub expression: String,
    pub sentence: String,
    pub sentence_audio: String,
    pub image: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReqResult {
    result: Option<Vec<u128>>,
    error: Option<String>,
}

#[derive(Debug)]
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

pub struct UpdateNotesRes {
    pub dict_words_vec: Vec<String>,
    pub err_vec: Vec<String>,
    pub success_len: usize,
    pub total_len: usize,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ConfigOptions {
    pub del_words: bool,
    pub tts: bool,
    pub auto_load_new_notes: bool,
}

// other
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ConfigJson {
    pub fields: UserNoteFields,
    pub media_path: String,
    pub options: ConfigOptions,
}

pub async fn update_anki_cards(
    sentence_objs_vec: &Vec<Sentence>,
    config: &ConfigJson,
) -> Result<UpdateNotesRes, Box<dyn std::error::Error>> {
    let client = AnkiDirectClient::default();

    let mut err_vec: Vec<String> = Vec::new();

    let mut note_ids_and_sentences: Vec<(Option<u128>, AnkiSentence)> = Vec::new();

    for current_sentence in sentence_objs_vec {
        let exp = &current_sentence.parent_expression;
        let id = match check_note_exists(&client, &exp.dict_word).await {
            Ok(id) => Some(id),
            Err(err) => {
                let err = format!("`{}`: {}", &exp.dict_word, err);
                err_vec.push(err);
                None
            }
        };

        let anki_sentence = AnkiSentence::into_anki_sentence(current_sentence.clone(), config);
        note_ids_and_sentences.push((id, anki_sentence));
    }

    if note_ids_and_sentences.iter().all(|(id, _)| id.is_none()) {
        return Err("Err: 0 IDs found. Check `err.log.txt` for errors.".into());
    }
    let nias_len = note_ids_and_sentences.len();

    let dict_words_vec: Vec<String> = note_ids_and_sentences
        .iter()
        .map(|s| s.1.sentence_obj.parent_expression.dict_word.clone())
        .collect();

    let mut requests_vec: Vec<Request<UpdateNoteParams>> = Vec::new();

    note_ids_and_sentences.into_iter().for_each(|(id, anki_s)| {
        if let Some(note_id) = id {
            let req: Request<UpdateNoteParams> = match &anki_s.filename.clone() {
                Some(filename) => {
                    into_update_note_req(note_id, &config.fields, anki_s, filename.to_string())
                }
                None => into_update_only_sentence_req(note_id, &config.fields, &anki_s),
            };
            requests_vec.push(req);
        }
    });

    match post_note_updates(requests_vec, client.client).await {
        Ok(_) => {
            let result = UpdateNotesRes {
                dict_words_vec,
                success_len: nias_len,
                err_vec,
                total_len: sentence_objs_vec.len(),
            };

            Ok(result)
        }
        Err(err) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Network Err: {}", err),
        ))),
    }
}

impl AppState {
    #[allow(dead_code)]
    pub fn delete_notes_after_update_wrapper(&mut self, res: &UpdateNotesRes) {
        self.notes_to_be_created.state.select(None);
        let words_to_delete: Vec<String> = self
            .expressions
            .iter_mut()
            .filter_map(|exp| {
                let wrd = &exp.dict_word.clone();
                if res.dict_words_vec.contains(wrd) {
                    Some(wrd.clone())
                } else {
                    None
                }
            })
            .collect();

        if let Err(err) = self.delete_words_from_file(&words_to_delete) {
            self.update_error_msg("Err Deleting Word from File", err.to_string());
        }
    }
}

fn url_into_file_name(url: &str) -> String {
    url.rsplit_once('/')
        .unwrap_or_else(|| panic!("url: {}", url))
        .1
        .to_string()
}

async fn direct_find_note_from_word(
    client: &AnkiDirectClient,
    word: &str,
) -> Result<u128, Box<dyn std::error::Error>> {
    let id_vec = NoteAction::find_note_ids(client, word).await?;

    match id_vec.last() {
        Some(id) => Ok(*id),
        None => Err(format!("No notes found for: {}", &word).into()),
    }
}

async fn post_note_updates(
    reqs: Vec<Request<UpdateNoteParams>>,
    client: reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let futures: Vec<_> = reqs
        .into_iter()
        .map(|req| client.post("http://localhost:8765").json(&req).send())
        .collect();

    let results = join_all(futures).await;

    for result in results {
        let res = result?;
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
    id: u128,
    anki_fields: &UserNoteFields,
    sentence: &AnkiSentence,
) -> Request<UpdateNoteParams> {
    let sentence_field =
        format_sentence_field(&anki_fields.sentence, &sentence.sentence_obj.sentence);
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
    id: u128,
    anki_fields: &UserNoteFields,
    sentence: AnkiSentence,
    filename: String,
) -> Request<UpdateNoteParams> {
    let sentence_field =
        format_sentence_field(&anki_fields.sentence, &sentence.sentence_obj.sentence);

    let picture: Option<Vec<Media>> = match &sentence.sentence_obj.img_url {
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
        sentence.local_audio_url.map(|local_audio_url| {
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
            url: sentence.sentence_obj.audio_url.unwrap(),
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
) -> Result<u128, Box<dyn std::error::Error>> {
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
            return Ok(note_id);
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
