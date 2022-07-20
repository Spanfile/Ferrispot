//! Contains objects that represent all Spotify IDs.
//!
//! # Working with IDs
//!
//! There are three different kinds of Spotify IDs that this module can handle:
//! - URIs: `spotify:track:2pDPOMX0kWA7kcPBcDCQBu`
//! - URLs: `https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu`. The URL may contain any query prameters.
//! - Bare IDs: `2pDPOMX0kWA7kcPBcDCQBu`
//!
//! The IDs are split into two kinds: [PlayableItem] and [PlayableContext]. Playable items are individual items that can
//! be played; tracks or episodes. Playable contexts are collections of one or more playable items; artists, albums,
//! playlists or shows. These two kinds are grouped into one common [SpotifyId] that encompasses all of them.
//!
//! ## The core ID type
//!
//! At the core, the [Id] struct contains a single Spotify ID of any kind. The struct is generic over the kind of ID it
//! contains using the various type structs that implement the [ItemTypeId]-trait. This type is the only kind that
//! supports parsing from a bare ID, since you have to specify the ID type yourself. It is possible to let the parser
//! figure out the ID's type from the input by using either [PlayableItem and
//! PlayableContext](self#playableitem-and-playablecontext) or [SpotifyId](self#spotifyid).
//!
//! You may parse any ID string into an [Id] by specifying the type in the [Id]'s type parameter:
//!
//! ```
//! # use ferrispot::model::id::{AlbumId, Id, TrackId};
//! # use ferrispot::prelude::*;
//! let uri_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
//! let url_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
//! let bare_track_string = "2pDPOMX0kWA7kcPBcDCQBu";
//! let bare_album_string = "0tDsHtvN9YNuZjlqHvDY2P";
//!
//! let id_from_uri = Id::<TrackId>::from_uri(uri_string).unwrap();
//! let id_from_url = Id::<TrackId>::from_url(url_string).unwrap();
//! let track_id_from_bare = Id::<TrackId>::from_bare(bare_track_string).unwrap();
//! let album_id_from_bare = Id::<AlbumId>::from_bare(bare_album_string).unwrap();
//! ```
//!
//! Attempting to parse an ID of the wrong type will fail:
//!
//! ```
//! # use ferrispot::model::id::{Id, AlbumId};
//! # use ferrispot::prelude::*;
//! let uri_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
//!
//! assert!(Id::<AlbumId>::from_uri(uri_string).is_err());
//! ```
//!
//! [Id] internally stores the originally given string in a [Cow] which helps avoid string allocations in certain cases.
//! For example, most Spotify API endpoints require an input ID in the URI form, so if the ID is originally parsed from
//! an URI, the entire original string can be used instead of having to allocate a new string. You may also retrieve the
//! ID in an URL form, so if the original string was also an URL, no new strings are allocated.
//!
//! ## `PlayableItem` and `PlayableContext`
//!
//! [PlayableItem] and [PlayableContext] are wrapper enums that encompass an [Id] in their variants. Their benefit is
//! that they simplify ID parsing by allowing the parser figure out the correct type for the [Id] from the input ID
//! string. Their downside is that they do not support parsing from a bare ID, since it is impossible to figure out
//! which kind of ID it is.
//!
//! ```
//! let track_uri_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
//! let track_url_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
//! let album_uri_string = "spotify:album:0tDsHtvN9YNuZjlqHvDY2P";
//! let album_url_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P"
//!
//! // both of these are PlayableItem::Track(Id::<TrackId>)
//! let track_from_uri = PlayableItem::from_uri(track_uri_string).unwrap();
//! let track_from_url = PlayableItem::from_url(track_url_string).unrwap();
//!
//! // both of these are PlayableContext::Album(Id::<AlbumId>)
//! let album_from_uri = PlayableContext::from_uri(album_uri_string).unwrap();
//! let album_from_url = PlayableContext::from_url(album_url_string).unrwap();
//! ```
//!
//! Attempting to parse an ID of the wrong type will fail:
//!
//! ```
//! # use ferrispot::model::id::{Id, AlbumId};
//! # use ferrispot::prelude::*;
//! let uri_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
//!
//! assert!(PlayableContext::from_uri(uri_string).is_err());
//! ```
//!
//! ## `SpotifyId`
//!
//! [SpotifyId] encompasses both [PlayableItem] and [PlayableContext] into one type which lets you parse any kind of ID
//! into a single type. Like with [PlayableItem] and [PlayableContext], bare IDs are not supported.
//!
//! ```
//! let track_uri_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
//! let album_url_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P"
//! let artist_url_string = "https://open.spotify.com/artist/6pNgnvzBa6Bthsv8SrZJYl";
//!
//! // SpotifyId::Item(PlayableItem::Track(Id::<TrackId>))
//! let track_from_uri = SpotifyId::from_uri(track_uri_string).unwrap();
//!
//! // SpotifyId::Context(PlayableContext::Album(Id::<AlbumId>))
//! let album_from_url = SpotifyId::from_url(album_url_string).unwrap();
//!
//! // SpotifyId::Context(PlayableContext::Artist(Id::<ArtistId>))
//! let artist_from_url = SpotifyId::from_url(artist_url_string).unwrap();
//! ```

