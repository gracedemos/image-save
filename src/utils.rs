use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    pub title: String
}