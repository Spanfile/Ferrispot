use super::{
    id::{ArtistId, Id},
    object_type::{obj_deserialize, TypeArtist},
    ExternalUrls, Image,
};
use serde::Deserialize;

mod private {
    use super::{CommonArtistFields, FullArtistFields, NonLocalArtistFields};

    pub(super) trait CommonFields {
        fn common_fields(&self) -> &CommonArtistFields;
    }

    pub(super) trait FullFields {
        fn full_fields(&self) -> &FullArtistFields;
    }

    pub(super) trait NonLocalFields {
        fn non_local_fields(&self) -> &NonLocalArtistFields;
    }
}

pub trait CommonArtistInformation: super::private::Sealed {
    fn name(&self) -> &str;
    fn external_urls(&self) -> &ExternalUrls;
}

pub trait FullArtistInformation: super::private::Sealed {
    fn genres(&self) -> &[String];
    fn images(&self) -> &[Image];
    fn popularity(&self) -> u32;
}

pub trait NonLocalArtistInformation: super::private::Sealed {
    fn id(&self) -> &str;
}

impl<T> CommonArtistInformation for T
where
    T: private::CommonFields + super::private::Sealed,
{
    fn name(&self) -> &str {
        &self.common_fields().name
    }

    fn external_urls(&self) -> &ExternalUrls {
        &self.common_fields().external_urls
    }
}

impl<T> FullArtistInformation for T
where
    T: private::FullFields + super::private::Sealed,
{
    fn genres(&self) -> &[String] {
        &self.full_fields().genres
    }

    fn images(&self) -> &[Image] {
        &self.full_fields().images
    }

    fn popularity(&self) -> u32 {
        self.full_fields().popularity
    }
}

impl<T> NonLocalArtistInformation for T
where
    T: private::NonLocalFields + super::private::Sealed,
{
    fn id(&self) -> &str {
        self.non_local_fields().id.id()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Artist {
    Full(Box<FullArtist>),
    Partial(Box<PartialArtist>),
    Local(Box<LocalArtist>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct CommonArtistFields {
    name: String,
    #[serde(default)]
    external_urls: ExternalUrls,
    #[serde(rename = "type", deserialize_with = "obj_deserialize", skip_serializing)]
    item_type: TypeArtist,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct FullArtistFields {
    // followers: Followers,
    genres: Vec<String>,
    images: Vec<Image>,
    popularity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct NonLocalArtistFields {
    id: ArtistId<'static>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FullArtist {
    common: CommonArtistFields,
    non_local: NonLocalArtistFields,
    full: FullArtistFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PartialArtist {
    common: CommonArtistFields,
    non_local: NonLocalArtistFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LocalArtist {
    common: CommonArtistFields,
}

impl From<ArtistObject> for Artist {
    fn from(obj: ArtistObject) -> Self {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => Self::Full(Box::new(FullArtist {
                common: obj.common,
                non_local,
                full,
            })),

            (Some(non_local), None) => Self::Partial(Box::new(PartialArtist {
                common: obj.common,
                non_local,
            })),

            (None, None) => Self::Local(Box::new(LocalArtist { common: obj.common })),

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
        Self::Partial(Box::new(partial))
    }
}

impl From<FullArtist> for Artist {
    fn from(full: FullArtist) -> Self {
        Self::Full(Box::new(full))
    }
}

impl From<LocalArtist> for Artist {
    fn from(local: LocalArtist) -> Self {
        Self::Local(Box::new(local))
    }
}

impl From<Artist> for FullArtist {
    fn from(artist: Artist) -> Self {
        match artist {
            Artist::Full(full) => *full,

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
            Artist::Partial(partial) => *partial,

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
            Artist::Full(full) => LocalArtist { common: full.common },
            Artist::Partial(partial) => LocalArtist { common: partial.common },

            Artist::Local(local) => *local,
        }
    }
}

impl From<ArtistObject> for LocalArtist {
    fn from(obj: ArtistObject) -> Self {
        LocalArtist { common: obj.common }
    }
}

impl super::private::Sealed for FullArtist {}
impl super::private::Sealed for PartialArtist {}
impl super::private::Sealed for LocalArtist {}

impl private::CommonFields for FullArtist {
    fn common_fields(&self) -> &CommonArtistFields {
        &self.common
    }
}

impl private::CommonFields for PartialArtist {
    fn common_fields(&self) -> &CommonArtistFields {
        &self.common
    }
}

impl private::CommonFields for LocalArtist {
    fn common_fields(&self) -> &CommonArtistFields {
        &self.common
    }
}

impl private::NonLocalFields for FullArtist {
    fn non_local_fields(&self) -> &NonLocalArtistFields {
        &self.non_local
    }
}

impl private::NonLocalFields for PartialArtist {
    fn non_local_fields(&self) -> &NonLocalArtistFields {
        &self.non_local
    }
}

impl private::FullFields for FullArtist {
    fn full_fields(&self) -> &FullArtistFields {
        &self.full
    }
}
