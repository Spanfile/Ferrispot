//! Contains objects that represent all Spotify IDs.
//!
//! # Working with IDs
//!
//! There are three different kinds of Spotify IDs that this module can handle:
//! - URIs: `spotify:track:2pDPOMX0kWA7kcPBcDCQBu`
//! - URLs: `https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu`. The URL may contain any query prameters.
//! - Bare IDs: `2pDPOMX0kWA7kcPBcDCQBu`
//!
//! The IDs are split into two kinds: playable items and playable contexts. Playable items are individual items that can
//! be played; tracks or episodes. Playable contexts are collections of one or more playable items; artists, albums,
//! playlists or shows. The enums [PlayableItem] and [PlayableContext] represent the two kinds. These two kinds are
//! grouped into one common [SpotifyId] that encompasses all of them.
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
//! You may also let the parser figure out if the input is an URL or an URI:
//!
//! ```
//! # use ferrispot::model::id::{Id, TrackId};
//! # use ferrispot::prelude::*;
//! let track_uri_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
//! let track_url_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
//!
//! let track_from_uri = Id::<TrackId>::from_url_or_uri(track_uri_string).unwrap();
//! let track_from_url = Id::<TrackId>::from_url_or_uri(track_url_string).unwrap();
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
//! ### Efficiency
//!
//! [Id] internally stores the originally given string in a [Cow]. This means it will borrow the input string if it is
//! given as an `&str`, which helps avoid string allocations in certain cases. For example, most Spotify API endpoints
//! require an input ID in the URI form, so if the ID is originally parsed from an URI, the entire original string can
//! be used instead of having to allocate a new string. You may also retrieve the ID in an URL form, so if the original
//! string was also an URL, no new strings are allocated.
//!
//! You may convert an [Id] that borrows the original input into a static [Id] that owns its value by using the
//! [`as_static`](IdTrait::as_static)-method. Note that cloning a borrowing [Id] does not turn it into an owning [Id]!
//!
//! ```
//! # use ferrispot::model::id::{Id, TrackId};
//! # use ferrispot::prelude::*;
//! let id_string = String::from("spotify:track:2pDPOMX0kWA7kcPBcDCQBu");
//!
//! // this Id borrows the input string and shares the borrow's lifetime
//! let track_id = Id::<TrackId>::from_url_or_uri(&id_string).unwrap();
//!
//! // dropping the original string is invalid, since it's borrowed in the Id
//! // drop(id_string);
//!
//! // convert the Id into a static Id by cloning the internal borrowed string and drop the borrowind Id
//! let owning_track_id = track_id.as_static();
//! drop(track_id);
//!
//! // dropping the original string is now possible, since it's not borrowed anymore
//! drop(id_string);
//! ```
//!
//! ## `PlayableItem` and `PlayableContext`
//!
//! [PlayableItem] and [PlayableContext] are wrapper enums that encompass an [Id] in their variants. Their benefit is
//! that they simplify ID parsing by allowing the parser figure out the correct type for the [Id] from the input ID
//! string. Their downside is that they do not support parsing from a bare ID, since it is impossible to figure out
//! which kind of ID it is.
//!
//! ```
//! # use ferrispot::model::id::{PlayableContext, PlayableItem};
//! # use ferrispot::prelude::*;
//! let track_uri_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
//! let track_url_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
//! let album_uri_string = "spotify:album:0tDsHtvN9YNuZjlqHvDY2P";
//! let album_url_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P";
//!
//! // both of these are PlayableItem::Track(Id::<TrackId>)
//! let track_from_uri = PlayableItem::from_uri(track_uri_string).unwrap();
//! let track_from_url = PlayableItem::from_url(track_url_string).unwrap();
//!
//! // both of these are PlayableContext::Album(Id::<AlbumId>)
//! let album_from_uri = PlayableContext::from_uri(album_uri_string).unwrap();
//! let album_from_url = PlayableContext::from_url(album_url_string).unwrap();
//! ```
//!
//! Attempting to parse an ID of the wrong type will fail:
//!
//! ```
//! # use ferrispot::model::id::PlayableContext;
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
//! # use ferrispot::model::id::SpotifyId;
//! # use ferrispot::prelude::*;
//! let track_uri_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
//! let album_url_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P";
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

