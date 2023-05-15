//! An abstraction over the (now undocumented) Spotify object model.
//!
//! Types here are *not* 1:1 representations of what the Spotify API returns, since such types are tedious to work with
//! in a type-safe manner. Refer to the type documentation on how they map to the Spotify API objects.

pub mod album;
pub mod artist;
pub mod error;
pub mod id;
pub mod playback;
pub mod search;
pub mod track;
pub mod user;

mod country_code;
pub(crate) mod object_type;
mod page;

use std::{fmt, str::FromStr};

pub use country_code::CountryCode;
pub use page::Page;
use serde::{Deserialize, Serialize};

use crate::error::IdError;

// TODO: maybe make the fields private and expose them through functions
/// Contains an URL to an image and its dimensions, if specified.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    #[serde(flatten)]
    pub dimensions: Option<ImageDimensions>,
}

/// An image's dimensions.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// A content restriction.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Restrictions {
    // TODO: this can be "market", "product", "explicit" or something else in the future. make it an enum
    /// Reason for the content restriction.
    pub reason: Option<String>,
}

/// A date's precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatePrecision {
    Year,
    Month,
    Day,
}

/// Known external URLs for an object.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalUrls {
    /// The Spotify URL for the object.
    pub spotify: Option<String>,
}

/// Known external IDs for an object.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalIds {
    /// [International Standard Recording Code](https://en.wikipedia.org/wiki/International_Standard_Recording_Code)
    pub isrc: Option<String>,
    /// [International Article Number](https://en.wikipedia.org/wiki/International_Article_Number)
    pub ean: Option<String>,
    /// [Universal Product Code](https://en.wikipedia.org/wiki/Universal_Product_Code)
    pub upc: Option<String>,
}

// TODO: is this even used anywhere?
/// A copyright.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Copyright {
    pub text: String,
    pub copyright_type: CopyrightType,
}

/// The type of a copyright.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CopyrightType {
    #[serde(rename = "P")]
    Performance,
    C, // TODO: what the shit is this supposed to be? i can't find anything about it in the spotify docs
}

/// The type of an item in the Spotify catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ItemType {
    Album,
    Artist,
    Playlist,
    Track,
    Show,
    Episode,
    Collection,
    User,
}

impl crate::private::Sealed for ItemType {}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemType::Album => write!(f, "album"),
            ItemType::Artist => write!(f, "artist"),
            ItemType::Playlist => write!(f, "playlist"),
            ItemType::Track => write!(f, "track"),
            ItemType::Show => write!(f, "show"),
            ItemType::Episode => write!(f, "episode"),
            ItemType::Collection => write!(f, "collection"),
            ItemType::User => write!(f, "user"),
        }
    }
}

impl FromStr for ItemType {
    type Err = crate::error::IdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "album" => Ok(Self::Album),
            "artist" => Ok(Self::Artist),
            "playlist" => Ok(Self::Playlist),
            "track" => Ok(Self::Track),
            "show" => Ok(Self::Show),
            "episode" => Ok(Self::Episode),
            "collection" => Ok(Self::Collection),
            "user" => Ok(Self::User),

            other => Err(IdError::InvalidItemType(other.to_owned())),
        }
    }
}
