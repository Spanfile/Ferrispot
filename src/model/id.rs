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
//! # The core ID type
//!
//! The core [Id] is best thought of as a transparent type-safe wrapper for Spotify IDs. The struct contains a single
//! Spotify ID of any kind. The struct is generic over the kind of ID it contains using the various type structs that
//! implement the [ItemTypeId]-trait, such as [TrackId] or [AlbumId].
//!
//! You may parse any ID string into an [Id] by specifying the kind in the [Id]'s type parameter:
//!
//! ```
//! # use ferrispot::model::id::{AlbumId, Id, TrackId};
//! # use ferrispot::prelude::*;
//! let track_id = Id::<TrackId>::from_uri("spotify:track:2pDPOMX0kWA7kcPBcDCQBu").unwrap();
//!
//! let album_id =
//!     Id::<AlbumId>::from_url("https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P").unwrap();
//! ```
//!
//! Attempting to parse an ID of the wrong type will fail:
//!
//! ```
//! # use ferrispot::model::id::{Id, AlbumId};
//! # use ferrispot::prelude::*;
//! // the URI is for a track, but we're attempting to parse an album ID
//! assert!(Id::<AlbumId>::from_uri("spotify:track:2pDPOMX0kWA7kcPBcDCQBu").is_err());
//! ```
//!
//! This type is the only ID type that supports parsing from a bare ID, since you have to specify the ID's kind
//! yourself. It is possible to let the parser figure out the ID's kind from the input by using either [PlayableItem and
//! PlayableContext](self#playableitem-and-playablecontext) or [SpotifyId](self#spotifyid).
//!
//! ```
//! # use ferrispot::model::id::{AlbumId, Id, TrackId};
//! # use ferrispot::prelude::*;
//! // the given strings are validated only to *look* like valid Spotify IDs. there are no
//! // guarantees they actually exist within Spotify's catalog
//! let track_id_from_bare = Id::<TrackId>::from_bare("2pDPOMX0kWA7kcPBcDCQBu").unwrap();
//! let album_id_from_bare = Id::<AlbumId>::from_bare("aaaaaaaaaaaaaaaaaaaaaa").unwrap();
//! ```
//!
//! You may also let the parser figure out if the input is an URL or an URI:
//!
//! ```
//! # use ferrispot::model::id::{Id, TrackId};
//! # use ferrispot::prelude::*;
//! let track_from_uri =
//!     Id::<TrackId>::from_url_or_uri("spotify:track:2pDPOMX0kWA7kcPBcDCQBu").unwrap();
//!
//! let track_from_url =
//!     Id::<TrackId>::from_url_or_uri("https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu")
//!         .unwrap();
//! ```
//!
//! ## Efficiency
//!
//! [Id] internally stores the originally given string in a [Cow]. This means it will borrow the input string if it is
//! given as an `&str`, which helps avoid string allocations in certain cases. For example, most Spotify API endpoints
//! require an input ID in the URI form, so if the ID is originally parsed from an URI, the entire original string can
//! be used instead of having to allocate a new string. You may also retrieve the ID in an URL form, so if the original
//! string was also an URL, no new strings are allocated.
//!
//! ```
//! # use ferrispot::model::id::{Id, TrackId};
//! # use ferrispot::prelude::*;
//! # use std::borrow::Cow;
//! let id_from_uri =
//!     Id::<TrackId>::from_uri(String::from("spotify:track:2pDPOMX0kWA7kcPBcDCQBu")).unwrap();
//!
//! // this will borrow the string value from the Id
//! let uri: Cow<_> = id_from_uri.as_uri();
//! assert!(matches!(uri, Cow::Borrowed(_)));
//!
//! // this will allocate a new string formatted as an URL:
//! // https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu
//! let url: Cow<_> = id_from_uri.as_url();
//! assert!(matches!(url, Cow::Owned(_)));
//!
//! // retrieving the bare ID string never allocates, since it can be sliced from any kind of
//! // ID string
//! let bare: &str = id_from_uri.as_str();
//! assert_eq!(bare, "2pDPOMX0kWA7kcPBcDCQBu");
//! ```
//!
//! ### Id references
//!
//! A reference to an Id (`&Id<'_, T>`) can be tedious to work with due to the extra lifetime requirements for the
//! borrow. To aid in this, you may extract a new Id from a given Id with the
//! [`as_borrowed`-function](IdTrait::as_borrowed). The new Id will borrow from the given Id's underlying value, thus it
//! may not outlive the original Id. Therefore, the new Id acts as if it was a reference to the original Id without
//! being a borrowed value (`&Id<'_, T>`).
//! ```
//! # use ferrispot::model::id::{Id, TrackId};
//! # use ferrispot::prelude::*;
//! let owning_track_id =
//!     Id::<TrackId>::from_uri(String::from("spotify:track:2pDPOMX0kWA7kcPBcDCQBu")).unwrap();
//!
//! // instead of having to borrow the entire Id type...
//! let borrowed_id: &Id<'_, _> = &owning_track_id;
//!
//! // ... it is more convenient to use a new Id that borrows the given Id's value
//! let borrowed_id: Id<'_, _> = owning_track_id.as_borrowed();
//! ```
//!
//! ### Owned `Id`s
//!
//! You may convert an [Id] that borrows the original input into a static [Id] that owns its value by using the
//! [`as_owned`-function](IdTrait::as_owned).
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
//! // convert the Id into a static Id by cloning the internal borrowed string and drop the borrowing Id
//! let owning_track_id = track_id.as_owned();
//! drop(track_id);
//!
//! // dropping the original string is now possible, since it's not borrowed anymore
//! drop(id_string);
//! ```
//!
//! ### Clone semantics
//!
//! The [`as_owned`](IdTrait::as_owned)- and [`as_borrowed`](IdTrait::as_borrowed)-functions may seem very similar to
//! the `clone`-function natively provided by the language. However, there are certain important differences that
//! warrant the two special functions.
//!
//! #### Borrowing from an owning Id
//!
//! Using [`as_borrowed`](IdTrait::as_borrowed) on an owning Id returns a new Id that borrows from the given Id's value.
//! The new Id's lifetime may not outlive that of the given Id's.
//! ```compile_fail
//! # use ferrispot::model::id::{Id, TrackId};
//! # use ferrispot::prelude::*;
//! let id_string = String::from("spotify:track:2pDPOMX0kWA7kcPBcDCQBu");
//! let original_track_id = Id::<TrackId>::from_uri(id_string).unwrap();
//!
//! // this new Id borrows from the string the given Id owns, but acts as if it borrows from the
//! // Id, much like a reference would
//! let borrowing_id: Id<TrackId> = original_track_id.as_borrowed();
//!
//! // the new Ids lifetime may not outlive that of the original Id's. this will not compile, since
//! // the borrowing Id is used later
//! drop(original_track_id);
//!
//! let id_str = borrowing_id.as_str();
//! ```
//!
//! #### Borrowing from a borrowing Id
//!
//! Using [`as_borrowed`](IdTrait::as_borrowed) on a borrowing Id returns a new Id that borrows from the given Id's
//! value. The new Id's lifetime may not outlive that of the given Id's.
//! ```compile_fail
//! # use ferrispot::model::id::{Id, TrackId};
//! # use ferrispot::prelude::*;
//! let original_track_id =
//!     Id::<TrackId>::from_uri("spotify:track:2pDPOMX0kWA7kcPBcDCQBu").unwrap();
//!
//! // this new Id borrows from the same string the given Id borrows from, but acts as if it borrows
//! // from the Id, much like a reference would
//! let borrowing_id: Id<TrackId> = original_track_id.as_borrowed();
//!
//! // the new Ids lifetime may not outlive that of the original Id's. this will not compile, since
//! // the borrowing Id is used later
//! drop(original_track_id);
//!
//! let id_str = borrowing_id.as_str();
//! ```
//!
//! Cloning a borrowing Id returns a new Id that borrow's from the given Id's value. The cloned Id's lifetime may not
//! outlive that of the given Id's *value's* lifetime it borrows from, therefore the cloned Id *may* outlive the given
//! Id.
//! ```
//! # use ferrispot::model::id::{Id, TrackId};
//! # use ferrispot::prelude::*;
//! let original_track_id =
//!     Id::<TrackId>::from_uri("spotify:track:2pDPOMX0kWA7kcPBcDCQBu").unwrap();
//!
//! // this new Id borrows from the same string the given Id borrows from, but otherwise is an
//! // entirely new Id
//! let cloned_id: Id<TrackId> = original_track_id.clone();
//!
//! // the cloned Id's lifetime may outlive that of the original Id's
//! drop(original_track_id);
//!
//! let id_str = cloned_id.as_str();
//! ```
//!
//! #### Cloning and owning an Id
//!
//! Cloning or using [`as_owned`](IdTrait::as_owned) on an owning Id, or using [`as_owned`](IdTrait::as_owned) on a
//! borrowing Id, returns a new Id that clones the given Id's value and owns it.
//!
//! # `PlayableItem` and `PlayableContext`
//!
//! [PlayableItem] and [PlayableContext] are wrapper enums that encompass an [Id] in their variants. Their benefit is
//! that they simplify ID parsing by allowing the parser figure out the correct type for the [Id] from the input ID
//! string. Their downside is that they do not support parsing from a bare ID, since it is impossible to figure out
//! which kind of ID it is.
//!
//! Playable items are individual items; tracks or episodes. Playable contexts are collections of playable items;
//! albums, artists, playlists or shows.
//!
//! ```
//! # use ferrispot::model::id::{PlayableContext, PlayableItem};
//! # use ferrispot::prelude::*;
//! // both of these are PlayableItem::Track(Id::<TrackId>)
//! let track_from_uri = PlayableItem::from_uri("spotify:track:2pDPOMX0kWA7kcPBcDCQBu").unwrap();
//! let track_from_url =
//!     PlayableItem::from_url("https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu").unwrap();
//!
//! // both of these are PlayableContext::Album(Id::<AlbumId>)
//! let album_from_uri = PlayableContext::from_uri("spotify:album:0tDsHtvN9YNuZjlqHvDY2P").unwrap();
//! let album_from_url =
//!     PlayableContext::from_url("https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P").unwrap();
//! ```
//!
//! Attempting to parse an ID of the wrong type will fail:
//! ```
//! # use ferrispot::model::id::PlayableContext;
//! # use ferrispot::prelude::*;
//! // PlayableContext expects an album, an artist, a playlist or a show, but we're using a track ID
//! assert!(PlayableContext::from_uri("spotify:track:2pDPOMX0kWA7kcPBcDCQBu").is_err());
//! ```
//!
//! # `SpotifyId`
//!
//! [SpotifyId] encompasses both [PlayableItem] and [PlayableContext] into one type which lets you parse any kind of ID
//! into a single type. Like with [PlayableItem] and [PlayableContext], bare IDs are not supported.
//!
//! ```
//! # use ferrispot::model::id::SpotifyId;
//! # use ferrispot::prelude::*;
//! // SpotifyId::Item(PlayableItem::Track(Id::<TrackId>))
//! let track_from_uri = SpotifyId::from_uri("spotify:track:2pDPOMX0kWA7kcPBcDCQBu").unwrap();
//!
//! // SpotifyId::Context(PlayableContext::Album(Id::<AlbumId>))
//! let album_from_url =
//!     SpotifyId::from_url("https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P").unwrap();
//!
//! // SpotifyId::Context(PlayableContext::Artist(Id::<ArtistId>))
//! let artist_from_url =
//!     SpotifyId::from_url("https://open.spotify.com/artist/6pNgnvzBa6Bthsv8SrZJYl").unwrap();
//! ```

