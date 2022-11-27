//! Everything related to tracks.
//!
//! Contains the three different kinds of tracks; [FullTrack], [PartialTrack] and [LocalTrack].
//!
//! - [FullTrack]: may contain all possible information about a track. Generally retrieved from the
//!   [track-](crate::client::UnscopedClient::track) and [tracks-functions](crate::client::UnscopedClient::tracks).
//! - [PartialTrack]: contains most information about a track. Generally retrieved as part of a response to, for
//!   example, an album listing (TODO: make a link to the album endpoint once it exists).
//! - [LocalTrack]: contains only the basic information about a track. Only retrieved through a playlist that contains
//!   local tracks.
//!
//! The track object Spotify returns from the API is not directly available.
//! TODO: have a way to write these objects into a serializer such that it outputs what the Spotify API returned

use std::{collections::HashSet, time::Duration};

use serde::{Deserialize, Serialize, Serializer};

use super::{
    album::PartialAlbum,
    artist::PartialArtist,
    country_code::CountryCode,
    id::{Id, IdTrait, TrackId},
    object_type::{object_type_serialize, TypeTrack},
    ExternalIds, ExternalUrls, Restrictions,
};
use crate::{error::ConversionError, util::duration_millis};

mod private {
    use super::{CommonTrackFields, FullTrackFields, NonLocalTrackFields};

    pub(super) trait CommonFields {
        fn common_fields(&self) -> &CommonTrackFields;
    }

    pub(super) trait FullFields {
        fn full_fields(&self) -> &FullTrackFields;
    }

    pub(super) trait NonLocalFields {
        fn non_local_fields(&self) -> &NonLocalTrackFields;
    }
}

/// Functions for retrieving information that is common to every track type.
pub trait CommonTrackInformation: crate::private::Sealed {
    /// The track's name.
    fn name(&self) -> &str;
    /// The artists of the track.
    fn artists(&self) -> &[PartialArtist];
    /// The track's number in its corresponding disc.
    fn track_number(&self) -> u32;
    /// The track's disc's number.
    fn disc_number(&self) -> u32;
    /// The track's duration.
    fn duration(&self) -> Duration;
    /// Whether or not the track is rated as explicit.
    fn explicit(&self) -> bool;
    /// An URL to a 30 second preview of the track.
    fn preview_url(&self) -> Option<&str>;
    /// The external URLs for the track.
    fn external_urls(&self) -> &ExternalUrls;
    /// The countries the track is available in.
    fn available_markets(&self) -> &HashSet<CountryCode>;
    /// Whether or not the track is playable.
    fn is_playable(&self) -> Option<bool>;
    /// When [track relinking](https://developer.spotify.com/documentation/general/guides/track-relinking-guide/) is
    /// applied, the original track this track is linked from.
    fn linked_from(&self) -> Option<&LinkedTrack>;
    /// The restrictions on the track.
    fn restrictions(&self) -> &Restrictions;
}

/// Functions for retrieving information only in full tracks.
pub trait FullTrackInformation: crate::private::Sealed {
    /// The album this track is in.
    fn album(&self) -> &PartialAlbum;
    /// The external IDs for the track.
    fn external_ids(&self) -> &ExternalIds;
    /// The track's popularity.
    fn popularity(&self) -> u32;
}

/// Functions for retrieving information that is available in non-local tracks.
pub trait NonLocalTrackInformation: crate::private::Sealed {
    /// The track's Spotify ID.
    fn id(&self) -> &str;
}

impl<T> CommonTrackInformation for T
where
    T: private::CommonFields + crate::private::Sealed,
{
    fn name(&self) -> &str {
        &self.common_fields().name
    }

    fn artists(&self) -> &[PartialArtist] {
        &self.common_fields().artists
    }

    fn track_number(&self) -> u32 {
        self.common_fields().track_number
    }

    fn disc_number(&self) -> u32 {
        self.common_fields().disc_number
    }

    fn duration(&self) -> Duration {
        self.common_fields().duration
    }

    fn explicit(&self) -> bool {
        self.common_fields().explicit
    }

    fn preview_url(&self) -> Option<&str> {
        self.common_fields().preview_url.as_deref()
    }

    fn external_urls(&self) -> &ExternalUrls {
        &self.common_fields().external_urls
    }

    fn available_markets(&self) -> &HashSet<CountryCode> {
        &self.common_fields().available_markets
    }

    fn is_playable(&self) -> Option<bool> {
        self.common_fields().is_playable
    }

    fn linked_from(&self) -> Option<&LinkedTrack> {
        self.common_fields().linked_from.as_ref()
    }

    fn restrictions(&self) -> &Restrictions {
        &self.common_fields().restrictions
    }
}