use super::ItemType;
use crate::error::{IdError, Result};
use serde::{
    de::{self, Visitor},
    Deserialize,
};
use std::{borrow::Cow, marker::PhantomData};

const ID_LENGTH: usize = 22; // I hope Spotify never changes this length
const URL_PREFIX: &str = "https://open.spotify.com/";
const URI_PREFIX: &str = "spotify:";

mod private {
    pub trait Sealed {}
}

/// Used to signify a type that describes a kind of Spotify ID.
///
/// See the [module-level docs](self) for information on how to work with IDs.
pub trait ItemTypeId: private::Sealed {
    /// The Spotify catalog item type this type corresponds to.
    const ITEM_TYPE: ItemType;
}

/// Functions common to all ID types.
///
/// See the [module-level docs](self) for information on how to work with IDs.
pub trait IdTrait<'a>: private::Sealed {
    /// Returns this ID as a bare Spotify ID.
    fn id(&'a self) -> &'a str;

    /// Returns this ID as a Spotify URI.
    ///
    /// This function returns a [Cow], since it allows the function to avoid needlessly allocating a new string if the
    /// original ID string this ID was constructed from is already an URI. Otherwise, it will allocate a new URI string.
    fn uri(&'a self) -> Cow<'a, str>;

    /// Returns this ID as a Spotify URL.
    ///
    /// This function returns a [Cow], since it allows the function to avoid needlessly allocating a new string if the
    /// original ID string this ID was constructed from is already an URL. Otherwise, it will allocate a new URL string.
    fn url(&'a self) -> Cow<'a, str>;
}

/// Trait for parsing any string-looking type that contains a Spotify URI into an ID type.
///
/// See the [module-level docs](self) for information on how to work with IDs.
pub trait IdFromUri<'a>: private::Sealed
where
    Self: Sized,
{
    /// Parses a Spotify URI string into an ID.
    fn from_uri<C>(uri: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>;
}

/// Trait for parsing any string-looking type that contains a Spotify URL into an ID type.
///
/// See the [module-level docs](self) for information on how to work with IDs.
pub trait IdFromUrl<'a>: private::Sealed
where
    Self: Sized,
{
    /// Parses a Spotify URL into an ID.
    fn from_url<C>(url: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>;
}

/// Trait for parsing any string-looking type that contains a bare Spotify ID into an ID type.
///
/// See the [module-level docs](self) for information on how to work with IDs.
pub trait IdFromBare<'a>: private::Sealed
where
    Self: Sized,
{
    /// Parses a bare Spotify ID into an ID.
    fn from_bare<C>(bare: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>;
}

/// Common type that contains a single Spotify ID of a certain kind. The generic type parameter `T` is used to signify
/// which kind of ID it contains.
///
/// See the [module-level docs](self) for information on how to work with IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id<'a, T>
where
    T: ItemTypeId,
{
    value: Cow<'a, str>,
    kind: IdKind,
    phantom: PhantomData<&'a T>,
}

/// Specifies a kind of ID.
#[derive(Debug, Clone, PartialEq, Eq)]
enum IdKind {
    /// The ID is a Spotify URI. The field is the index of the ID in the original string.
    Uri(usize),
    /// The ID is a Spotify URL. The field is the index of the ID in the original string.
    Url(usize),
    /// The ID is a bare Spotify ID.
    Bare,
}

/// Common type for all Spotify IDs.
///
/// See the [module-level docs](self) for information on how to work with IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpotifyId<'a> {
    Item(PlayableItem<'a>),
    Context(PlayableContext<'a>),
}

/// Common type for all individually playable IDs.
///
/// See the [module-level docs](self) for information on how to work with IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayableItem<'a> {
    Track(Id<'a, TrackId>),
    Episode(Id<'a, EpisodeId>),
}

/// Common type for all playable context IDs.
///
/// See the [module-level docs](self) for information on how to work with IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayableContext<'a> {
    Artist(Id<'a, ArtistId>),
    Album(Id<'a, AlbumId>),
    Playlist(Id<'a, PlaylistId>),
    Show(Id<'a, ShowId>),
}

/// Signifies a track ID.
///
/// See the [module-level docs](self) for information on how to work with IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackId;

/// Signifies an episode ID.
///
/// See the [module-level docs](self) for information on how to work with IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeId;

/// Signifies an artist ID.
///
/// See the [module-level docs](self) for information on how to work with IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtistId;

/// Signifies an album ID.
///
/// See the [module-level docs](self) for information on how to work with IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumId;

/// Signifies a playlist ID.
///
/// See the [module-level docs](self) for information on how to work with IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistId;

/// Signifies a show ID.
///
/// See the [module-level docs](self) for information on how to work with IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowId;

impl<'a, T> private::Sealed for Id<'a, T> where T: ItemTypeId {}
impl private::Sealed for TrackId {}
impl private::Sealed for EpisodeId {}
impl private::Sealed for ArtistId {}
impl private::Sealed for AlbumId {}
impl private::Sealed for PlaylistId {}
impl private::Sealed for ShowId {}

impl private::Sealed for SpotifyId<'_> {}
impl private::Sealed for PlayableItem<'_> {}
impl private::Sealed for PlayableContext<'_> {}

impl ItemTypeId for TrackId {
    const ITEM_TYPE: ItemType = ItemType::Track;
}

impl ItemTypeId for EpisodeId {
    const ITEM_TYPE: ItemType = ItemType::Episode;
}

impl ItemTypeId for ArtistId {
    const ITEM_TYPE: ItemType = ItemType::Artist;
}

impl ItemTypeId for AlbumId {
    const ITEM_TYPE: ItemType = ItemType::Album;
}

impl ItemTypeId for PlaylistId {
    const ITEM_TYPE: ItemType = ItemType::Playlist;
}

impl ItemTypeId for ShowId {
    const ITEM_TYPE: ItemType = ItemType::Show;
}

impl<'a, T> IdFromUri<'a> for Id<'a, T>
where
    T: ItemTypeId,
{
    fn from_uri<C>(uri: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let uri: Cow<'a, str> = uri.into();
        let (item_type, id_index) = parse_item_type_and_id_from_uri(&uri)?;

        if item_type == T::ITEM_TYPE {
            Ok(Self {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            })
        } else {
            Err(IdError::WrongItemType(item_type).into())
        }
    }
}

impl<'a, T> IdFromUrl<'a> for Id<'a, T>
where
    T: ItemTypeId,
{
    fn from_url<C>(url: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let uri: Cow<'a, str> = url.into();
        let (item_type, id_index) = parse_item_type_and_id_from_url(&uri)?;

        if item_type == T::ITEM_TYPE {
            Ok(Self {
                value: uri,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            })
        } else {
            Err(IdError::WrongItemType(item_type).into())
        }
    }
}

impl<'a, T> IdFromBare<'a> for Id<'a, T>
where
    T: ItemTypeId,
{
    fn from_bare<C>(bare: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let bare: Cow<'a, str> = bare.into();

        if verify_valid_id(&bare) {
            Ok(Self {
                value: bare,
                kind: IdKind::Bare,
                phantom: PhantomData,
            })
        } else {
            Err(IdError::InvalidId(bare.to_string()).into())
        }
    }
}

impl<'a> IdFromUri<'a> for PlayableItem<'a> {
    fn from_uri<C>(uri: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let uri: Cow<'a, str> = uri.into();
        let (item_type, id_index) = parse_item_type_and_id_from_uri(&uri)?;

        match item_type {
            ItemType::Track => Ok(Self::Track(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            })),

            ItemType::Episode => Ok(Self::Episode(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            })),

            item_type => Err(IdError::WrongItemType(item_type).into()),
        }
    }
}

impl<'a> IdFromUrl<'a> for PlayableItem<'a> {
    fn from_url<C>(url: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let url: Cow<'a, str> = url.into();
        let (item_type, id_index) = parse_item_type_and_id_from_url(&url)?;

        match item_type {
            ItemType::Track => Ok(Self::Track(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            })),

            ItemType::Episode => Ok(Self::Episode(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            })),

            item_type => Err(IdError::WrongItemType(item_type).into()),
        }
    }
}

impl<'a> IdFromUri<'a> for PlayableContext<'a> {
    fn from_uri<C>(uri: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let uri: Cow<'a, str> = uri.into();
        let (item_type, id_index) = parse_item_type_and_id_from_uri(&uri)?;

        match item_type {
            ItemType::Album => Ok(Self::Album(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            })),

            ItemType::Artist => Ok(Self::Artist(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            })),

            ItemType::Playlist => Ok(Self::Playlist(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            })),

            ItemType::Show => Ok(Self::Show(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            })),

            item_type => Err(IdError::WrongItemType(item_type).into()),
        }
    }
}