use std::{borrow::Cow, fmt, marker::PhantomData};

use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};

use super::ItemType;
use crate::{error::IdError, util::maybe_split_once::MaybeSplitOnce};

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
    /// This type that has the `'static` lifetime, which essentially means it owns its ID value.
    type Owned: 'static;

    /// This type that has a certain lifetime `'b` that is guaranteed to not outlive this type's lifetime `'a`.
    type Borrowed<'b>
    where
        'a: 'b,
        Self: 'a;

    /// Returns this ID as a bare Spotify ID.
    fn as_str(&'a self) -> &'a str;

    /// Returns this ID as a Spotify URI.
    ///
    /// This function returns a [Cow], since it allows the function to avoid needlessly allocating a new string if the
    /// original ID string this ID was constructed from is already an URI. Otherwise, it will allocate a new URI string.
    fn as_uri(&'a self) -> Cow<'a, str>;

    /// Returns this ID as a Spotify URL.
    ///
    /// This function returns a [Cow], since it allows the function to avoid needlessly allocating a new string if the
    /// original ID string this ID was constructed from is already an URL. Otherwise, it will allocate a new URL string.
    fn as_url(&'a self) -> Cow<'a, str>;

    /// Returns a new Id that clones the value from this Id and owns it.
    fn as_owned(&'a self) -> Self::Owned;

    /// Returns a new Id that borrows from this Id.
    ///
    /// This function is primarily used to avoid double references. A value of type `&Id<'_, T>` can be tedious to work
    /// with, so this function can be used to return a new Id that borrows from the Id the reference would point to.
    fn as_borrowed<'b>(&'a self) -> Self::Borrowed<'b>
    where
        'a: 'b;
}

/// Trait for parsing any string-looking type that contains a Spotify URL or URI into an ID type.
///
/// See the [module-level docs](self) for information on how to work with IDs.
pub trait IdFromKnownKind<'a>: private::Sealed
where
    Self: Sized,
{
    /// Parses a Spotify URI string into an ID.
    fn from_uri<C>(uri: C) -> Result<Self, IdError>
    where
        C: Into<Cow<'a, str>>;

    /// Parses a Spotify URL into an ID.
    fn from_url<C>(url: C) -> Result<Self, IdError>
    where
        C: Into<Cow<'a, str>>;

    /// Parses either a Spotify URL or Spotify URI into an ID.
    fn from_url_or_uri<C>(url_or_uri: C) -> Result<Self, IdError>
    where
        C: Into<Cow<'a, str>>,
    {
        let url_or_uri = url_or_uri.into();

        if url_or_uri.starts_with(URI_PREFIX) {
            Self::from_uri(url_or_uri)
        } else if url_or_uri.starts_with(URL_PREFIX) {
            Self::from_url(url_or_uri)
        } else {
            Err(IdError::MalformedString(url_or_uri.to_string()))
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
    fn from_bare<C>(bare: C) -> Result<Self, IdError>
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
    phantom: PhantomData<T>,
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

impl<T> private::Sealed for Id<'_, T> where T: ItemTypeId {}
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

impl<'a, T> Id<'a, T>
where
    T: ItemTypeId,
{
    /// When calling this function, be absolutely sure the value matches the ID kind.
    fn new(value: Cow<'a, str>, kind: IdKind) -> Self {
        Self {
            value,
            kind,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> IdFromKnownKind<'a> for Id<'a, T>
where
    T: ItemTypeId,
{
    fn from_uri<C>(uri: C) -> Result<Self, IdError>
    where
        C: Into<Cow<'a, str>>,
    {
        let uri: Cow<'a, str> = uri.into();
        let (item_type, id_index) = parse_item_type_and_id_from_uri(&uri)?;

        if item_type == T::ITEM_TYPE {
            Ok(Id::new(uri, IdKind::Uri(id_index)))
        } else {
            Err(IdError::WrongItemType(item_type))
        }
    }

    fn from_url<C>(url: C) -> Result<Self, IdError>
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
            Err(IdError::WrongItemType(item_type))
        }
    }
}

impl<'a, T> IdFromBare<'a> for Id<'a, T>
where
    T: ItemTypeId,
{
    fn from_bare<C>(bare: C) -> Result<Self, IdError>
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
            Err(IdError::InvalidId(bare.to_string()))
        }
    }
}

impl<'a> IdFromKnownKind<'a> for PlayableItem<'a> {
    fn from_uri<C>(uri: C) -> Result<Self, IdError>
    where
        C: Into<Cow<'a, str>>,
    {
        let uri: Cow<'a, str> = uri.into();
        let (item_type, id_index) = parse_item_type_and_id_from_uri(&uri)?;

        match item_type {
            ItemType::Track => Ok(Self::Track(Id::new(uri, IdKind::Uri(id_index)))),
            ItemType::Episode => Ok(Self::Episode(Id::new(uri, IdKind::Uri(id_index)))),

            item_type => Err(IdError::WrongItemType(item_type)),
        }
    }

    fn from_url<C>(url: C) -> Result<Self, IdError>
    where
        C: Into<Cow<'a, str>>,
    {
        let url: Cow<'a, str> = url.into();
        let (item_type, id_index) = parse_item_type_and_id_from_url(&url)?;

        match item_type {
            ItemType::Track => Ok(Self::Track(Id::new(url, IdKind::Url(id_index)))),
            ItemType::Episode => Ok(Self::Episode(Id::new(url, IdKind::Url(id_index)))),

            item_type => Err(IdError::WrongItemType(item_type)),
        }
    }
}

impl<'a> IdFromKnownKind<'a> for PlayableContext<'a> {
    fn from_uri<C>(uri: C) -> Result<Self, IdError>
    where
        C: Into<Cow<'a, str>>,
    {
        let uri: Cow<'a, str> = uri.into();
        let (item_type, id_index) = parse_item_type_and_id_from_uri(&uri)?;

        match item_type {
            ItemType::Album => Ok(Self::Album(Id::new(uri, IdKind::Uri(id_index)))),
            ItemType::Artist => Ok(Self::Artist(Id::new(uri, IdKind::Uri(id_index)))),
            ItemType::Playlist => Ok(Self::Playlist(Id::new(uri, IdKind::Uri(id_index)))),
            ItemType::Show => Ok(Self::Show(Id::new(uri, IdKind::Uri(id_index)))),

            item_type => Err(IdError::WrongItemType(item_type)),
        }
    }

    fn from_url<C>(url: C) -> Result<Self, IdError>
    where
        C: Into<Cow<'a, str>>,
    {
        let url: Cow<'a, str> = url.into();
        let (item_type, id_index) = parse_item_type_and_id_from_url(&url)?;

        match item_type {
            ItemType::Album => Ok(Self::Album(Id::new(url, IdKind::Url(id_index)))),
            ItemType::Artist => Ok(Self::Artist(Id::new(url, IdKind::Url(id_index)))),
            ItemType::Playlist => Ok(Self::Playlist(Id::new(url, IdKind::Url(id_index)))),
            ItemType::Show => Ok(Self::Show(Id::new(url, IdKind::Url(id_index)))),

            item_type => Err(IdError::WrongItemType(item_type)),
        }
    }
}

impl<'a> IdFromKnownKind<'a> for SpotifyId<'a> {
    fn from_uri<C>(uri: C) -> Result<Self, IdError>
    where
        C: Into<Cow<'a, str>>,
    {
        let uri: Cow<'a, str> = uri.into();
        let (item_type, id_index) = parse_item_type_and_id_from_uri(&uri)?;

        match item_type {
            ItemType::Track => Ok(Self::Item(PlayableItem::Track(Id::new(uri, IdKind::Uri(id_index))))),
            ItemType::Episode => Ok(Self::Item(PlayableItem::Episode(Id::new(uri, IdKind::Uri(id_index))))),

            ItemType::Album => Ok(Self::Context(PlayableContext::Album(Id::new(
                uri,
                IdKind::Uri(id_index),
            )))),

            ItemType::Artist => Ok(Self::Context(PlayableContext::Artist(Id::new(
                uri,
                IdKind::Uri(id_index),
            )))),

            ItemType::Playlist => Ok(Self::Context(PlayableContext::Playlist(Id::new(
                uri,
                IdKind::Uri(id_index),
            )))),

            ItemType::Show => Ok(Self::Context(PlayableContext::Show(Id::new(
                uri,
                IdKind::Uri(id_index),
            )))),
        }
    }

    fn from_url<C>(url: C) -> Result<Self, IdError>
    where
        C: Into<Cow<'a, str>>,
    {
        let url: Cow<'a, str> = url.into();
        let (item_type, id_index) = parse_item_type_and_id_from_url(&url)?;

        match item_type {
            ItemType::Track => Ok(Self::Item(PlayableItem::Track(Id::new(url, IdKind::Url(id_index))))),
            ItemType::Episode => Ok(Self::Item(PlayableItem::Episode(Id::new(url, IdKind::Url(id_index))))),

            ItemType::Album => Ok(Self::Context(PlayableContext::Album(Id::new(
                url,
                IdKind::Url(id_index),
            )))),

            ItemType::Artist => Ok(Self::Context(PlayableContext::Artist(Id::new(
                url,
                IdKind::Url(id_index),
            )))),

            ItemType::Playlist => Ok(Self::Context(PlayableContext::Playlist(Id::new(
                url,
                IdKind::Url(id_index),
            )))),

            ItemType::Show => Ok(Self::Context(PlayableContext::Show(Id::new(
                url,
                IdKind::Url(id_index),
            )))),
        }
    }
}

impl<'a, T> IdTrait<'a> for Id<'a, T>
where
    T: ItemTypeId + 'static,
{
    type Owned = Id<'static, T>;
    type Borrowed<'b> = Id<'b, T> where 'a: 'b, Self: 'a;

    fn as_str(&self) -> &str {
        match self.kind {
            IdKind::Uri(index) => &self.value[index..],
            IdKind::Url(index) => &self.value[index..index + ID_LENGTH],
            IdKind::Bare => &self.value,
        }
    }

    fn as_uri(&'a self) -> Cow<'a, str> {
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

    fn as_url(&'a self) -> Cow<'a, str> {
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

    fn as_owned(&'a self) -> Self::Owned {
        Id::new(Cow::Owned(self.value.clone().into_owned()), self.kind)
    }

    fn as_borrowed<'b>(&'a self) -> Self::Borrowed<'b>
    where
        'a: 'b,
    {
        Id::new(Cow::Borrowed(&self.value), self.kind)
    }
}

impl<'a> IdTrait<'a> for SpotifyId<'a> {
    type Owned = SpotifyId<'static>;
    type Borrowed<'b> = SpotifyId<'b> where 'a: 'b, Self: 'a;

    fn as_str(&'a self) -> &'a str {
        match self {
            SpotifyId::Item(item) => item.as_str(),
            SpotifyId::Context(context) => context.as_str(),
        }
    }

    fn as_uri(&'a self) -> Cow<'a, str> {
        match self {
            SpotifyId::Item(item) => item.as_uri(),
            SpotifyId::Context(context) => context.as_uri(),
        }
    }

    fn as_url(&'a self) -> Cow<'a, str> {
        match self {
            SpotifyId::Item(item) => item.as_url(),
            SpotifyId::Context(context) => context.as_url(),
        }
    }

    fn as_owned(&'a self) -> Self::Owned {
        match self {
            SpotifyId::Item(item) => SpotifyId::Item(item.as_owned()),
            SpotifyId::Context(context) => SpotifyId::Context(context.as_owned()),
        }
    }

    fn as_borrowed<'b>(&'a self) -> Self::Borrowed<'b>
    where
        'a: 'b,
    {
        match self {
            SpotifyId::Item(item) => SpotifyId::Item(item.as_borrowed()),
            SpotifyId::Context(context) => SpotifyId::Context(context.as_borrowed()),
        }
    }
}

impl<'a> IdTrait<'a> for PlayableItem<'a> {
    type Owned = PlayableItem<'static>;
    type Borrowed<'b> = PlayableItem<'b> where 'a: 'b, Self: 'a;

    fn as_str(&self) -> &str {
        match self {
            PlayableItem::Track(track) => track.as_str(),
            PlayableItem::Episode(episode) => episode.as_str(),
        }
    }

    fn as_uri(&'a self) -> Cow<'a, str> {
        match self {
            PlayableItem::Track(track) => track.as_uri(),
            PlayableItem::Episode(episode) => episode.as_uri(),
        }
    }

    fn as_url(&'a self) -> Cow<'a, str> {
        match self {
            PlayableItem::Track(track) => track.as_url(),
            PlayableItem::Episode(episode) => episode.as_url(),
        }
    }

    fn as_owned(&'a self) -> Self::Owned {
        match self {
            PlayableItem::Track(track) => PlayableItem::Track(track.as_owned()),
            PlayableItem::Episode(episode) => PlayableItem::Episode(episode.as_owned()),
        }
    }

    fn as_borrowed<'b>(&'a self) -> Self::Borrowed<'b>
    where
        'a: 'b,
    {
        match self {
            PlayableItem::Track(track) => PlayableItem::Track(track.as_borrowed()),
            PlayableItem::Episode(episode) => PlayableItem::Episode(episode.as_borrowed()),
        }
    }
}

impl<'a> IdTrait<'a> for PlayableContext<'a> {
    type Owned = PlayableContext<'static>;
    type Borrowed<'b> = PlayableContext<'b> where 'a: 'b, Self: 'a;

    fn as_str(&self) -> &str {
        match self {
            PlayableContext::Artist(artist) => artist.as_str(),
            PlayableContext::Album(album) => album.as_str(),
            PlayableContext::Playlist(playlist) => playlist.as_str(),
            PlayableContext::Show(show) => show.as_str(),
        }
    }

    fn as_uri(&'a self) -> Cow<'a, str> {
        match self {
            PlayableContext::Artist(artist) => artist.as_uri(),
            PlayableContext::Album(album) => album.as_uri(),
            PlayableContext::Playlist(playlist) => playlist.as_uri(),
            PlayableContext::Show(show) => show.as_uri(),
        }
    }

    fn as_url(&'a self) -> Cow<'a, str> {
        match self {
            PlayableContext::Artist(artist) => artist.as_url(),
            PlayableContext::Album(album) => album.as_url(),
            PlayableContext::Playlist(playlist) => playlist.as_url(),
            PlayableContext::Show(show) => show.as_url(),
        }
    }

    fn as_owned(&'a self) -> Self::Owned {
        match self {
            PlayableContext::Artist(artist) => PlayableContext::Artist(artist.as_owned()),
            PlayableContext::Album(album) => PlayableContext::Album(album.as_owned()),
            PlayableContext::Playlist(playlist) => PlayableContext::Playlist(playlist.as_owned()),
            PlayableContext::Show(show) => PlayableContext::Show(show.as_owned()),
        }
    }

    fn as_borrowed<'b>(&'a self) -> Self::Borrowed<'b>
    where
        'a: 'b,
    {
        match self {
            PlayableContext::Artist(artist) => PlayableContext::Artist(artist.as_borrowed()),
            PlayableContext::Album(album) => PlayableContext::Album(album.as_borrowed()),
            PlayableContext::Playlist(playlist) => PlayableContext::Playlist(playlist.as_borrowed()),
            PlayableContext::Show(show) => PlayableContext::Show(show.as_borrowed()),
        }
    }
}

impl<'a, T> fmt::Display for Id<'a, T>
where
    T: ItemTypeId + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> fmt::Display for PlayableItem<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> fmt::Display for PlayableContext<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'a> fmt::Display for SpotifyId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
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

impl<'a> Serialize for SpotifyId<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SpotifyId::Item(item) => item.serialize(serializer),
            SpotifyId::Context(context) => context.serialize(serializer),
        }
    }
}

impl<'a> Serialize for PlayableItem<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PlayableItem::Track(track_id) => track_id.serialize(serializer),
            PlayableItem::Episode(episode_id) => episode_id.serialize(serializer),
        }
    }
}

