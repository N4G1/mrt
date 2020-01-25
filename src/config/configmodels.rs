use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ConfigFile {
    pub version: String,
    pub tags: HashMap<String, Tag>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Tag {
    pub paths: Vec<String>,
}
