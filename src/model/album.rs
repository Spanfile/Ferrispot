use super::{
    artist::{ArtistObject, PartialArtist},
    country_code::CountryCode,
    object_type::{obj_deserialize, TypeAlbum},
    Copyright, DatePrecision, ExternalIds, ExternalUrls, Image, Restrictions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

mod private {
    use super::{CommonAlbumFields, FullAlbumFields, NonLocalAlbumFields};

    pub trait Sealed {}

    pub(super) trait CommonFields {
        fn common_fields(&self) -> &CommonAlbumFields;
    }

    pub(super) trait FullFields {
        fn full_fields(&self) -> &FullAlbumFields;
    }

    pub(super) trait NonLocalFields {
        fn non_local_fields(&self) -> &NonLocalAlbumFields;
    }
}

pub trait CommonAlbumInformation: private::Sealed {
    fn name(&self) -> &str;
    fn artists(&self) -> Vec<PartialArtist>;
    fn images(&self) -> &[Image];
    fn external_urls(&self) -> &ExternalUrls;
    fn available_markets(&self) -> &HashSet<CountryCode>;
    fn restrictions(&self) -> &Restrictions;
}

pub trait FullAlbumInformation: private::Sealed {
    // pub tracks: Page<PartialTrack>, // TODO: paging
    // TODO: the artist album thing with the album group field

    fn copyrights(&self) -> &[Copyright];
    fn external_ids(&self) -> &ExternalIds;
    fn genres(&self) -> &[String];
    fn label(&self) -> &str;
    fn popularity(&self) -> u32;
}

pub trait NonLocalAlbumInformation: private::Sealed {
    fn album_type(&self) -> AlbumType;
    fn id(&self) -> &str;
    fn release_date(&self) -> &str;
    fn release_date_precision(&self) -> DatePrecision;
}

impl<T> CommonAlbumInformation for T
where
    T: private::CommonFields + private::Sealed,
{
    fn name(&self) -> &str {
        &self.common_fields().name
    }

    fn artists(&self) -> Vec<PartialArtist> {
        self.common_fields()
            .artists
            .iter()
            .map(|artist_obj| artist_obj.to_owned().into())
            .collect()
    }

    fn images(&self) -> &[Image] {
        &self.common_fields().images
    }

    fn external_urls(&self) -> &ExternalUrls {
        &self.common_fields().external_urls
    }

    fn available_markets(&self) -> &HashSet<CountryCode> {
        &self.common_fields().available_markets
    }

    fn restrictions(&self) -> &Restrictions {
        &self.common_fields().restrictions
    }
}

impl<T> FullAlbumInformation for T
where
    T: private::FullFields + private::Sealed,
{
    fn copyrights(&self) -> &[Copyright] {
        &self.full_fields().copyrights
    }

    fn external_ids(&self) -> &ExternalIds {
        &self.full_fields().external_ids
    }

    fn genres(&self) -> &[String] {
        &self.full_fields().genres
    }

    fn label(&self) -> &str {
        &self.full_fields().label
    }

    fn popularity(&self) -> u32 {
        self.full_fields().popularity
    }
}

impl<T> NonLocalAlbumInformation for T
where
    T: private::NonLocalFields + private::Sealed,
{
    fn album_type(&self) -> AlbumType {
        self.non_local_fields().album_type
    }

    fn id(&self) -> &str {
        &self.non_local_fields().id
    }

    fn release_date(&self) -> &str {
        &self.non_local_fields().release_date
    }

    fn release_date_precision(&self) -> DatePrecision {
        self.non_local_fields().release_date_precision
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Album {
    Full(Box<FullAlbum>),
    Partial(Box<PartialAlbum>),
    Local(Box<LocalAlbum>),
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
    #[serde(rename = "type", deserialize_with = "obj_deserialize", skip_serializing)]
    item_type: TypeAlbum,

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
            (Some(non_local), Some(full)) => Self::Full(Box::new(FullAlbum {
                common: obj.common,
                non_local,
                full,
            })),

            (Some(non_local), None) => Self::Partial(Box::new(PartialAlbum {
                common: obj.common,
                non_local,
            })),

            (None, None) => Self::Local(Box::new(LocalAlbum { common: obj.common })),

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
        Self::Partial(Box::new(partial))
    }
}

impl From<FullAlbum> for Album {
    fn from(full: FullAlbum) -> Self {
        Self::Full(Box::new(full))
    }
}

impl From<LocalAlbum> for Album {
    fn from(local: LocalAlbum) -> Self {
        Self::Local(Box::new(local))
    }
}

impl From<Album> for FullAlbum {
    fn from(album: Album) -> Self {
        match album {
            Album::Full(full) => *full,

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
            Album::Partial(partial) => *partial,

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
            Album::Full(full) => LocalAlbum { common: full.common },
            Album::Partial(partial) => LocalAlbum { common: partial.common },

            Album::Local(local) => *local,
        }
    }
}

impl From<AlbumObject> for LocalAlbum {
    fn from(obj: AlbumObject) -> Self {
        LocalAlbum { common: obj.common }
    }
}

impl private::Sealed for FullAlbum {}
impl private::Sealed for PartialAlbum {}
impl private::Sealed for LocalAlbum {}

impl private::CommonFields for FullAlbum {
    fn common_fields(&self) -> &CommonAlbumFields {
        &self.common
    }
}

impl private::CommonFields for PartialAlbum {
    fn common_fields(&self) -> &CommonAlbumFields {
        &self.common
    }
}

impl private::CommonFields for LocalAlbum {
    fn common_fields(&self) -> &CommonAlbumFields {
        &self.common
    }
}

impl private::NonLocalFields for FullAlbum {
    fn non_local_fields(&self) -> &NonLocalAlbumFields {
        &self.non_local
    }
}

impl private::NonLocalFields for PartialAlbum {
    fn non_local_fields(&self) -> &NonLocalAlbumFields {
        &self.non_local
    }
}

impl private::FullFields for FullAlbum {
    fn full_fields(&self) -> &FullAlbumFields {
        &self.full
    }
}