impl<'a> IdFromUrl<'a> for PlayableContext<'a> {
    fn from_url<C>(url: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let url: Cow<'a, str> = url.into();
        let (item_type, id_index) = parse_item_type_and_id_from_url(&url)?;

        match item_type {
            ItemType::Album => Ok(Self::Album(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            })),

            ItemType::Artist => Ok(Self::Artist(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            })),

            ItemType::Playlist => Ok(Self::Playlist(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            })),

            ItemType::Show => Ok(Self::Show(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            })),

            item_type => Err(IdError::WrongItemType(item_type).into()),
        }
    }
}

impl<'a> IdFromUri<'a> for SpotifyId<'a> {
    fn from_uri<C>(uri: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let uri: Cow<'a, str> = uri.into();
        let (item_type, id_index) = parse_item_type_and_id_from_uri(&uri)?;

        match item_type {
            ItemType::Track => Ok(Self::Item(PlayableItem::Track(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            }))),

            ItemType::Episode => Ok(Self::Item(PlayableItem::Episode(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            }))),

            ItemType::Album => Ok(Self::Context(PlayableContext::Album(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            }))),

            ItemType::Artist => Ok(Self::Context(PlayableContext::Artist(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            }))),

            ItemType::Playlist => Ok(Self::Context(PlayableContext::Playlist(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            }))),

            ItemType::Show => Ok(Self::Context(PlayableContext::Show(Id {
                value: uri,
                kind: IdKind::Uri(id_index),
                phantom: PhantomData,
            }))),
        }
    }
}