use crate::{
    error::{IdError, Result},
    util::maybe_split_once::MaybeSplitOnce,
};

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
    /// This type that has the `'static` lifetime.
    type StaticSelf: 'static;

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

    /// Returns a new Id that clones the value from this Id and owns it.
    fn as_static(&'a self) -> Self::StaticSelf;
}

/// Trait for parsing any string-looking type that contains a Spotify URL or URI into an ID type.
///
/// See the [module-level docs](self) for information on how to work with IDs.
pub trait IdFromKnownKind<'a>: private::Sealed
where
    Self: Sized,
{
    /// Parses a Spotify URI string into an ID.
    fn from_uri<C>(uri: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>;

    /// Parses a Spotify URL into an ID.
    fn from_url<C>(url: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>;

    /// Parses either a Spotify URL or Spotify URI into an ID.
    fn from_url_or_uri<C>(url_or_uri: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let url_or_uri = url_or_uri.into();

        if url_or_uri.starts_with(URI_PREFIX) {
            Self::from_uri(url_or_uri)
        } else if url_or_uri.starts_with(URL_PREFIX) {
            Self::from_url(url_or_uri)
        } else {
            Err(IdError::MalformedString(url_or_uri.to_string()).into())
        }
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl<'a, T> IdFromKnownKind<'a> for Id<'a, T>
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

impl<'a> IdFromKnownKind<'a> for PlayableItem<'a> {
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

impl<'a> IdFromKnownKind<'a> for PlayableContext<'a> {
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

impl<'a> IdFromKnownKind<'a> for SpotifyId<'a> {
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

    fn from_url<C>(url: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>,
    {
        let url: Cow<'a, str> = url.into();
        let (item_type, id_index) = parse_item_type_and_id_from_url(&url)?;

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
    T: ItemTypeId + 'static,
{
    type StaticSelf = Id<'static, T>;

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

    fn as_static(&'a self) -> Self::StaticSelf {
        Id {
            value: Cow::Owned(self.value.clone().into_owned()),
            kind: self.kind,
            phantom: PhantomData,
        }
    }
}

impl<'a> IdTrait<'a> for SpotifyId<'a> {
    type StaticSelf = SpotifyId<'static>;

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

    fn as_static(&'a self) -> Self::StaticSelf {
        match self {
            SpotifyId::Item(item) => SpotifyId::Item(item.as_static()),
            SpotifyId::Context(context) => SpotifyId::Context(context.as_static()),
        }
    }
}

impl<'a> IdTrait<'a> for PlayableItem<'a> {
    type StaticSelf = PlayableItem<'static>;

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

    fn as_static(&'a self) -> Self::StaticSelf {
        match self {
            PlayableItem::Track(track) => PlayableItem::Track(track.as_static()),
            PlayableItem::Episode(episode) => PlayableItem::Episode(episode.as_static()),
        }
    }
}

impl<'a> IdTrait<'a> for PlayableContext<'a> {
    type StaticSelf = PlayableContext<'static>;

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

    fn as_static(&'a self) -> Self::StaticSelf {
        match self {
            PlayableContext::Artist(artist) => PlayableContext::Artist(artist.as_static()),
            PlayableContext::Album(album) => PlayableContext::Album(album.as_static()),
            PlayableContext::Playlist(playlist) => PlayableContext::Playlist(playlist.as_static()),
            PlayableContext::Show(show) => PlayableContext::Show(show.as_static()),
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

impl<'de> Deserialize<'de> for SpotifyId<'static> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
            type Value = SpotifyId<'static>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Spotify URI or a Spotify URL (bare IDs cannot be deserialized into SpotifyIds)")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_string(v.to_owned())
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (item_type, kind) = if v.starts_with(URI_PREFIX) {
                    let (item_type, id_index) = parse_item_type_and_id_from_uri(&v)
                        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;
                    (item_type, IdKind::Uri(id_index))
                } else if v.starts_with(URL_PREFIX) {
                    let (item_type, id_index) = parse_item_type_and_id_from_url(&v)
                        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &"a Spotify URL"))?;
                    (item_type, IdKind::Url(id_index))
                } else {
                    return Err(de::Error::invalid_value(de::Unexpected::Str(&v), &self));
                };

                // TODO: this would really benefit from a refactor wrt. creating the Ids, but how? they're all actually
                // different types even though they look the same
                match item_type {
                    ItemType::Track => Ok(SpotifyId::Item(PlayableItem::Track(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    }))),

                    ItemType::Episode => Ok(SpotifyId::Item(PlayableItem::Episode(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    }))),

                    ItemType::Album => Ok(SpotifyId::Context(PlayableContext::Album(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    }))),

                    ItemType::Artist => Ok(SpotifyId::Context(PlayableContext::Artist(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    }))),

                    ItemType::Playlist => Ok(SpotifyId::Context(PlayableContext::Playlist(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    }))),

                    ItemType::Show => Ok(SpotifyId::Context(PlayableContext::Show(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    }))),
                }
            }
        }

        deserializer.deserialize_string(IdVisitor)
    }
}

