use super::{
    artist::ArtistObject, country_code::CountryCode, Copyright, DatePrecision, ExternalIds, ExternalUrls, Image,
    Restrictions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Album {
    Full(FullAlbum),
    Partial(PartialAlbum),
    Local(LocalAlbum),
}

/// This struct covers all the possible album responses from Spotify's API. It has a function that converts it into an
/// [Album], depending on which fields are set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AlbumObject {
    /// Fields available in every album
    #[serde(flatten)]
    common: CommonAlbumFields,

    /// Fields only in non-local albums
    #[serde(flatten)]
    non_local: Option<NonLocalAlbumFields>,

    /// Fields only in full albums
    #[serde(flatten)]
    full: Option<FullAlbumFields>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CommonAlbumFields {
    // basic information
    name: String,
    artists: Vec<ArtistObject>,
    images: Vec<Image>,
    #[serde(default)]
    external_urls: ExternalUrls,
    #[serde(rename = "type")]
    item_type: String, // TODO: make a type-safe type much like in aspotify

    // track relinking
    available_markets: HashSet<CountryCode>,
    #[serde(default)]
    restrictions: Restrictions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FullAlbumFields {
    copyrights: Vec<Copyright>,
    external_ids: ExternalIds,
    genres: Vec<String>,
    label: String,
    popularity: u32,
    // pub tracks: Page<PartialTrack>, // TODO: paging
    // TODO: the artist album thing with the album group field
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NonLocalAlbumFields {
    album_type: AlbumType,
    id: String,
    release_date: String, // TODO: proper date type pls
    release_date_precision: DatePrecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FullAlbum {
    common: CommonAlbumFields,
    non_local: NonLocalAlbumFields,
    full: FullAlbumFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialAlbum {
    common: CommonAlbumFields,
    non_local: NonLocalAlbumFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalAlbum {
    common: CommonAlbumFields,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlbumType {
    #[serde(alias = "ALBUM")]
    Album,
    #[serde(alias = "SINGLE")]
    Single,
    #[serde(alias = "COMPILATION")]
    Compilation,
}

impl From<AlbumObject> for Album {
    fn from(obj: AlbumObject) -> Self {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => Self::Full(FullAlbum {
                common: obj.common,
                non_local,
                full,
            }),

            (Some(non_local), None) => Self::Partial(PartialAlbum {
                common: obj.common,
                non_local,
            }),

            (None, None) => Self::Local(LocalAlbum { common: obj.common }),

            (non_local, full) => panic!(
                "impossible case trying to convert AlbumObject into Album: non-local album fields is {:?} while full \
                 album fields is {:?}",
                non_local, full
            ),
        }
    }
}

impl From<PartialAlbum> for Album {
    fn from(partial: PartialAlbum) -> Self {
        Self::Partial(partial)
    }
}

impl From<FullAlbum> for Album {
    fn from(full: FullAlbum) -> Self {
        Self::Full(full)
    }
}

impl From<LocalAlbum> for Album {
    fn from(local: LocalAlbum) -> Self {
        Self::Local(local)
    }
}

impl From<Album> for FullAlbum {
    fn from(album: Album) -> Self {
        match album {
            Album::Full(full) => full,

            Album::Partial(_) => panic!("attempt to convert partial album into full album"),
            Album::Local(_) => panic!("attempt to convert local album into full album"),
        }
    }
}

impl From<AlbumObject> for FullAlbum {
    fn from(obj: AlbumObject) -> Self {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => FullAlbum {
                common: obj.common,
                non_local,
                full,
            },

            (non_local, full) => panic!(
                "attempt to convert non-full album object into full album (non-local album fields is {:?}, full album \
                 fields is {:?})",
                non_local, full
            ),
        }
    }
}

impl From<Album> for PartialAlbum {
    fn from(album: Album) -> Self {
        match album {
            Album::Full(full) => PartialAlbum {
                common: full.common,
                non_local: full.non_local,
            },
            Album::Partial(partial) => partial,

            Album::Local(_) => panic!("attempt to convert local album into partial album"),
        }
    }
}

impl From<AlbumObject> for PartialAlbum {
    fn from(obj: AlbumObject) -> Self {
        if let Some(non_local) = obj.non_local {
            PartialAlbum {
                common: obj.common,
                non_local,
            }
        } else {
            panic!(
                "attempt to convert local album object into partial album (non-local album fields is {:?})",
                obj.non_local
            );
        }
    }
}

impl From<Album> for LocalAlbum {
    fn from(album: Album) -> Self {
        match album {
            Album::Full(FullAlbum { common, .. }) | Album::Partial(PartialAlbum { common, .. }) => {
                LocalAlbum { common }
            }

            Album::Local(local) => local,
        }
    }
}

impl From<AlbumObject> for LocalAlbum {
    fn from(obj: AlbumObject) -> Self {
        LocalAlbum { common: obj.common }
    }
}