impl<'a> Serialize for PlayableContext<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PlayableContext::Artist(artist_id) => artist_id.serialize(serializer),
            PlayableContext::Album(album_id) => album_id.serialize(serializer),
            PlayableContext::Playlist(playlist_id) => playlist_id.serialize(serializer),
            PlayableContext::Show(show_id) => show_id.serialize(serializer),
        }
    }
}

impl<'a, T> Serialize for Id<'a, T>
where
    T: ItemTypeId,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.value)
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
                // Id, and therefore SpotifyId, always store the input string in its entirety in themselves for
                // efficiency. when deserializing a string, it's impossible to reliably borrow the input here since the
                // `v` parameter isn't guaranteed to outlive the visitor. hence, convert it into an owned String and
                // deserialize that
                self.visit_string(v.to_owned())
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (item_type, kind) = parse_item_type_and_kind_from_url_or_uri(&v)
                    .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;

                match item_type {
                    ItemType::Track => Ok(SpotifyId::Item(PlayableItem::Track(Id::new(Cow::Owned(v), kind)))),
                    ItemType::Episode => Ok(SpotifyId::Item(PlayableItem::Episode(Id::new(Cow::Owned(v), kind)))),
                    ItemType::Album => Ok(SpotifyId::Context(PlayableContext::Album(Id::new(Cow::Owned(v), kind)))),

                    ItemType::Artist => Ok(SpotifyId::Context(PlayableContext::Artist(Id::new(
                        Cow::Owned(v),
                        kind,
                    )))),

                    ItemType::Playlist => Ok(SpotifyId::Context(PlayableContext::Playlist(Id::new(
                        Cow::Owned(v),
                        kind,
                    )))),

                    ItemType::Show => Ok(SpotifyId::Context(PlayableContext::Show(Id::new(Cow::Owned(v), kind)))),
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
                // Id, and therefore PlayableItem, always store the input string in its entirety in themselves for
                // efficiency. when deserializing a string, it's impossible to reliably borrow the input here since the
                // `v` parameter isn't guaranteed to outlive the visitor. hence, convert it into an owned String and
                // deserialize that
                self.visit_string(v.to_owned())
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (item_type, kind) = parse_item_type_and_kind_from_url_or_uri(&v)
                    .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;

                match item_type {
                    ItemType::Track => Ok(PlayableItem::Track(Id::new(Cow::Owned(v), kind))),
                    ItemType::Episode => Ok(PlayableItem::Episode(Id::new(Cow::Owned(v), kind))),

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
                // Id, and therefore PlayableContext, always store the input string in its entirety in themselves for
                // efficiency. when deserializing a string, it's impossible to reliably borrow the input here since the
                // `v` parameter isn't guaranteed to outlive the visitor. hence, convert it into an owned String and
                // deserialize that
                self.visit_string(v.to_owned())
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (item_type, kind) = parse_item_type_and_kind_from_url_or_uri(&v)
                    .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;

                match item_type {
                    ItemType::Album => Ok(PlayableContext::Album(Id::new(Cow::Owned(v), kind))),
                    ItemType::Artist => Ok(PlayableContext::Artist(Id::new(Cow::Owned(v), kind))),
                    ItemType::Playlist => Ok(PlayableContext::Playlist(Id::new(Cow::Owned(v), kind))),
                    ItemType::Show => Ok(PlayableContext::Show(Id::new(Cow::Owned(v), kind))),

                    _ => Err(de::Error::invalid_type(de::Unexpected::Str(&v), &self)),
                }
            }
        }

        deserializer.deserialize_string(IdVisitor)
    }
}

