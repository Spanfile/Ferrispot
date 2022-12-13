//! Everything related to albums.
//!
//! Contains the three different kinds of albums; [FullAlbum], [PartialAlbum] and [LocalAlbum].
//!
//! - [FullAlbum]: may contain all possible information about an album. Generally retrieved from the album- and
//!   albums-endpoints (TODO: make links once implemented)
//! - [PartialAlbum]: contains most information about an album. Generally retrieved as part of a response to, for
//!   example, an artist listing (TODO: make a link to the artist endpoint once it exists).
//! - [LocalAlbum]: contains only the basic information about an album. Only retrieved through a playlist that contains
//!   local tracks.
//!
//! Additionally, there is the [Album] enum that encompasses all three kinds of albums.
//!
//! The album object Spotify returns from the API is not directly available. The three album objects, or the [Album]
//! enum, may be serialized to get almost all of the original API response back. The model strips certain unnecessary or
//! redundant fields from the response.
//!
//! # Album equality
//!
//! Two albums are considered equal when their Spotify IDs are the same. However, since [LocalAlbum] doesn't have a
//! Spotify ID, it resorts to comparing all available fields.

use std::{collections::HashSet, marker::PhantomData};

use serde::{Deserialize, Serialize, Serializer};

pub(crate) use self::private::{AlbumObject, CommonAlbumFields, FullAlbumFields, NonLocalAlbumFields};
use super::{
    artist::PartialArtist,
    country_code::CountryCode,
    id::{AlbumId, Id, IdTrait},
    page::{Page, PageInformation, PageObject},
    track::{PartialTrack, TrackObject},
    Copyright, DatePrecision, ExternalIds, ExternalUrls, Image, Restrictions,
};
use crate::error::ConversionError;

mod private {
    use std::collections::HashSet;

    use serde::{Deserialize, Serialize};

    use crate::model::{
        album::{AlbumTracks, AlbumType},
        artist::PartialArtist,
        id::{AlbumId, Id},
        object_type::{object_type_serialize, TypeAlbum},
        Copyright, CountryCode, DatePrecision, ExternalIds, ExternalUrls, Image, Restrictions,
    };

    pub(super) trait CommonFields {
        fn common_fields(&self) -> &CommonAlbumFields;
    }

    pub(super) trait FullFields {
        fn full_fields(&self) -> &FullAlbumFields;
    }

    pub(super) trait NonLocalFields {
        fn non_local_fields(&self) -> &NonLocalAlbumFields;
    }

    /// This struct covers all the possible album responses from Spotify's API. It has a function that converts it into
    /// an [Album], depending on which fields are set.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AlbumObject {
        /// Fields available in every album
        #[serde(flatten)]
        pub(crate) common: CommonAlbumFields,

        /// Fields only in non-local albums
        #[serde(flatten)]
        pub(crate) non_local: Option<NonLocalAlbumFields>,