impl<'a> IdFromUrl<'a> for SpotifyId<'a> {
    fn from_url<C>(url: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let url: Cow<'a, str> = url.into();
        let (item_type, id_index) = parse_item_type_and_id_from_uri(&url)?;

        match item_type {
            ItemType::Track => Ok(Self::Item(PlayableItem::Track(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            }))),

            ItemType::Episode => Ok(Self::Item(PlayableItem::Episode(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            }))),

            ItemType::Album => Ok(Self::Context(PlayableContext::Album(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            }))),

            ItemType::Artist => Ok(Self::Context(PlayableContext::Artist(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            }))),

            ItemType::Playlist => Ok(Self::Context(PlayableContext::Playlist(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            }))),

            ItemType::Show => Ok(Self::Context(PlayableContext::Show(Id {
                value: url,
                kind: IdKind::Url(id_index),
                phantom: PhantomData,
            }))),
        }
    }
}

impl<'a, T> IdTrait<'a> for Id<'a, T>
where
    T: ItemTypeId,
{
    fn id(&self) -> &str {
        match self.kind {
            IdKind::Uri(index) => &self.value[index..],
            IdKind::Url(index) => &self.value[index..index + ID_LENGTH],
            IdKind::Bare => &self.value,
        }
    }

    fn uri(&'a self) -> Cow<'a, str> {
        match &self.kind {
            IdKind::Uri(_) => match &self.value {
                Cow::Borrowed(b) => Cow::Borrowed(b),
                Cow::Owned(o) => Cow::Borrowed(o),
            },

            IdKind::Url(index) => Cow::Owned(format!(
                "spotify:{}:{}",
                T::ITEM_TYPE,
                &self.value[*index..*index + ID_LENGTH]
            )),

            IdKind::Bare => Cow::Owned(format!("spotify:{}:{}", T::ITEM_TYPE, self.value)),
        }
    }

    fn url(&'a self) -> Cow<'a, str> {
        match &self.kind {
            IdKind::Url(_) => match &self.value {
                Cow::Borrowed(b) => Cow::Borrowed(b),
                Cow::Owned(o) => Cow::Borrowed(o),
            },

            IdKind::Uri(index) => Cow::Owned(format!(
                "https://open.spotify.com/{}/{}",
                T::ITEM_TYPE,
                &self.value[*index..*index + ID_LENGTH]
            )),

            IdKind::Bare => Cow::Owned(format!("https://open.spotify.com/{}/{}", T::ITEM_TYPE, self.value)),
        }
    }
}

impl<'a> IdTrait<'a> for SpotifyId<'a> {
    fn id(&'a self) -> &'a str {
        match self {
            SpotifyId::Item(item) => item.id(),
            SpotifyId::Context(context) => context.id(),
        }
    }

    fn uri(&'a self) -> Cow<'a, str> {
        match self {
            SpotifyId::Item(item) => item.uri(),
            SpotifyId::Context(context) => context.uri(),
        }
    }

    fn url(&'a self) -> Cow<'a, str> {
        match self {
            SpotifyId::Item(item) => item.url(),
            SpotifyId::Context(context) => context.url(),
        }
    }
}

impl<'a> IdTrait<'a> for PlayableItem<'a> {
    fn id(&self) -> &str {
        match self {
            PlayableItem::Track(track) => track.id(),
            PlayableItem::Episode(episode) => episode.id(),
        }
    }

    fn uri(&'a self) -> Cow<'a, str> {
        match self {
            PlayableItem::Track(track) => track.uri(),
            PlayableItem::Episode(episode) => episode.uri(),
        }
    }

    fn url(&'a self) -> Cow<'a, str> {
        match self {
            PlayableItem::Track(track) => track.url(),
            PlayableItem::Episode(episode) => episode.url(),
        }
    }
}

impl<'a> IdTrait<'a> for PlayableContext<'a> {
    fn id(&self) -> &str {
        match self {
            PlayableContext::Artist(artist) => artist.id(),
            PlayableContext::Album(album) => album.id(),
            PlayableContext::Playlist(playlist) => playlist.id(),
            PlayableContext::Show(show) => show.id(),
        }
    }

    fn uri(&'a self) -> Cow<'a, str> {
        match self {
            PlayableContext::Artist(artist) => artist.uri(),
            PlayableContext::Album(album) => album.uri(),
            PlayableContext::Playlist(playlist) => playlist.uri(),
            PlayableContext::Show(show) => show.uri(),
        }
    }

    fn url(&'a self) -> Cow<'a, str> {
        match self {
            PlayableContext::Artist(artist) => artist.url(),
            PlayableContext::Album(album) => album.url(),
            PlayableContext::Playlist(playlist) => playlist.url(),
            PlayableContext::Show(show) => show.url(),
        }
    }
}

impl<'a> From<PlayableItem<'a>> for SpotifyId<'a> {
    fn from(item: PlayableItem<'a>) -> Self {
        Self::Item(item)
    }
}

impl<'a> From<PlayableContext<'a>> for SpotifyId<'a> {
    fn from(context: PlayableContext<'a>) -> Self {
        Self::Context(context)
    }
}

impl<'a, T> From<Id<'a, T>> for SpotifyId<'a>
where
    T: ItemTypeId,
    PlayableItem<'a>: From<Id<'a, T>>,
    PlayableContext<'a>: From<Id<'a, T>>,
{
    fn from(id: Id<'a, T>) -> Self {
        match T::ITEM_TYPE {
            ItemType::Track | ItemType::Episode => Self::Item(id.into()),
            ItemType::Album | ItemType::Artist | ItemType::Playlist | ItemType::Show => Self::Context(id.into()),
        }
    }
}

impl<'a> From<Id<'a, TrackId>> for PlayableItem<'a> {
    fn from(id: Id<'a, TrackId>) -> Self {
        Self::Track(id)
    }
}

impl<'a> From<Id<'a, EpisodeId>> for PlayableItem<'a> {
    fn from(id: Id<'a, EpisodeId>) -> Self {
        Self::Episode(id)
    }
}

impl<'a> From<Id<'a, AlbumId>> for PlayableContext<'a> {
    fn from(id: Id<'a, AlbumId>) -> Self {
        Self::Album(id)
    }
}

impl<'a> From<Id<'a, ArtistId>> for PlayableContext<'a> {
    fn from(id: Id<'a, ArtistId>) -> Self {
        Self::Artist(id)
    }
}

