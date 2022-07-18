use super::ItemType;
use crate::error::{IdError, Result};
use serde::Deserialize;
use std::{borrow::Cow, marker::PhantomData};

const ID_LENGTH: usize = 22; // I hope Spotify never changes this length
const URL_PREFIX: &str = "https://open.spotify.com/";
const URI_PREFIX: &str = "spotify:";

mod private {
    pub trait Sealed {}
}

pub trait ItemTypeId: private::Sealed {
    const ITEM_TYPE: ItemType;
}

pub trait IdTrait<'a>: private::Sealed {
    fn id(&'a self) -> &'a str;
    fn uri(&'a self) -> Cow<'a, str>;
}

pub trait IdFromUri<'a>: private::Sealed
where
    Self: Sized,
{
    fn from_uri<C>(uri: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>;
}

pub trait IdFromUrl<'a>: private::Sealed
where
    Self: Sized,
{
    fn from_url<C>(url: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>;
}

pub trait IdFromBare<'a>: private::Sealed
where
    Self: Sized,
{
    fn from_bare<C>(bare: C) -> Result<Self>
    where
        C: Into<Cow<'a, str>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id<'a, T>
where
    T: ItemTypeId,
{
    value: IdValue<'a>,
    phantom: PhantomData<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IdValue<'a> {
    Uri(usize, Cow<'a, str>),
    Url(usize, Cow<'a, str>),
    Bare(Cow<'a, str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayableItem<'a> {
    Track(Id<'a, TrackId>),
    Episode(Id<'a, EpisodeId>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayableContext<'a> {
    Artist(Id<'a, ArtistId>),
    Album(Id<'a, AlbumId>),
    Playlist(Id<'a, PlaylistId>),
    Show(Id<'a, ShowId>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtistId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowId;

impl<'a, T> private::Sealed for Id<'a, T> where T: ItemTypeId {}
impl private::Sealed for TrackId {}
impl private::Sealed for EpisodeId {}
impl private::Sealed for ArtistId {}
impl private::Sealed for AlbumId {}
impl private::Sealed for PlaylistId {}
impl private::Sealed for ShowId {}

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
                value: IdValue::Uri(id_index, uri),
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
                value: IdValue::Url(id_index, uri),
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
                value: IdValue::Bare(bare),
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
                value: IdValue::Uri(id_index, uri),
                phantom: PhantomData,
            })),

            ItemType::Episode => Ok(Self::Episode(Id {
                value: IdValue::Uri(id_index, uri),
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
                value: IdValue::Url(id_index, url),
                phantom: PhantomData,
            })),

            ItemType::Episode => Ok(Self::Episode(Id {
                value: IdValue::Url(id_index, url),
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
                value: IdValue::Uri(id_index, uri),
                phantom: PhantomData,
            })),

            ItemType::Artist => Ok(Self::Artist(Id {
                value: IdValue::Uri(id_index, uri),
                phantom: PhantomData,
            })),

            ItemType::Playlist => Ok(Self::Playlist(Id {
                value: IdValue::Uri(id_index, uri),
                phantom: PhantomData,
            })),

            ItemType::Show => Ok(Self::Show(Id {
                value: IdValue::Uri(id_index, uri),
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
                value: IdValue::Url(id_index, url),
                phantom: PhantomData,
            })),

            ItemType::Artist => Ok(Self::Artist(Id {
                value: IdValue::Url(id_index, url),
                phantom: PhantomData,
            })),

            ItemType::Playlist => Ok(Self::Playlist(Id {
                value: IdValue::Url(id_index, url),
                phantom: PhantomData,
            })),

            ItemType::Show => Ok(Self::Show(Id {
                value: IdValue::Url(id_index, url),
                phantom: PhantomData,
            })),

            item_type => Err(IdError::WrongItemType(item_type).into()),
        }
    }
}

impl<'a, T> IdTrait<'a> for Id<'a, T>
where
    T: ItemTypeId,
{
    fn id(&self) -> &str {
        match self.value {
            IdValue::Uri(index, ref uri) => &uri[index..],
            IdValue::Url(index, ref url) => &url[index..index + ID_LENGTH],
            IdValue::Bare(ref bare) => bare,
        }
    }

    fn uri(&'a self) -> Cow<'a, str> {
        match &self.value {
            IdValue::Uri(_, uri) => match uri {
                Cow::Borrowed(b) => Cow::Borrowed(b),
                Cow::Owned(o) => Cow::Borrowed(o),
            },

            IdValue::Url(index, ref url) => {
                Cow::Owned(format!("spotify:{}:{}", T::ITEM_TYPE, &url[*index..*index + ID_LENGTH]))
            }

            IdValue::Bare(bare) => Cow::Owned(format!("spotify:{}:{}", T::ITEM_TYPE, bare)),
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
        todo!()
    }
}

pub(crate) fn parse_item_type_and_id_from_uri(uri: &str) -> Result<(ItemType, usize)> {
    if let Some((item_type, id)) = uri
        .strip_prefix(URI_PREFIX)
        .and_then(|prefix_removed| prefix_removed.split_once(':'))
    {
        let item_type: ItemType = item_type.parse()?;

        if !verify_valid_id(id) {
            Err(IdError::InvalidId(id.to_owned()).into())
        } else {
            // the ID is always at the end of the string
            Ok((item_type, uri.len() - ID_LENGTH))
        }
    } else {
        Err(IdError::MalformedString(uri.to_string()).into())
    }
}

pub(crate) fn parse_item_type_and_id_from_url(url: &str) -> Result<(ItemType, usize)> {
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

        if !verify_valid_id(id) {
            Err(IdError::InvalidId(id.to_owned()).into())
        } else {
            // the position of the ID in the string is the domain + the type + /
            Ok((item_type, URL_PREFIX.len() + item_type_str.len() + 1))
        }
    } else {
        Err(IdError::MalformedString(url.to_string()).into())
    }
}

pub(crate) fn verify_valid_id(id: &str) -> bool {
    // Spotify IDs are base-62 strings and they look like 3mXLyNsVeLelMakgpGUp1f
    if id.len() != 22 {
        return false;
    }

    for c in id.chars() {
        if !c.is_ascii_alphabetic() && !c.is_ascii_digit() {
            return false;
        }
    }

    true
}
