//! Everything related to tracks.

use super::{
    album::{AlbumObject, PartialAlbum},
    artist::{ArtistObject, PartialArtist},
    country_code::CountryCode,
    id::{Id, IdTrait, TrackId},
    object_type::{obj_deserialize, TypeTrack},
    ExternalIds, ExternalUrls, Restrictions,
};
use crate::util::duration_millis;
use serde::Deserialize;
use std::{collections::HashSet, time::Duration};

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

pub trait CommonTrackInformation: super::private::Sealed {
    fn name(&self) -> &str;
    fn artists(&self) -> Vec<PartialArtist>;
    fn track_number(&self) -> u32;
    fn disc_number(&self) -> u32;
    fn duration(&self) -> Duration;
    fn explicit(&self) -> bool;
    fn preview_url(&self) -> Option<&str>;
    fn external_urls(&self) -> &ExternalUrls;
    fn available_markets(&self) -> &HashSet<CountryCode>;
    fn is_playable(&self) -> Option<bool>;
    fn linked_from(&self) -> Option<&LinkedTrack>;
    fn restrictions(&self) -> &Restrictions;
}

pub trait FullTrackInformation: super::private::Sealed {
    fn album(&self) -> PartialAlbum;
    fn external_ids(&self) -> &ExternalIds;
    fn popularity(&self) -> u32;
}

pub trait NonLocalTrackInformation: super::private::Sealed {
    fn id(&self) -> &str;
}

impl<T> CommonTrackInformation for T
where
    T: private::CommonFields + super::private::Sealed,
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
    T: private::FullFields + super::private::Sealed,
{
    fn album(&self) -> PartialAlbum {
        self.full_fields().album.to_owned().into()
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
    T: private::NonLocalFields + super::private::Sealed,
{
    fn id(&self) -> &str {
        self.non_local_fields().id.id()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum Track {
    Full(Box<FullTrack>),
    Partial(Box<PartialTrack>),
    Local(Box<LocalTrack>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct CommonTrackFields {
    // basic information
    name: String,
    artists: Vec<ArtistObject>,
    track_number: u32,
    disc_number: u32,
    #[serde(rename = "duration_ms", with = "duration_millis")]
    duration: Duration,
    explicit: bool,
    preview_url: Option<String>,
    is_local: bool, // TODO: i don't like this field
    #[serde(default)]
    external_urls: ExternalUrls,
    #[serde(rename = "type", deserialize_with = "obj_deserialize", skip_serializing)]
    item_type: TypeTrack,

    // track relinking
    #[serde(default)]
    available_markets: HashSet<CountryCode>,
    is_playable: Option<bool>,
    linked_from: Option<LinkedTrack>, // TODO: this is ew
    #[serde(default)]
    restrictions: Restrictions,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct FullTrackFields {
    album: AlbumObject,
    #[serde(default)]
    external_ids: ExternalIds,
    popularity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct NonLocalTrackFields {
    id: Id<'static, TrackId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct FullTrack {
    common: CommonTrackFields,
    non_local: NonLocalTrackFields,
    full: FullTrackFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PartialTrack {
    common: CommonTrackFields,
    non_local: NonLocalTrackFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LocalTrack {
    common: CommonTrackFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LinkedTrack {
    #[serde(default)]
    pub external_urls: ExternalUrls,
    pub id: Id<'static, TrackId>,
}

impl From<TrackObject> for Track {
    fn from(obj: TrackObject) -> Self {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => Self::Full(Box::new(FullTrack {
                common: obj.common,
                non_local,
                full,
            })),

            (Some(non_local), None) => Self::Partial(Box::new(PartialTrack {
                common: obj.common,
                non_local,
            })),

            (None, None) => Self::Local(Box::new(LocalTrack { common: obj.common })),

            (non_local, full) => panic!(
                "impossible case trying to convert TrackObject into Track: non-local track fields is {:?} while full \
                 track fields is {:?}",
                non_local, full
            ),
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

impl From<Track> for FullTrack {
    fn from(track: Track) -> Self {
        match track {
            Track::Full(full) => *full,

            Track::Partial(_) => panic!("attempt to convert partial track into full track"),
            Track::Local(_) => panic!("attempt to convert local track into full track"),
        }
    }
}

impl From<TrackObject> for FullTrack {
    fn from(obj: TrackObject) -> Self {
        match (obj.non_local, obj.full) {
            (Some(non_local), Some(full)) => FullTrack {
                common: obj.common,
                non_local,
                full,
            },

            (non_local, full) => panic!(
                "attempt to convert non-full track object into full track (non-local track fields is {:?}, full track \
                 fields is {:?})",
                non_local, full
            ),
        }
    }
}

impl From<Track> for PartialTrack {
    fn from(track: Track) -> Self {
        match track {
            Track::Full(full) => PartialTrack {
                common: full.common,
                non_local: full.non_local,
            },
            Track::Partial(partial) => *partial,

            Track::Local(_) => panic!("attempt to convert local track into partial track"),
        }
    }
}

impl From<TrackObject> for PartialTrack {
    fn from(obj: TrackObject) -> Self {
        if let Some(non_local) = obj.non_local {
            PartialTrack {
                common: obj.common,
                non_local,
            }
        } else {
            panic!(
                "attempt to convert local track object into partial track (non-local track fields is {:?})",
                obj.non_local
            );
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

impl super::private::Sealed for FullTrack {}
impl super::private::Sealed for PartialTrack {}
impl super::private::Sealed for LocalTrack {}

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
