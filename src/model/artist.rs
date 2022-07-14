use super::{ExternalUrls, Image};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Artist {
    Full(FullArtist),
    Partial(PartialArtist),
    Local(LocalArtist),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ArtistObject {
    /// Fields available in every artist
    #[serde(flatten)]
    common: CommonArtistFields,

    /// Fields only in non-local artist
    #[serde(flatten)]
    non_local: Option<NonLocalArtistFields>,

    /// Fields only in full artist
    #[serde(flatten)]
    full: Option<FullArtistFields>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CommonArtistFields {
    name: String,
    #[serde(default)]
    external_urls: ExternalUrls,
    #[serde(rename = "type")]
    item_type: String, // TODO: make a type-safe type much like in aspotify
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FullArtistFields {
    // followers: Followers,
    genres: Vec<String>,
    images: Vec<Image>,
    popularity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NonLocalArtistFields {
    id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullArtist {
    common: CommonArtistFields,
    non_local: NonLocalArtistFields,
    full: FullArtistFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialArtist {
    common: CommonArtistFields,
    non_local: NonLocalArtistFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalArtist {
    common: CommonArtistFields,
}

impl From<ArtistObject> for Artist {
    fn from(obj: ArtistObject) -> Self {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => Self::Full(FullArtist {
                common: obj.common,
                non_local,
                full,
            }),

            (Some(non_local), None) => Self::Partial(PartialArtist {
                common: obj.common,
                non_local,
            }),

            (None, None) => Self::Local(LocalArtist { common: obj.common }),

            (non_local, full) => panic!(
                "impossible case trying to convert ArtistObject into Artist: non-local artist fields is {:?} while \
                 full artist fields is {:?}",
                non_local, full
            ),
        }
    }
}

impl From<PartialArtist> for Artist {
    fn from(partial: PartialArtist) -> Self {
        Self::Partial(partial)
    }
}

impl From<FullArtist> for Artist {
    fn from(full: FullArtist) -> Self {
        Self::Full(full)
    }
}

impl From<LocalArtist> for Artist {
    fn from(local: LocalArtist) -> Self {
        Self::Local(local)
    }
}

impl From<Artist> for FullArtist {
    fn from(artist: Artist) -> Self {
        match artist {
            Artist::Full(full) => full,

            Artist::Partial(_) => panic!("attempt to convert partial artist into full artist"),
            Artist::Local(_) => panic!("attempt to convert local artist into full artist"),
        }
    }
}

impl From<ArtistObject> for FullArtist {
    fn from(obj: ArtistObject) -> Self {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => FullArtist {
                common: obj.common,
                non_local,
                full,
            },

            (non_local, full) => panic!(
                "attempt to convert non-full artist object into full artist (non-local artist fields is {:?}, full \
                 artist fields is {:?})",
                non_local, full
            ),
        }
    }
}

impl From<Artist> for PartialArtist {
    fn from(artist: Artist) -> Self {
        match artist {
            Artist::Full(full) => PartialArtist {
                common: full.common,
                non_local: full.non_local,
            },
            Artist::Partial(partial) => partial,

            Artist::Local(_) => panic!("attempt to convert local artist into partial artist"),
        }
    }
}

impl From<ArtistObject> for PartialArtist {
    fn from(obj: ArtistObject) -> Self {
        if let Some(non_local) = obj.non_local {
            PartialArtist {
                common: obj.common,
                non_local,
            }
        } else {
            panic!(
                "attempt to convert local artist object into partial artist (non-local artist fields is {:?})",
                obj.non_local
            );
        }
    }
}

impl From<Artist> for LocalArtist {
    fn from(artist: Artist) -> Self {
        match artist {
            Artist::Full(FullArtist { common, .. }) | Artist::Partial(PartialArtist { common, .. }) => {
                LocalArtist { common }
            }

            Artist::Local(local) => local,
        }
    }
}

impl From<ArtistObject> for LocalArtist {
    fn from(obj: ArtistObject) -> Self {
        LocalArtist { common: obj.common }
    }
}