        /// Fields only in full albums
        #[serde(flatten)]
        pub(crate) full: Option<FullAlbumFields>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub(crate) struct CommonAlbumFields {
        // basic information
        pub(crate) name: String,
        pub(crate) artists: Vec<PartialArtist>,
        pub(crate) images: Vec<Image>,
        #[serde(default)]
        pub(crate) external_urls: ExternalUrls,
        #[serde(rename = "type", with = "object_type_serialize")]
        pub(crate) item_type: TypeAlbum,

        // track relinking
        #[serde(default)]
        pub(crate) available_markets: HashSet<CountryCode>,
        #[serde(default)]
        pub(crate) restrictions: Restrictions,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub(crate) struct FullAlbumFields {
        pub(crate) copyrights: Vec<Copyright>,
        pub(crate) external_ids: ExternalIds,
        pub(crate) genres: Vec<String>,
        pub(crate) label: String,
        pub(crate) popularity: u32,
        pub(crate) tracks: AlbumTracks,
        // TODO: the artist album thing with the album group field
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub(crate) struct NonLocalAlbumFields {
        pub(crate) album_type: AlbumType,
        pub(crate) id: Id<'static, AlbumId>,
        pub(crate) release_date: String, // TODO: proper date type pls
        pub(crate) release_date_precision: DatePrecision,
    }
}

/// Functions for retrieving information that is common to every album type.
pub trait CommonAlbumInformation: crate::private::Sealed {
    /// The album's name.
    fn name(&self) -> &str;
    /// The artists of the album.
    fn artists(&self) -> &[PartialArtist];
    /// The images for the album.
    fn images(&self) -> &[Image];
    /// The external URLs for the album.
    fn external_urls(&self) -> &ExternalUrls;
    /// The countries the album is available in.
    fn available_markets(&self) -> &HashSet<CountryCode>;
    /// The restrictions on the album.
    fn restrictions(&self) -> &Restrictions;
}

/// Functions for retrieving information only in full albums.
pub trait FullAlbumInformation: crate::private::Sealed {
    // TODO: the artist album thing with the album group field

    /// The tracks in the album.
    fn tracks(&self) -> Page<AlbumTracks, PartialTrack>;
    /// The album's copyrights.
    fn copyrights(&self) -> &[Copyright];
    /// The external IDs for the album.
    fn external_ids(&self) -> &ExternalIds;
    /// The album's genres.
    fn genres(&self) -> &[String];
    /// The album's label.
    fn label(&self) -> &str;
    /// The album's popularity.
    fn popularity(&self) -> u32;
}

/// Functions for retrieving information that is available in non-local albums.
pub trait NonLocalAlbumInformation: crate::private::Sealed {
    /// The album's type.
    fn album_type(&self) -> AlbumType;
    /// The album's Spotify ID.
    fn id(&self) -> Id<'_, AlbumId>;
    /// The album's release date.
    fn release_date(&self) -> &str;
    /// The album's release date's precision.
    fn release_date_precision(&self) -> DatePrecision;
}

impl<T> CommonAlbumInformation for T
where
    T: private::CommonFields + crate::private::Sealed,
{
    fn name(&self) -> &str {
        &self.common_fields().name
    }

    fn artists(&self) -> &[PartialArtist] {
        &self.common_fields().artists
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
    T: private::FullFields + crate::private::Sealed,
{
    fn tracks(&self) -> Page<AlbumTracks, PartialTrack> {
        Page {
            inner: self.full_fields().tracks.clone(),
            phantom: PhantomData,
        }
    }

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
    T: private::NonLocalFields + crate::private::Sealed,
{
    fn album_type(&self) -> AlbumType {
        self.non_local_fields().album_type
    }

    fn id(&self) -> Id<'_, AlbumId> {
        self.non_local_fields().id.as_borrowed()
    }

    fn release_date(&self) -> &str {
        &self.non_local_fields().release_date
    }

    fn release_date_precision(&self) -> DatePrecision {
        self.non_local_fields().release_date_precision
    }
}

/// An enum that encompasses all album types.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Album {
    Full(Box<FullAlbum>),
    Partial(Box<PartialAlbum>),
    Local(Box<LocalAlbum>),
}

/// This struct's only purpose is to make serializing more efficient by holding only references to its data. When
/// attempting to serialize an album object, its fields will be passed as references to this object which is then
/// serialized. This avoids having to clone the entire album in order to reconstruct a AlbumObject.
#[derive(Serialize)]
struct AlbumObjectRef<'a> {
    #[serde(flatten)]
    common: &'a CommonAlbumFields,
    #[serde(flatten)]
    non_local: Option<&'a NonLocalAlbumFields>,
    #[serde(flatten)]
    full: Option<&'a FullAlbumFields>,
}

/// A page of tracks in an album.
///
/// This object is retrieved only through the [tracks](FullAlbumInformation::tracks)-function. You won't be interacting
/// objects of this type directly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[doc(hidden)]
pub struct AlbumTracks {
    #[serde(flatten)]
    page: PageObject<TrackObject>,
}

/// A full album. Contains [full information](self::FullAlbumInformation), in addition to all
/// [common](self::CommonAlbumInformation) and [non-local](self::NonLocalAlbumInformation) information about an album.
#[derive(Debug, Clone, Eq, Deserialize)]
#[serde(try_from = "AlbumObject")]
pub struct FullAlbum {
    common: CommonAlbumFields,
    non_local: NonLocalAlbumFields,
    full: FullAlbumFields,
    // TODO: there's a total_tracks field in I think common fields but make sure anyways and add it
}

/// A partial album. Contains all [common](self::CommonAlbumInformation) and [non-local](self::NonLocalAlbumInformation)
/// information about an album.
#[derive(Debug, Clone, Eq, Deserialize)]
#[serde(try_from = "AlbumObject")]
pub struct PartialAlbum {
    common: CommonAlbumFields,
    non_local: NonLocalAlbumFields,
}

/// A local album. Contains only the information [common to every album](self::CommonAlbumInformation).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "AlbumObject")]
pub struct LocalAlbum {
    common: CommonAlbumFields,
}

/// An album's type.
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

impl PartialEq for FullAlbum {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for PartialAlbum {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq<PartialAlbum> for FullAlbum {
    fn eq(&self, other: &PartialAlbum) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq<LocalAlbum> for FullAlbum {
    fn eq(&self, other: &LocalAlbum) -> bool {
        self.common == other.common
    }
}

impl PartialEq<FullAlbum> for PartialAlbum {
    fn eq(&self, other: &FullAlbum) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq<LocalAlbum> for PartialAlbum {
    fn eq(&self, other: &LocalAlbum) -> bool {
        self.common == other.common
    }
}

impl PartialEq<FullAlbum> for LocalAlbum {
    fn eq(&self, other: &FullAlbum) -> bool {
        self.common == other.common
    }
}

impl PartialEq<PartialAlbum> for LocalAlbum {
    fn eq(&self, other: &PartialAlbum) -> bool {
        self.common == other.common
    }
}

impl TryFrom<AlbumObject> for Album {
    type Error = ConversionError;

    fn try_from(obj: AlbumObject) -> Result<Self, Self::Error> {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => Ok(Self::Full(Box::new(FullAlbum {
                common: obj.common,
                non_local,
                full,
            }))),

            (Some(non_local), None) => Ok(Self::Partial(Box::new(PartialAlbum {
                common: obj.common,
                non_local,
            }))),

            (None, None) => Ok(Self::Local(Box::new(LocalAlbum { common: obj.common }))),

            (non_local, full) => Err(ConversionError(
                format!(
                    "impossible case trying to convert AlbumObject into Album: non-local album fields is \
                     {non_local:?} while full album fields is {full:?}"
                )
                .into(),
            )),
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

impl TryFrom<Album> for FullAlbum {
    type Error = ConversionError;

    fn try_from(album: Album) -> Result<Self, Self::Error> {
        match album {
            Album::Full(full) => Ok(*full),

            Album::Partial(_) => Err(ConversionError(
                "attempt to convert partial album into full album".into(),
            )),

            Album::Local(_) => Err(ConversionError("attempt to convert local album into full album".into())),
        }
    }
}

impl TryFrom<AlbumObject> for FullAlbum {
    type Error = ConversionError;

    fn try_from(obj: AlbumObject) -> Result<Self, Self::Error> {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => Ok(FullAlbum {
                common: obj.common,
                non_local,
                full,
            }),

            (non_local, full) => Err(ConversionError(
                format!(
                    "attempt to convert non-full album object into full album (non-local album fields is \
                     {non_local:?}, full album fields is {full:?})"
                )
                .into(),
            )),
        }
    }
}

impl TryFrom<Album> for PartialAlbum {
    type Error = ConversionError;

    fn try_from(album: Album) -> Result<Self, Self::Error> {
        match album {
            Album::Full(full) => Ok(PartialAlbum {
                common: full.common,
                non_local: full.non_local,
            }),

            Album::Partial(partial) => Ok(*partial),

            Album::Local(_) => Err(ConversionError(
                "attempt to convert local album into partial album".into(),
            )),
        }
    }
}

impl TryFrom<AlbumObject> for PartialAlbum {
    type Error = ConversionError;

    fn try_from(obj: AlbumObject) -> Result<Self, Self::Error> {
        if let Some(non_local) = obj.non_local {
            Ok(PartialAlbum {
                common: obj.common,
                non_local,
            })
        } else {
            Err(ConversionError(
                format!(
                    "attempt to convert local album object into partial album (non-local album fields is {:?})",
                    obj.non_local
                )
                .into(),
            ))
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

impl From<FullAlbum> for AlbumObject {
    fn from(value: FullAlbum) -> Self {
        Self {
            common: value.common,
            non_local: Some(value.non_local),
            full: Some(value.full),
        }
    }
}

impl From<PartialAlbum> for AlbumObject {
    fn from(value: PartialAlbum) -> Self {
        Self {
            common: value.common,
            non_local: Some(value.non_local),
            full: None,
        }
    }
}

impl From<LocalAlbum> for AlbumObject {
    fn from(value: LocalAlbum) -> Self {
        Self {
            common: value.common,
            non_local: None,
            full: None,
        }
    }
}

impl crate::private::Sealed for FullAlbum {}
impl crate::private::Sealed for PartialAlbum {}
impl crate::private::Sealed for LocalAlbum {}
impl crate::private::Sealed for AlbumTracks {}

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

impl PageInformation<PartialTrack> for AlbumTracks {
    type Items = Vec<PartialTrack>;

    fn items(&self) -> Self::Items {
        self.page.items()
    }

    fn take_items(self) -> Self::Items {
        self.page.take_items()
    }

    fn next(self) -> Option<String> {
        <PageObject<TrackObject> as PageInformation<PartialTrack>>::next(self.page)
    }
}

impl Serialize for Album {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Album::Full(full_album) => full_album.serialize(serializer),
            Album::Partial(partial_album) => partial_album.serialize(serializer),
            Album::Local(local_album) => local_album.serialize(serializer),
        }
    }
}

impl Serialize for FullAlbum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        AlbumObjectRef {
            common: &self.common,
            non_local: Some(&self.non_local),
            full: Some(&self.full),
        }
        .serialize(serializer)
    }
}

impl Serialize for PartialAlbum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        AlbumObjectRef {
            common: &self.common,
            non_local: Some(&self.non_local),
            full: None,
        }
        .serialize(serializer)
    }
}

impl Serialize for LocalAlbum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        AlbumObjectRef {
            common: &self.common,
            non_local: None,
            full: None,
        }
        .serialize(serializer)
    }
}

// TODO: unit tests for all the various functions here. deserializing, serializing, equality between tracks, conversion
// between tracks