impl<'a> From<Id<'a, PlaylistId>> for PlayableContext<'a> {
    fn from(id: Id<'a, PlaylistId>) -> Self {
        Self::Playlist(id)
    }
}

impl<'a> From<Id<'a, ShowId>> for PlayableContext<'a> {
    fn from(id: Id<'a, ShowId>) -> Self {
        Self::Show(id)
    }
}

impl<'de, T> Deserialize<'de> for Id<'static, T>
where
    T: ItemTypeId,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdVisitor<'a, T> {
            phantom: PhantomData<&'a T>,
        }

        impl<'de, T> Visitor<'de> for IdVisitor<'de, T>
        where
            T: ItemTypeId + 'static,
        {
            type Value = Id<'static, T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_fmt(format_args!("a Spotify {:?} ID", T::ITEM_TYPE))
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                // TODO: is it actually possible to borrow everything from the source string? the string would have to
                // be kept alive as long as the resulting IDs as well which makes me think it's not possible
                self.visit_string(v.to_owned())
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (kind, value) = if v.starts_with(URI_PREFIX) {
                    let (item_type, id_index) = parse_item_type_and_id_from_uri(&v)
                        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;

                    if item_type == T::ITEM_TYPE {
                        (IdKind::Uri(id_index), Cow::Owned(v))
                    } else {
                        return Err(de::Error::invalid_value(de::Unexpected::Str(&v), &self));
                    }
                } else if v.starts_with(URL_PREFIX) {
                    let (item_type, id_index) = parse_item_type_and_id_from_url(&v)
                        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &"a Spotify URL"))?;

                    if item_type == T::ITEM_TYPE {
                        (IdKind::Url(id_index), Cow::Owned(v))
                    } else {
                        return Err(de::Error::invalid_value(de::Unexpected::Str(&v), &self));
                    }
                } else if verify_valid_id(&v) {
                    (IdKind::Bare, Cow::Owned(v))
                } else {
                    return Err(de::Error::invalid_value(de::Unexpected::Str(&v), &self));
                };

                Ok(Id {
                    value,
                    kind,
                    phantom: PhantomData,
                })
            }
        }

        deserializer.deserialize_string(IdVisitor::<T> { phantom: PhantomData })
    }
}