impl<'de, T> Deserialize<'de> for Id<'static, T>
where
    T: ItemTypeId + 'static,
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
                // Id always stores the input string in its entirety in it for efficiency. when deserializing a string,
                // it's impossible to reliably borrow the input here since the `v` parameter isn't guaranteed to outlive
                // the visitor. hence, convert it into an owned String and deserialize that
                self.visit_string(v.to_owned())
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (_, kind) = parse_item_type_and_kind_from_url_or_uri(&v)
                    .or_else(|_| {
                        if verify_valid_id(&v) {
                            Ok((T::ITEM_TYPE, IdKind::Bare))
                        } else {
                            Err(IdError::InvalidId(v.clone()))
                        }
                    })
                    .and_then(|(item_type, kind)| {
                        if item_type == T::ITEM_TYPE {
                            Ok((item_type, kind))
                        } else {
                            Err(IdError::WrongItemType(item_type))
                        }
                    })
                    .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(&v), &self))?;

                Ok(Id::new(Cow::Owned(v), kind))
            }
        }

        deserializer.deserialize_string(IdVisitor::<T> { phantom: PhantomData })
    }
}

fn parse_item_type_and_kind_from_url_or_uri(url_or_uri: &str) -> Result<(ItemType, IdKind), IdError> {
    if url_or_uri.starts_with(URI_PREFIX) {
        let (item_type, id_index) = parse_item_type_and_id_from_uri(url_or_uri)?;

        Ok((item_type, IdKind::Uri(id_index)))
    } else if url_or_uri.starts_with(URL_PREFIX) {
        let (item_type, id_index) = parse_item_type_and_id_from_url(url_or_uri)?;

        Ok((item_type, IdKind::Url(id_index)))
    } else {
        Err(IdError::MalformedString(url_or_uri.to_string()))
    }
}