impl<'de> Deserialize<'de> for PlayableItem<'static> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
            type Value = PlayableItem<'static>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter
                    .write_str("a Spotify URI or a Spotify URL (bare IDs cannot be deserialized into PlayableItems)")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_string(v.to_owned())
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (item_type, kind) = if v.starts_with(URI_PREFIX) {
                    let (item_type, id_index) = parse_item_type_and_id_from_uri(&v)
                        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;
                    (item_type, IdKind::Uri(id_index))
                } else if v.starts_with(URL_PREFIX) {
                    let (item_type, id_index) = parse_item_type_and_id_from_url(&v)
                        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;
                    (item_type, IdKind::Url(id_index))
                } else {
                    return Err(de::Error::invalid_value(de::Unexpected::Str(&v), &self));
                };

                // TODO: this would really benefit from a refactor wrt. creating the Ids, but how? they're all actually
                // different types even though they look the same
                match item_type {
                    ItemType::Track => Ok(PlayableItem::Track(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    })),

                    ItemType::Episode => Ok(PlayableItem::Episode(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    })),

                    _ => Err(de::Error::invalid_type(de::Unexpected::Str(&v), &self)),
                }
            }
        }

        deserializer.deserialize_string(IdVisitor)
    }
}

impl<'de> Deserialize<'de> for PlayableContext<'static> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdVisitor;

        impl<'de> Visitor<'de> for IdVisitor {
            type Value = PlayableContext<'static>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter
                    .write_str("a Spotify URI or a Spotify URL (bare IDs cannot be deserialized into PlayableItems)")
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_string(v.to_owned())
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (item_type, kind) = if v.starts_with(URI_PREFIX) {
                    let (item_type, id_index) = parse_item_type_and_id_from_uri(&v)
                        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;
                    (item_type, IdKind::Uri(id_index))
                } else if v.starts_with(URL_PREFIX) {
                    let (item_type, id_index) = parse_item_type_and_id_from_url(&v)
                        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;
                    (item_type, IdKind::Url(id_index))
                } else {
                    return Err(de::Error::invalid_value(de::Unexpected::Str(&v), &self));
                };

                // TODO: this would really benefit from a refactor wrt. creating the Ids, but how? they're all actually
                // different types even though they look the same
                match item_type {
                    ItemType::Album => Ok(PlayableContext::Album(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    })),

                    ItemType::Artist => Ok(PlayableContext::Artist(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    })),

                    ItemType::Playlist => Ok(PlayableContext::Playlist(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    })),

                    ItemType::Show => Ok(PlayableContext::Show(Id {
                        value: Cow::Owned(v),
                        kind,
                        phantom: PhantomData,
                    })),

                    _ => Err(de::Error::invalid_type(de::Unexpected::Str(&v), &self)),
                }
            }
        }

        deserializer.deserialize_string(IdVisitor)
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
                        .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;

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

    if let Some((item_type_str, id)) = url
        // remove the leading domain
        .strip_prefix(URL_PREFIX)
        // split by / to get "track" and "3mXLyNsVeLelMakgpGUp1f?si=AAAAAAAAAAAAAAAA"
        .and_then(|prefix_removed| prefix_removed.split_once('/'))
        // remove the possible query from the path to get just the ID
        .map(|(item_type_str, id_with_possible_query)| {
            let (left, _) = id_with_possible_query.maybe_split_once('?');
            (item_type_str, left)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn track_id_from_uri() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn track_id_from_url() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_url(id_string).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn track_id_from_url_with_query() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu?si=AAAAAAAAAA";
        let id = Id::<TrackId>::from_url(id_string).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn track_id_from_bare() {
        let id_string = "2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_bare(id_string).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn album_id_from_uri() {
        let id_string = "spotify:album:0tDsHtvN9YNuZjlqHvDY2P";
        let id = Id::<AlbumId>::from_uri(id_string).unwrap();

        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn album_id_from_url() {
        let id_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P";
        let id = Id::<AlbumId>::from_url(id_string).unwrap();

        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn album_id_from_bare() {
        let id_string = "0tDsHtvN9YNuZjlqHvDY2P";
        let id = Id::<AlbumId>::from_bare(id_string).unwrap();

        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn artist_id_from_uri() {
        let id_string = "spotify:artist:6pNgnvzBa6Bthsv8SrZJYl";
        let id = Id::<ArtistId>::from_uri(id_string).unwrap();

        assert_eq!(id.id(), "6pNgnvzBa6Bthsv8SrZJYl");
    }

    #[test]
    fn artist_id_from_url() {
        let id_string = "https://open.spotify.com/artist/6pNgnvzBa6Bthsv8SrZJYl";
        let id = Id::<ArtistId>::from_url(id_string).unwrap();

        assert_eq!(id.id(), "6pNgnvzBa6Bthsv8SrZJYl");
    }

    #[test]
    fn artist_id_from_bare() {
        let id_string = "6pNgnvzBa6Bthsv8SrZJYl";
        let id = Id::<ArtistId>::from_bare(id_string).unwrap();

        assert_eq!(id.id(), "6pNgnvzBa6Bthsv8SrZJYl");
    }

    #[test]
    fn playlist_id_from_uri() {
        let id_string = "spotify:playlist:37i9dQZF1DWZipvLjDtZYe";
        let id = Id::<PlaylistId>::from_uri(id_string).unwrap();

        assert_eq!(id.id(), "37i9dQZF1DWZipvLjDtZYe");
    }

    #[test]
    fn playlist_id_from_url() {
        let id_string = "https://open.spotify.com/playlist/37i9dQZF1DWZipvLjDtZYe";
        let id = Id::<PlaylistId>::from_url(id_string).unwrap();

        assert_eq!(id.id(), "37i9dQZF1DWZipvLjDtZYe");
    }

    #[test]
    fn playlist_id_from_bare() {
        let id_string = "37i9dQZF1DWZipvLjDtZYe";
        let id = Id::<PlaylistId>::from_bare(id_string).unwrap();

        assert_eq!(id.id(), "37i9dQZF1DWZipvLjDtZYe");
    }

    #[test]
    fn playable_item_id_from_track_uri() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = PlayableItem::from_uri(id_string).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn playable_item_id_from_track_url() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = PlayableItem::from_url(id_string).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn playable_item_id_from_track_url_with_query() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu?si=AAAAAAAAAA";
        let id = PlayableItem::from_url(id_string).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn playable_context_id_from_album_uri() {
        let id_string = "spotify:album:0tDsHtvN9YNuZjlqHvDY2P";
        let id = PlayableContext::from_uri(id_string).unwrap();

        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn playable_context_id_from_album_url() {
        let id_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P";
        let id = PlayableContext::from_url(id_string).unwrap();

        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn playable_context_id_from_album_url_with_query() {
        let id_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P?si=AAAAAAAAAA";
        let id = PlayableContext::from_url(id_string).unwrap();

        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn spotify_id_from_track_uri() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = SpotifyId::from_uri(id_string).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn spotify_id_from_track_url() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = SpotifyId::from_url(id_string).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn spotify_id_from_track_url_with_query() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu?si=AAAAAAAAAA";
        let id = SpotifyId::from_url(id_string).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn spotify_id_from_album_uri() {
        let id_string = "spotify:album:0tDsHtvN9YNuZjlqHvDY2P";
        let id = SpotifyId::from_uri(id_string).unwrap();

        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn spotify_id_from_album_url() {
        let id_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P";
        let id = SpotifyId::from_url(id_string).unwrap();

        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn spotify_id_from_track_uri_and_url() {
        let url_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let uri_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";

        let url_id = SpotifyId::from_url_or_uri(url_string).unwrap();
        let uri_id = SpotifyId::from_url_or_uri(uri_string).unwrap();

        assert_eq!(url_id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
        assert_eq!(uri_id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn wrong_url_prefix() {
        let id_string = "https://google.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_url(id_string);

        assert!(matches!(id, Err(Error::InvalidSpotifyId(IdError::MalformedString(_)))))
    }

    #[test]
    fn wrong_uri_prefix() {
        let id_string = "wrong:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string);

        assert!(matches!(id, Err(Error::InvalidSpotifyId(IdError::MalformedString(_)))))
    }

    #[test]
    fn wrong_id_type_in_url() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<ArtistId>::from_url(id_string);

        assert!(matches!(
            id,
            Err(Error::InvalidSpotifyId(IdError::WrongItemType(ItemType::Track)))
        ))
    }

    #[test]
    fn wrong_id_type_in_uri() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<ArtistId>::from_uri(id_string);

        assert!(matches!(
            id,
            Err(Error::InvalidSpotifyId(IdError::WrongItemType(ItemType::Track)))
        ))
    }

    #[test]
    fn unknown_id_type_in_uri() {
        let id_string = "spotify:wrong:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string);

        assert!(matches!(id, Err(Error::InvalidSpotifyId(IdError::InvalidItemType(_)))))
    }

    #[test]
    fn unknown_id_type_in_url() {
        let id_string = "https://open.spotify.com/wrong/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_url(id_string);

        assert!(matches!(id, Err(Error::InvalidSpotifyId(IdError::InvalidItemType(_)))))
    }

    #[test]
    fn invalid_id_too_short() {
        let id_string = "_";
        let id = Id::<TrackId>::from_bare(id_string);

        assert!(matches!(id, Err(Error::InvalidSpotifyId(IdError::InvalidId(_)))))
    }

    #[test]
    fn invalid_id_too_long() {
        let id_string = "2pDPOMX0kWA7kcPBcDCQBu_";
        let id = Id::<TrackId>::from_bare(id_string);

        assert!(matches!(id, Err(Error::InvalidSpotifyId(IdError::InvalidId(_)))))
    }

    #[test]
    fn invalid_id_illegal_characters() {
        let id_string = "2pDPOMX0kWA7kcPBcDCQB_";
        let id = Id::<TrackId>::from_bare(id_string);

        assert!(matches!(id, Err(Error::InvalidSpotifyId(IdError::InvalidId(_)))))
    }

    #[test]
    fn invalid_id_in_url() {
        let id_string = "https://open.spotify.com/track/_";
        let id = Id::<TrackId>::from_url(id_string);

        assert!(matches!(id, Err(Error::InvalidSpotifyId(IdError::InvalidId(_)))))
    }

    #[test]
    fn invalid_id_in_uri() {
        let id_string = "spotify:track:_";
        let id = Id::<TrackId>::from_uri(id_string);

        assert!(matches!(id, Err(Error::InvalidSpotifyId(IdError::InvalidId(_)))))
    }

    #[test]
    fn uri_from_uri_borrows() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string).unwrap();

        let uri = id.uri();
        assert!(matches!(uri, Cow::Borrowed("spotify:track:2pDPOMX0kWA7kcPBcDCQBu")));
    }

    #[test]
    fn url_from_url_borrows() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_url(id_string).unwrap();

        let url = id.url();
        assert!(matches!(
            url,
            Cow::Borrowed("https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu")
        ));
    }

    #[test]
    fn uri_from_url_allocates() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_url(id_string).unwrap();

        let uri = id.uri();
        assert!(matches!(uri, Cow::Owned(_)));
    }

    #[test]
    fn url_from_uri_allocates() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string).unwrap();

        let url = id.url();
        assert!(matches!(url, Cow::Owned(_)));
    }

    #[test]
    fn uri_from_bare_allocates() {
        let id_string = "2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_bare(id_string).unwrap();

        let uri = id.uri();
        assert!(matches!(uri, Cow::Owned(_)));
    }

    #[test]
    fn url_from_bare_allocates() {
        let id_string = "2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_bare(id_string).unwrap();

        let url = id.url();
        assert!(matches!(url, Cow::Owned(_)));
    }

    #[test]
    fn deserialize_id_from_uri() {
        let json = "\"spotify:track:2pDPOMX0kWA7kcPBcDCQBu\"";
        let id: Id<TrackId> = serde_json::from_str(json).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn deserialize_id_from_url() {
        let json = "\"https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu\"";
        let id: Id<TrackId> = serde_json::from_str(json).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn deserialize_id_from_bare() {
        let json = "\"2pDPOMX0kWA7kcPBcDCQBu\"";
        let id: Id<TrackId> = serde_json::from_str(json).unwrap();

        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn deserialize_playable_item_from_uri() {
        let json = "\"spotify:track:2pDPOMX0kWA7kcPBcDCQBu\"";
        let id: PlayableItem = serde_json::from_str(json).unwrap();

        assert!(matches!(id, PlayableItem::Track(_)));
        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn deserialize_playable_item_from_url() {
        let json = "\"https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu\"";
        let id: PlayableItem = serde_json::from_str(json).unwrap();

        assert!(matches!(id, PlayableItem::Track(_)));
        assert_eq!(id.id(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn cannot_deserialize_playable_item_from_bare() {
        let json = "\"2pDPOMX0kWA7kcPBcDCQBu\"";
        let result: std::result::Result<PlayableItem, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }

    #[test]
    fn deserialize_playable_context_from_uri() {
        let json = "\"spotify:album:0tDsHtvN9YNuZjlqHvDY2P\"";
        let id: PlayableContext = serde_json::from_str(json).unwrap();

        assert!(matches!(id, PlayableContext::Album(_)));
        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn deserialize_playable_context_from_url() {
        let json = "\"https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P\"";
        let id: PlayableContext = serde_json::from_str(json).unwrap();

        assert!(matches!(id, PlayableContext::Album(_)));
        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn cannot_deserialize_playable_context_from_bare() {
        let json = "\"0tDsHtvN9YNuZjlqHvDY2P\"";
        let result: std::result::Result<PlayableContext, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }

    #[test]
    fn deserialize_spotify_id_from_uri() {
        let json = "\"spotify:album:0tDsHtvN9YNuZjlqHvDY2P\"";
        let id: SpotifyId = serde_json::from_str(json).unwrap();

        assert!(matches!(id, SpotifyId::Context(PlayableContext::Album(_))));
        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn deserialize_spotify_id_from_url() {
        let json = "\"https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P\"";
        let id: SpotifyId = serde_json::from_str(json).unwrap();

        assert!(matches!(id, SpotifyId::Context(PlayableContext::Album(_))));
        assert_eq!(id.id(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn cannot_deserialize_spotify_id_from_bare() {
        let json = "\"0tDsHtvN9YNuZjlqHvDY2P\"";
        let result: std::result::Result<SpotifyId, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }
}