fn parse_item_type_and_id_from_uri(uri: &str) -> Result<(ItemType, usize)> {
    if let Some((item_type, id)) = uri
        .strip_prefix(URI_PREFIX)
        .and_then(|prefix_removed| prefix_removed.split_once(':'))
    {
        let item_type: ItemType = item_type.parse()?;

        if verify_valid_id(id) {
            // the ID is always at the end of the string
            Ok((item_type, uri.len() - ID_LENGTH))
        } else {
            Err(IdError::InvalidId(id.to_owned()).into())
        }
    } else {
        Err(IdError::MalformedString(uri.to_string()).into())
    }
}

fn parse_item_type_and_id_from_url(url: &str) -> Result<(ItemType, usize)> {
    // a whole URL could look like: https://open.spotify.com/track/3mXLyNsVeLelMakgpGUp1f?si=AAAAAAAAAAAAAAAA
    // TODO: definitely gonna need some unit tests for this shit
    if let Some((item_type_str, id)) = url
        // remove the leading domain
        .strip_prefix(URL_PREFIX)
        // split by / to get "track" and "3mXLyNsVeLelMakgpGUp1f?si=AAAAAAAAAAAAAAAA"
        .and_then(|prefix_removed| prefix_removed.split_once('/'))
        // remove the possible query from the path to get just the ID
        .and_then(|(item_type, id_with_possible_query)| {
            Some(item_type).zip(id_with_possible_query.split_once('?').map(|(id, _)| id))
        })
    {
        let item_type: ItemType = item_type_str.parse()?;

        if verify_valid_id(id) {
            // the position of the ID in the string is the domain + the item type + /
            Ok((item_type, URL_PREFIX.len() + item_type_str.len() + 1))
        } else {
            Err(IdError::InvalidId(id.to_owned()).into())
        }
    } else {
        Err(IdError::MalformedString(url.to_string()).into())
    }
}

fn verify_valid_id(id: &str) -> bool {
    // Spotify IDs are base-62 strings and they look like 3mXLyNsVeLelMakgpGUp1f
    if id.len() != ID_LENGTH {
        return false;
    }

    for c in id.chars() {
        if !c.is_ascii_alphabetic() && !c.is_ascii_digit() {
            return false;
        }
    }

    true
}
