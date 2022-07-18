use self::private::PrivateId;
use super::ItemType;
use crate::error::{IdError, Result};
use serde::Deserialize;
use std::borrow::Cow;

mod private {
    use crate::model::ItemType;
    use std::borrow::Cow;

    pub trait PrivateId<'a>: super::Id<ItemType> {
        const ITEM_TYPE: ItemType;
        fn new(id: Cow<'a, str>) -> Self;
    }
}

pub trait ParseableId<'a>
where
    Self: Sized,
{
    fn from_uri<S>(uri: &'a str) -> Result<Self>;
}

pub trait Id<T>
where
    T: std::fmt::Display,
{
    fn item_type(&self) -> T;
    fn id(&self) -> &str;

    fn format_uri(&self) -> String {
        format!("spotify:{}:{}", self.item_type(), self.id())
    }

    fn format_url(&self) -> String {
        format!("https://open.spotify.com/{}/{}", self.item_type(), self.id())
    }
}

impl<'a, T> ParseableId<'a> for T
where
    T: private::PrivateId<'a> + Sized,
{
    fn from_uri<S>(uri: &'a str) -> Result<Self> {
        let (item_type, id) = parse_item_type_and_id(uri)?;

        if item_type == Self::ITEM_TYPE {
            Ok(Self::new(Cow::Borrowed(id)))
        } else {
            Err(IdError::WrongItemType(item_type).into())
        }
    }
}

pub(crate) fn parse_item_type_and_id(uri: &str) -> Result<(ItemType, &str)> {
    if let Some((item_type, id)) = uri
        .strip_prefix("spotify:")
        .and_then(|prefix_removed| prefix_removed.split_once(':'))
    {
        let item_type: ItemType = item_type.parse()?;

        if !verify_valid_id(id) {
            Err(IdError::InvalidId(id.to_owned()).into())
        } else {
            Ok((item_type, id))
        }
    } else {
        Err(IdError::MalformedString(uri.to_owned()).into())
    }
}

pub(crate) fn verify_valid_id(id: &str) -> bool {
    // Spotify IDs are base-62 strings and they look like 6rqhFgbbKwnb9MLmUQDhG6
    if id.len() != 22 {
        return false;
    }

    for c in id.chars() {
        if !c.is_ascii_alphabetic() || !c.is_ascii_digit() {
            return false;
        }
    }

    true
}

pub enum PlayableItem<'a> {
    Track(TrackId<'a>),
    Episode(EpisodeId<'a>),
}

pub enum PlayableContext<'a> {
    Artist(ArtistId<'a>),
    Album(AlbumId<'a>),
    Playlist(PlaylistId<'a>),
    Show(ShowId<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TrackId<'a>(Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EpisodeId<'a>(Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ArtistId<'a>(Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AlbumId<'a>(Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlaylistId<'a>(Cow<'a, str>);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ShowId<'a>(Cow<'a, str>);

impl<'a> ParseableId<'a> for PlayableItem<'a> {
    fn from_uri<S>(uri: &'a str) -> Result<Self> {
        let (item_type, id) = parse_item_type_and_id(uri)?;

        match item_type {
            ItemType::Track => Ok(Self::Track(TrackId::new(Cow::Borrowed(id)))),
            ItemType::Episode => Ok(Self::Episode(EpisodeId::new(Cow::Borrowed(id)))),

            _ => Err(IdError::WrongItemType(item_type).into()),
        }
    }
}

impl<'a> ParseableId<'a> for PlayableContext<'a> {
    fn from_uri<S>(uri: &'a str) -> Result<Self> {
        let (item_type, id) = parse_item_type_and_id(uri)?;

        match item_type {
            ItemType::Album => Ok(Self::Album(AlbumId::new(Cow::Borrowed(id)))),
            ItemType::Artist => Ok(Self::Artist(ArtistId::new(Cow::Borrowed(id)))),
            ItemType::Playlist => Ok(Self::Playlist(PlaylistId::new(Cow::Borrowed(id)))),
            ItemType::Show => Ok(Self::Show(ShowId::new(Cow::Borrowed(id)))),

            _ => Err(IdError::WrongItemType(item_type).into()),
        }
    }
}

impl Id<ItemType> for PlayableItem<'_> {
    fn id(&self) -> &str {
        match self {
            PlayableItem::Track(track) => track.id(),
            PlayableItem::Episode(episode) => episode.id(),
        }
    }

    fn item_type(&self) -> ItemType {
        match self {
            PlayableItem::Track(track) => track.item_type(),
            PlayableItem::Episode(episode) => episode.item_type(),
        }
    }
}

impl Id<ItemType> for PlayableContext<'_> {
    fn id(&self) -> &str {
        match self {
            PlayableContext::Artist(artist) => artist.id(),
            PlayableContext::Album(album) => album.id(),
            PlayableContext::Playlist(playlist) => playlist.id(),
            PlayableContext::Show(show) => show.id(),
        }
    }

    fn item_type(&self) -> ItemType {
        match self {
            PlayableContext::Artist(artist) => artist.item_type(),
            PlayableContext::Album(album) => album.item_type(),
            PlayableContext::Playlist(playlist) => playlist.item_type(),
            PlayableContext::Show(show) => show.item_type(),
        }
    }
}

impl<'a> private::PrivateId<'a> for TrackId<'a> {
    const ITEM_TYPE: ItemType = ItemType::Track;

    fn new(id: Cow<'a, str>) -> Self {
        Self(id)
    }
}

impl<'a> private::PrivateId<'a> for EpisodeId<'a> {
    const ITEM_TYPE: ItemType = ItemType::Episode;

    fn new(id: Cow<'a, str>) -> Self {
        Self(id)
    }
}

