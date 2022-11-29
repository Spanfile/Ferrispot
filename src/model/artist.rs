//! Everything related to artists.
//!
//! Contains the three different kinds of artists; [FullArtist], [PartialArtist] and [LocalArtist].
//!
//! - [FullArtist]: may contain all possible information about an artist. Generally retrieved from the artist- and
//!   artists-endpoints (TODO: make links once implemented)
//! - [PartialArtist]: contains most information about an artist. Generally retrieved as part of a response to, for
//!   example, a [track listing](crate::client::UnscopedAsyncClient::track).
//! - [LocalArtist]: contains only the basic information about an artist. Only retrieved through a playlist that
//!   contains local tracks.
//!
//! The artist object Spotify returns from the API is not directly available.
//! TODO: have a way to write these objects into a serializer such that it outputs what the Spotify API returned

use serde::{Deserialize, Serialize, Serializer};

use super::{
    id::{ArtistId, Id, IdTrait},
    object_type::{object_type_serialize, TypeArtist},
    ExternalUrls, Image,
};
use crate::error::ConversionError;

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

/// Functions for retrieving information that is common to every artist type.
pub trait CommonArtistInformation: crate::private::Sealed {
    /// The artist's name.
    fn name(&self) -> &str;
    /// The external URLs for the artist.
    fn external_urls(&self) -> &ExternalUrls;
}

/// Functions for retrieving information only in full artists.
pub trait FullArtistInformation: crate::private::Sealed {
    /// Genres the artist is associated with.
    fn genres(&self) -> &[String];
    /// Images for the artist.
    fn images(&self) -> &[Image];
    /// The artist's popularity.
    fn popularity(&self) -> u32;
}

/// Functions for retrieving information that is available in non-local artists.
pub trait NonLocalArtistInformation: crate::private::Sealed {
    /// The artist's Spotify ID.
    fn id(&self) -> &str;
}

impl<T> CommonArtistInformation for T
where
    T: private::CommonFields + crate::private::Sealed,
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
    T: private::FullFields + crate::private::Sealed,
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
    T: private::NonLocalFields + crate::private::Sealed,
{
    fn id(&self) -> &str {
        self.non_local_fields().id.id()
    }
}

/// An enum that encompasses all artist types.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Artist {
    Full(Box<FullArtist>),
    Partial(Box<PartialArtist>),
    Local(Box<LocalArtist>),
}

/// This struct covers all the possible artist responses from Spotify's API. It has a function that converts it into an
/// [Artist], depending on which fields are set.
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

/// This struct's only purpose is to make serializing more efficient by holding only references to its data. When
/// attempting to serialize an artist object, its fields will be passed as references to this object which is then
/// serialized. This avoids having to clone the entire artist in order to reconstruct a ArtistObject.
#[derive(Serialize)]
struct ArtistObjectRef<'a> {
    #[serde(flatten)]
    common: &'a CommonArtistFields,
    #[serde(flatten)]
    non_local: Option<&'a NonLocalArtistFields>,
    #[serde(flatten)]
    full: Option<&'a FullArtistFields>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CommonArtistFields {
    name: String,
    #[serde(default)]
    external_urls: ExternalUrls,
    #[serde(rename = "type", with = "object_type_serialize")]
    item_type: TypeArtist,
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
    id: Id<'static, ArtistId>,
}

// TODO: artist equality should only compare their IDs. does track relinking also apply to artists?
/// A full artist. Contains [full information](self::FullArtistInformation), in addition to all
/// [common](self::CommonArtistInformation) and [non-local](self::NonLocalArtistInformation) information about an
/// artist.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "ArtistObject")]
pub struct FullArtist {
    common: CommonArtistFields,
    non_local: NonLocalArtistFields,
    full: FullArtistFields,
}

/// A partial artist. Contains all [common](self::CommonArtistInformation) and
/// [non-local](self::NonLocalArtistInformation) information about an artist.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "ArtistObject")]
pub struct PartialArtist {
    common: CommonArtistFields,
    non_local: NonLocalArtistFields,
}

// support your local artists
/// A local artist. Contains only the information [common to every album](self::CommonArtistInformation).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "ArtistObject")]
pub struct LocalArtist {
    common: CommonArtistFields,
}

impl TryFrom<ArtistObject> for Artist {
    type Error = ConversionError;