fn parse_item_type_and_id_from_uri(uri: &str) -> Result<(ItemType, usize), IdError> {
    if let Some((item_type, id)) = uri
        .strip_prefix(URI_PREFIX)
        .and_then(|prefix_removed| prefix_removed.split_once(':'))
    {
        let item_type: ItemType = item_type.parse()?;

        if verify_valid_id(id) {
            // the ID is always at the end of the string
            Ok((item_type, uri.len() - ID_LENGTH))
        } else {
            Err(IdError::InvalidId(id.to_owned()))
        }
    } else {
        Err(IdError::MalformedString(uri.to_string()))
    }
}

fn parse_item_type_and_id_from_url(url: &str) -> Result<(ItemType, usize), IdError> {
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
            Err(IdError::InvalidId(id.to_owned()))
        }
    } else {
        Err(IdError::MalformedString(url.to_string()))
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

    #[test]
    fn track_id_from_uri() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn track_id_from_url() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn track_id_from_url_with_query() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu?si=AAAAAAAAAA";
        let id = Id::<TrackId>::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn track_id_from_bare() {
        let id_string = "2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_bare(id_string).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn album_id_from_uri() {
        let id_string = "spotify:album:0tDsHtvN9YNuZjlqHvDY2P";
        let id = Id::<AlbumId>::from_uri(id_string).unwrap();

        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn album_id_from_url() {
        let id_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P";
        let id = Id::<AlbumId>::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn album_id_from_bare() {
        let id_string = "0tDsHtvN9YNuZjlqHvDY2P";
        let id = Id::<AlbumId>::from_bare(id_string).unwrap();

        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn artist_id_from_uri() {
        let id_string = "spotify:artist:6pNgnvzBa6Bthsv8SrZJYl";
        let id = Id::<ArtistId>::from_uri(id_string).unwrap();

        assert_eq!(id.as_str(), "6pNgnvzBa6Bthsv8SrZJYl");
    }

    #[test]
    fn artist_id_from_url() {
        let id_string = "https://open.spotify.com/artist/6pNgnvzBa6Bthsv8SrZJYl";
        let id = Id::<ArtistId>::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "6pNgnvzBa6Bthsv8SrZJYl");
    }

    #[test]
    fn artist_id_from_bare() {
        let id_string = "6pNgnvzBa6Bthsv8SrZJYl";
        let id = Id::<ArtistId>::from_bare(id_string).unwrap();

        assert_eq!(id.as_str(), "6pNgnvzBa6Bthsv8SrZJYl");
    }

    #[test]
    fn playlist_id_from_uri() {
        let id_string = "spotify:playlist:37i9dQZF1DWZipvLjDtZYe";
        let id = Id::<PlaylistId>::from_uri(id_string).unwrap();

        assert_eq!(id.as_str(), "37i9dQZF1DWZipvLjDtZYe");
    }

    #[test]
    fn playlist_id_from_url() {
        let id_string = "https://open.spotify.com/playlist/37i9dQZF1DWZipvLjDtZYe";
        let id = Id::<PlaylistId>::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "37i9dQZF1DWZipvLjDtZYe");
    }

    #[test]
    fn playlist_id_from_bare() {
        let id_string = "37i9dQZF1DWZipvLjDtZYe";
        let id = Id::<PlaylistId>::from_bare(id_string).unwrap();

        assert_eq!(id.as_str(), "37i9dQZF1DWZipvLjDtZYe");
    }

    #[test]
    fn playable_item_id_from_track_uri() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = PlayableItem::from_uri(id_string).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn playable_item_id_from_track_url() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = PlayableItem::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn playable_item_id_from_track_url_with_query() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu?si=AAAAAAAAAA";
        let id = PlayableItem::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn playable_context_id_from_album_uri() {
        let id_string = "spotify:album:0tDsHtvN9YNuZjlqHvDY2P";
        let id = PlayableContext::from_uri(id_string).unwrap();

        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn playable_context_id_from_album_url() {
        let id_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P";
        let id = PlayableContext::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn playable_context_id_from_album_url_with_query() {
        let id_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P?si=AAAAAAAAAA";
        let id = PlayableContext::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn spotify_id_from_track_uri() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = SpotifyId::from_uri(id_string).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn spotify_id_from_track_url() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = SpotifyId::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn spotify_id_from_track_url_with_query() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu?si=AAAAAAAAAA";
        let id = SpotifyId::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn spotify_id_from_album_uri() {
        let id_string = "spotify:album:0tDsHtvN9YNuZjlqHvDY2P";
        let id = SpotifyId::from_uri(id_string).unwrap();

        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn spotify_id_from_album_url() {
        let id_string = "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P";
        let id = SpotifyId::from_url(id_string).unwrap();

        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn spotify_id_from_track_uri_and_url() {
        let url_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let uri_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";

        let url_id = SpotifyId::from_url_or_uri(url_string).unwrap();
        let uri_id = SpotifyId::from_url_or_uri(uri_string).unwrap();

        assert_eq!(url_id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
        assert_eq!(uri_id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn wrong_url_prefix() {
        let id_string = "https://google.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_url(id_string);

        assert!(matches!(id, Err(IdError::MalformedString(_))))
    }

    #[test]
    fn wrong_uri_prefix() {
        let id_string = "wrong:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string);

        assert!(matches!(id, Err(IdError::MalformedString(_))))
    }

    #[test]
    fn wrong_id_type_in_url() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<ArtistId>::from_url(id_string);

        assert!(matches!(id, Err(IdError::WrongItemType(ItemType::Track))))
    }

    #[test]
    fn wrong_id_type_in_uri() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<ArtistId>::from_uri(id_string);

        assert!(matches!(id, Err(IdError::WrongItemType(ItemType::Track))))
    }

    #[test]
    fn unknown_id_type_in_uri() {
        let id_string = "spotify:wrong:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string);

        assert!(matches!(id, Err(IdError::InvalidItemType(_))))
    }

    #[test]
    fn unknown_id_type_in_url() {
        let id_string = "https://open.spotify.com/wrong/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_url(id_string);

        assert!(matches!(id, Err(IdError::InvalidItemType(_))))
    }

    #[test]
    fn invalid_id_too_short() {
        let id_string = "_";
        let id = Id::<TrackId>::from_bare(id_string);

        assert!(matches!(id, Err(IdError::InvalidId(_))))
    }

    #[test]
    fn invalid_id_too_long() {
        let id_string = "2pDPOMX0kWA7kcPBcDCQBu_";
        let id = Id::<TrackId>::from_bare(id_string);

        assert!(matches!(id, Err(IdError::InvalidId(_))))
    }

    #[test]
    fn invalid_id_illegal_characters() {
        let id_string = "2pDPOMX0kWA7kcPBcDCQB_";
        let id = Id::<TrackId>::from_bare(id_string);

        assert!(matches!(id, Err(IdError::InvalidId(_))))
    }

    #[test]
    fn invalid_id_in_url() {
        let id_string = "https://open.spotify.com/track/_";
        let id = Id::<TrackId>::from_url(id_string);

        assert!(matches!(id, Err(IdError::InvalidId(_))))
    }

    #[test]
    fn invalid_id_in_uri() {
        let id_string = "spotify:track:_";
        let id = Id::<TrackId>::from_uri(id_string);

        assert!(matches!(id, Err(IdError::InvalidId(_))))
    }

    #[test]
    fn uri_from_uri_borrows() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string).unwrap();

        let uri = id.as_uri();
        assert!(matches!(uri, Cow::Borrowed("spotify:track:2pDPOMX0kWA7kcPBcDCQBu")));
    }

    #[test]
    fn url_from_url_borrows() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_url(id_string).unwrap();

        let url = id.as_url();
        assert!(matches!(
            url,
            Cow::Borrowed("https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu")
        ));
    }

    #[test]
    fn uri_from_url_allocates() {
        let id_string = "https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_url(id_string).unwrap();

        let uri = id.as_uri();
        assert!(matches!(uri, Cow::Owned(_)));
    }

    #[test]
    fn url_from_uri_allocates() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string).unwrap();

        let url = id.as_url();
        assert!(matches!(url, Cow::Owned(_)));
    }

    #[test]
    fn uri_from_bare_allocates() {
        let id_string = "2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_bare(id_string).unwrap();

        let uri = id.as_uri();
        assert!(matches!(uri, Cow::Owned(_)));
    }

    #[test]
    fn url_from_bare_allocates() {
        let id_string = "2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_bare(id_string).unwrap();

        let url = id.as_url();
        assert!(matches!(url, Cow::Owned(_)));
    }

    #[test]
    fn cloned_id_still_borrows() {
        let id_string = "spotify:track:2pDPOMX0kWA7kcPBcDCQBu";
        let id = Id::<TrackId>::from_uri(id_string).unwrap();

        let url = id.as_uri();
        assert!(matches!(url, Cow::Borrowed(_)));

        let cloned = id.clone();
        let url = cloned.as_uri();
        assert!(matches!(url, Cow::Borrowed(_)));
    }

    #[test]
    fn deserialize_id_from_uri() {
        let json = "\"spotify:track:2pDPOMX0kWA7kcPBcDCQBu\"";
        let id: Id<TrackId> = serde_json::from_str(json).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn deserialize_id_from_url() {
        let json = "\"https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu\"";
        let id: Id<TrackId> = serde_json::from_str(json).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn deserialize_id_from_bare() {
        let json = "\"2pDPOMX0kWA7kcPBcDCQBu\"";
        let id: Id<TrackId> = serde_json::from_str(json).unwrap();

        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn deserialize_playable_item_from_uri() {
        let json = "\"spotify:track:2pDPOMX0kWA7kcPBcDCQBu\"";
        let id: PlayableItem = serde_json::from_str(json).unwrap();

        assert!(matches!(id, PlayableItem::Track(_)));
        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
    }

    #[test]
    fn deserialize_playable_item_from_url() {
        let json = "\"https://open.spotify.com/track/2pDPOMX0kWA7kcPBcDCQBu\"";
        let id: PlayableItem = serde_json::from_str(json).unwrap();

        assert!(matches!(id, PlayableItem::Track(_)));
        assert_eq!(id.as_str(), "2pDPOMX0kWA7kcPBcDCQBu");
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
        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn deserialize_playable_context_from_url() {
        let json = "\"https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P\"";
        let id: PlayableContext = serde_json::from_str(json).unwrap();

        assert!(matches!(id, PlayableContext::Album(_)));
        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
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
        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn deserialize_spotify_id_from_url() {
        let json = "\"https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P\"";
        let id: SpotifyId = serde_json::from_str(json).unwrap();

        assert!(matches!(id, SpotifyId::Context(PlayableContext::Album(_))));
        assert_eq!(id.as_str(), "0tDsHtvN9YNuZjlqHvDY2P");
    }

    #[test]
    fn cannot_deserialize_spotify_id_from_bare() {
        let json = "\"0tDsHtvN9YNuZjlqHvDY2P\"";
        let result: std::result::Result<SpotifyId, _> = serde_json::from_str(json);

        assert!(result.is_err());
    }
}