impl<'a> private::PrivateId<'a> for ArtistId<'a> {
    const ITEM_TYPE: ItemType = ItemType::Artist;

    fn new(id: Cow<'a, str>) -> Self {
        Self(id)
    }
}

impl<'a> private::PrivateId<'a> for AlbumId<'a> {
    const ITEM_TYPE: ItemType = ItemType::Album;

    fn new(id: Cow<'a, str>) -> Self {
        Self(id)
    }
}

impl<'a> private::PrivateId<'a> for PlaylistId<'a> {
    const ITEM_TYPE: ItemType = ItemType::Playlist;

    fn new(id: Cow<'a, str>) -> Self {
        Self(id)
    }
}

impl<'a> private::PrivateId<'a> for ShowId<'a> {
    const ITEM_TYPE: ItemType = ItemType::Show;

    fn new(id: Cow<'a, str>) -> Self {
        Self(id)
    }
}

impl Id<ItemType> for TrackId<'_> {
    fn item_type(&self) -> ItemType {
        Self::ITEM_TYPE
    }

    fn id(&self) -> &str {
        &self.0
    }
}

impl Id<ItemType> for EpisodeId<'_> {
    fn item_type(&self) -> ItemType {
        Self::ITEM_TYPE
    }

    fn id(&self) -> &str {
        &self.0
    }
}

impl Id<ItemType> for ArtistId<'_> {
    fn item_type(&self) -> ItemType {
        Self::ITEM_TYPE
    }

    fn id(&self) -> &str {
        &self.0
    }
}

impl Id<ItemType> for AlbumId<'_> {
    fn item_type(&self) -> ItemType {
        Self::ITEM_TYPE
    }

    fn id(&self) -> &str {
        &self.0
    }
}

impl Id<ItemType> for PlaylistId<'_> {
    fn item_type(&self) -> ItemType {
        Self::ITEM_TYPE
    }

    fn id(&self) -> &str {
        &self.0
    }
}

impl Id<ItemType> for ShowId<'_> {
    fn item_type(&self) -> ItemType {
        Self::ITEM_TYPE
    }

    fn id(&self) -> &str {
        &self.0
    }
}

impl<'a> From<TrackId<'a>> for PlayableItem<'a> {
    fn from(id: TrackId<'a>) -> Self {
        Self::Track(id)
    }
}

impl<'a> From<EpisodeId<'a>> for PlayableItem<'a> {
    fn from(id: EpisodeId<'a>) -> Self {
        Self::Episode(id)
    }
}

impl<'a> From<ArtistId<'a>> for PlayableContext<'a> {
    fn from(id: ArtistId<'a>) -> Self {
        Self::Artist(id)
    }
}

impl<'a> From<AlbumId<'a>> for PlayableContext<'a> {
    fn from(id: AlbumId<'a>) -> Self {
        Self::Album(id)
    }
}

impl<'a> From<PlaylistId<'a>> for PlayableContext<'a> {
    fn from(id: PlaylistId<'a>) -> Self {
        Self::Playlist(id)
    }
}

impl<'a> From<ShowId<'a>> for PlayableContext<'a> {
    fn from(id: ShowId<'a>) -> Self {
        Self::Show(id)
    }
}
