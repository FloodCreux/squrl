use image::DynamicImage;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RequestResponse {
    pub duration: Option<String>,
    pub status_code: Option<String>,
    pub content: Option<ResponseContent>,
    pub cookies: Option<String>,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseContent {
    Body(String),
    Image(ImageResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResponse {
    pub data: Vec<u8>,

    #[serde(skip)]
    pub image: Option<DynamicImage>,
}

#[derive(Error, Debug)]
pub enum RequestResponseError {
    #[error("COULD NOT DECODE RESPONSE TEXT OR BYTES")]
    CouldNotDecodeResponse,
}