    fn try_from(obj: ArtistObject) -> Result<Self, Self::Error> {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => Ok(Self::Full(Box::new(FullArtist {
                common: obj.common,
                non_local,
                full,
            }))),

            (Some(non_local), None) => Ok(Self::Partial(Box::new(PartialArtist {
                common: obj.common,
                non_local,
            }))),

            (None, None) => Ok(Self::Local(Box::new(LocalArtist { common: obj.common }))),

            (non_local, full) => Err(ConversionError(
                format!(
                    "impossible case trying to convert ArtistObject into Artist: non-local artist fields is {:?} \
                     while full artist fields is {:?}",
                    non_local, full
                )
                .into(),
            )),
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

impl TryFrom<Artist> for FullArtist {
    type Error = ConversionError;

    fn try_from(artist: Artist) -> Result<Self, Self::Error> {
        match artist {
            Artist::Full(full) => Ok(*full),

            Artist::Partial(_) => Err(ConversionError(
                "attempt to convert partial artist into full artist".into(),
            )),

            Artist::Local(_) => Err(ConversionError(
                "attempt to convert local artist into full artist".into(),
            )),
        }
    }
}

impl TryFrom<ArtistObject> for FullArtist {
    type Error = ConversionError;

    fn try_from(obj: ArtistObject) -> Result<Self, Self::Error> {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => Ok(FullArtist {
                common: obj.common,
                non_local,
                full,
            }),

            (non_local, full) => Err(ConversionError(
                format!(
                    "attempt to convert non-full artist object into full artist (non-local artist fields is {:?}, \
                     full artist fields is {:?})",
                    non_local, full
                )
                .into(),
            )),
        }
    }
}

impl TryFrom<Artist> for PartialArtist {
    type Error = ConversionError;

    fn try_from(artist: Artist) -> Result<Self, Self::Error> {
        match artist {
            Artist::Full(full) => Ok(PartialArtist {
                common: full.common,
                non_local: full.non_local,
            }),

            Artist::Partial(partial) => Ok(*partial),

            Artist::Local(_) => Err(ConversionError(
                "attempt to convert local artist into partial artist".into(),
            )),
        }
    }
}

impl TryFrom<ArtistObject> for PartialArtist {
    type Error = ConversionError;

    fn try_from(obj: ArtistObject) -> Result<Self, Self::Error> {
        if let Some(non_local) = obj.non_local {
            Ok(PartialArtist {
                common: obj.common,
                non_local,
            })
        } else {
            Err(ConversionError(
                format!(
                    "attempt to convert local artist object into partial artist (non-local artist fields is {:?})",
                    obj.non_local
                )
                .into(),
            ))
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

impl From<FullArtist> for ArtistObject {
    fn from(value: FullArtist) -> Self {
        Self {
            common: value.common,
            non_local: Some(value.non_local),
            full: Some(value.full),
        }
    }
}

impl From<PartialArtist> for ArtistObject {
    fn from(value: PartialArtist) -> Self {
        Self {
            common: value.common,
            non_local: Some(value.non_local),
            full: None,
        }
    }
}

impl From<LocalArtist> for ArtistObject {
    fn from(value: LocalArtist) -> Self {
        Self {
            common: value.common,
            non_local: None,
            full: None,
        }
    }
}

impl crate::private::Sealed for FullArtist {}
impl crate::private::Sealed for PartialArtist {}
impl crate::private::Sealed for LocalArtist {}

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

impl Serialize for Artist {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Artist::Full(full_artist) => full_artist.serialize(serializer),
            Artist::Partial(partial_artist) => partial_artist.serialize(serializer),
            Artist::Local(local_artist) => local_artist.serialize(serializer),
        }
    }
}

impl Serialize for FullArtist {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ArtistObjectRef {
            common: &self.common,
            non_local: Some(&self.non_local),
            full: Some(&self.full),
        }
        .serialize(serializer)
    }
}

impl Serialize for PartialArtist {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ArtistObjectRef {
            common: &self.common,
            non_local: Some(&self.non_local),
            full: None,
        }
        .serialize(serializer)
    }
}

impl Serialize for LocalArtist {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ArtistObjectRef {
            common: &self.common,
            non_local: None,
            full: None,
        }
        .serialize(serializer)
    }
}