impl<T> FullTrackInformation for T
where
    T: private::FullFields + crate::private::Sealed,
{
    fn album(&self) -> &PartialAlbum {
        &self.full_fields().album
    }

    fn external_ids(&self) -> &ExternalIds {
        &self.full_fields().external_ids
    }

    fn popularity(&self) -> u32 {
        self.full_fields().popularity
    }
}

impl<T> NonLocalTrackInformation for T
where
    T: private::NonLocalFields + crate::private::Sealed,
{
    fn id(&self) -> &str {
        self.non_local_fields().id.id()
    }
}

/// An enum that encompasses all track types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Track {
    Full(Box<FullTrack>),
    Partial(Box<PartialTrack>),
    Local(Box<LocalTrack>),
}

/// This struct covers all the possible track responses from Spotify's API. It has a function that converts it into a
/// [Track], depending on which fields are set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TrackObject {
    /// Fields available in every track
    #[serde(flatten)]
    common: CommonTrackFields,

    /// Fields only in non-local tracks
    #[serde(flatten)]
    non_local: Option<NonLocalTrackFields>,

    /// Fields only in full tracks
    #[serde(flatten)]
    full: Option<FullTrackFields>,
}

/// This struct's only purpose is to make serializing more efficient by holding only references to its data. When
/// attempting to serialize a track object, its fields will be passed as references to this object which is then
/// serialized. This avoids having to clone the entire track in order to reconstruct a TrackObject.
#[derive(Serialize)]
struct TrackObjectRef<'a> {
    #[serde(flatten)]
    common: &'a CommonTrackFields,
    #[serde(flatten)]
    non_local: Option<&'a NonLocalTrackFields>,
    #[serde(flatten)]
    full: Option<&'a FullTrackFields>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CommonTrackFields {
    // basic information
    name: String,
    artists: Vec<PartialArtist>,
    track_number: u32,
    disc_number: u32,
    #[serde(rename = "duration_ms", with = "duration_millis")]
    duration: Duration,
    explicit: bool,
    preview_url: Option<String>,
    is_local: bool, // TODO: i don't like this field
    #[serde(default)]
    external_urls: ExternalUrls,
    #[serde(rename = "type", with = "object_type_serialize")]
    item_type: TypeTrack,

    // track relinking
    #[serde(default)]
    available_markets: HashSet<CountryCode>,
    is_playable: Option<bool>,
    linked_from: Option<LinkedTrack>, // TODO: this is ew
    #[serde(default)]
    restrictions: Restrictions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FullTrackFields {
    album: PartialAlbum,
    #[serde(default)]
    external_ids: ExternalIds,
    popularity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct NonLocalTrackFields {
    id: Id<'static, TrackId>,
}

/// A full track. Contains [full information](self::FullTrackInformation), in addition to all
/// [common](self::CommonTrackInformation) and [non-local](self::NonLocalTrackInformation) information about a track.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "TrackObject")]
pub struct FullTrack {
    common: CommonTrackFields,
    non_local: NonLocalTrackFields,
    full: FullTrackFields,
}

/// A partial track. Contains all [common](self::CommonTrackInformation) and [non-local](self::NonLocalTrackInformation)
/// information about a track.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "TrackObject")]
pub struct PartialTrack {
    common: CommonTrackFields,
    non_local: NonLocalTrackFields,
}

/// A local track. Contains only the information [common to every track](self::CommonTrackInformation).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "TrackObject")]
pub struct LocalTrack {
    common: CommonTrackFields,
}

/// Contains information about a linked track when
/// [track relinking](https://developer.spotify.com/documentation/general/guides/track-relinking-guide/) is applied
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedTrack {
    #[serde(default)]
    pub external_urls: ExternalUrls,
    pub id: Id<'static, TrackId>,
}

impl TryFrom<TrackObject> for Track {
    type Error = ConversionError;

    fn try_from(obj: TrackObject) -> Result<Self, Self::Error> {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => Ok(Self::Full(Box::new(FullTrack {
                common: obj.common,
                non_local,
                full,
            }))),

            (Some(non_local), None) => Ok(Self::Partial(Box::new(PartialTrack {
                common: obj.common,
                non_local,
            }))),

            (None, None) => Ok(Self::Local(Box::new(LocalTrack { common: obj.common }))),

            (non_local, full) => Err(ConversionError(
                format!(
                    "impossible case trying to convert TrackObject into Track: non-local track fields is {:?} while \
                     full track fields is {:?}",
                    non_local, full
                )
                .into(),
            )),
        }
    }
}

impl From<PartialTrack> for Track {
    fn from(partial: PartialTrack) -> Self {
        Self::Partial(Box::new(partial))
    }
}

impl From<FullTrack> for Track {
    fn from(full: FullTrack) -> Self {
        Self::Full(Box::new(full))
    }
}

