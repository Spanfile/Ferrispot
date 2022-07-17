pub mod album;
pub mod artist;
pub mod country_code;
pub mod error;
pub mod object_type;
pub mod page;
pub mod playback;
pub mod search;
pub mod track;

use serde::{Deserialize, Serialize};

// TODO: really gotta do a pass of what all derives are actually useful for everything

mod private {
    pub trait Sealed {}
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    #[serde(flatten)]
    pub dimensions: Option<ImageDimensions>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Restrictions {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatePrecision {
    Year,
    Month,
    Day,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalUrls {
    pub spotify: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalIds {
    pub isrc: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Copyright {
    pub text: String,
    pub copyright_type: CopyrightType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CopyrightType {
    #[serde(rename = "P")]
    Performance,
    C, // TODO: what the shit is this supposed to be? i can't find anything about it in the spotify docs
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Album,
    Artist,
    Playlist,
    Track,
    Show,
    Episode,
}
