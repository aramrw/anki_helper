use futures_util::StreamExt;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::thread;
use std::time;

#[derive(Serialize, Deserialize, Debug)]
struct JsonSchema {
    data: Vec<Main>,
}