impl From<LocalTrack> for Track {
    fn from(local: LocalTrack) -> Self {
        Self::Local(Box::new(local))
    }
}

impl TryFrom<Track> for FullTrack {
    type Error = ConversionError;

    fn try_from(track: Track) -> Result<Self, Self::Error> {
        match track {
            Track::Full(full) => Ok(*full),

            Track::Partial(_) => Err(ConversionError(
                "attempt to convert partial track into full track".into(),
            )),

            Track::Local(_) => Err(ConversionError("attempt to convert local track into full track".into())),
        }
    }
}

impl TryFrom<TrackObject> for FullTrack {
    type Error = ConversionError;

    fn try_from(obj: TrackObject) -> Result<Self, Self::Error> {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => Ok(FullTrack {
                common: obj.common,
                non_local,
                full,
            }),

            (non_local, full) => Err(ConversionError(
                format!(
                    "attempt to convert non-full track object into full track (non-local track fields is {:?}, full \
                     track fields is {:?})",
                    non_local, full
                )
                .into(),
            )),
        }
    }
}

impl TryFrom<Track> for PartialTrack {
    type Error = ConversionError;

    fn try_from(track: Track) -> Result<Self, Self::Error> {
        match track {
            Track::Full(full) => Ok(PartialTrack {
                common: full.common,
                non_local: full.non_local,
            }),

            Track::Partial(partial) => Ok(*partial),

            Track::Local(_) => Err(ConversionError(
                "attempt to convert local track into partial track".into(),
            )),
        }
    }
}

impl TryFrom<TrackObject> for PartialTrack {
    type Error = ConversionError;

    fn try_from(obj: TrackObject) -> Result<Self, Self::Error> {
        if let Some(non_local) = obj.non_local {
            Ok(PartialTrack {
                common: obj.common,
                non_local,
            })
        } else {
            Err(ConversionError(
                format!(
                    "attempt to convert local track object into partial track (non-local track fields is {:?})",
                    obj.non_local
                )
                .into(),
            ))
        }
    }
}

impl From<Track> for LocalTrack {
    fn from(track: Track) -> Self {
        match track {
            Track::Full(full) => LocalTrack { common: full.common },
            Track::Partial(partial) => LocalTrack { common: partial.common },
            Track::Local(local) => *local,
        }
    }
}

impl From<TrackObject> for LocalTrack {
    fn from(obj: TrackObject) -> Self {
        LocalTrack { common: obj.common }
    }
}

impl From<FullTrack> for TrackObject {
    fn from(value: FullTrack) -> Self {
        Self {
            common: value.common,
            non_local: Some(value.non_local),
            full: Some(value.full),
        }
    }
}

impl From<PartialTrack> for TrackObject {
    fn from(value: PartialTrack) -> Self {
        Self {
            common: value.common,
            non_local: Some(value.non_local),
            full: None,
        }
    }
}

impl From<LocalTrack> for TrackObject {
    fn from(value: LocalTrack) -> Self {
        Self {
            common: value.common,
            non_local: None,
            full: None,
        }
    }
}

impl crate::private::Sealed for FullTrack {}
impl crate::private::Sealed for PartialTrack {}
impl crate::private::Sealed for LocalTrack {}

impl private::CommonFields for FullTrack {
    fn common_fields(&self) -> &CommonTrackFields {
        &self.common
    }
}

impl private::CommonFields for PartialTrack {
    fn common_fields(&self) -> &CommonTrackFields {
        &self.common
    }
}

impl private::CommonFields for LocalTrack {
    fn common_fields(&self) -> &CommonTrackFields {
        &self.common
    }
}

impl private::NonLocalFields for FullTrack {
    fn non_local_fields(&self) -> &NonLocalTrackFields {
        &self.non_local
    }
}

impl private::NonLocalFields for PartialTrack {
    fn non_local_fields(&self) -> &NonLocalTrackFields {
        &self.non_local
    }
}

impl private::FullFields for FullTrack {
    fn full_fields(&self) -> &FullTrackFields {
        &self.full
    }
}

impl Serialize for Track {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Track::Full(full_track) => full_track.serialize(serializer),
            Track::Partial(partial_track) => partial_track.serialize(serializer),
            Track::Local(local_track) => local_track.serialize(serializer),
        }
    }
}

impl Serialize for FullTrack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TrackObjectRef {
            common: &self.common,
            non_local: Some(&self.non_local),
            full: Some(&self.full),
        }
        .serialize(serializer)
    }
}

impl Serialize for PartialTrack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TrackObjectRef {
            common: &self.common,
            non_local: Some(&self.non_local),
            full: None,
        }
        .serialize(serializer)
    }
}

impl Serialize for LocalTrack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TrackObjectRef {
            common: &self.common,
            non_local: None,
            full: None,
        }
        .serialize(serializer)
    }
}
