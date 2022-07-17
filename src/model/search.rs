use super::{
    album::{AlbumObject, FullAlbum},
    artist::{ArtistObject, FullArtist},
    page::Page,
    track::{FullTrack, TrackObject},
};

use serde::Deserialize;
use std::fmt::Display;

pub trait ToTypesString {
    fn to_types_string(self) -> String;
}

#[derive(Debug, Deserialize)]
pub struct SearchResults {
    tracks: Option<Page<TrackObject>>,
    artists: Option<Page<ArtistObject>>,
    albums: Option<Page<AlbumObject>>,
    // playlists: Page<Playlist>,
    // shows: Page<Show>,
    // episodes: Page<Episode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    Album,
    Artist,
    Playlist,
    Track,
    Show,
    Episode,
}

impl SearchResults {
    pub fn tracks(self) -> Option<Page<FullTrack>> {
        self.tracks.map(|page| Page {
            items: page.items.into_iter().map(FullTrack::from).collect(),
            limit: page.limit,
            offset: page.offset,
            total: page.total,
        })
    }

    pub fn artists(self) -> Option<Page<FullArtist>> {
        self.artists.map(|page| Page {
            items: page.items.into_iter().map(FullArtist::from).collect(),
            limit: page.limit,
            offset: page.offset,
            total: page.total,
        })
    }

    pub fn albums(self) -> Option<Page<FullAlbum>> {
        self.albums.map(|page| Page {
            items: page.items.into_iter().map(FullAlbum::from).collect(),
            limit: page.limit,
            offset: page.offset,
            total: page.total,
        })
    }
}

impl Display for SearchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchType::Album => write!(f, "album"),
            SearchType::Artist => write!(f, "artist"),
            SearchType::Playlist => write!(f, "playlist"),
            SearchType::Track => write!(f, "track"),
            SearchType::Show => write!(f, "show"),
            SearchType::Episode => write!(f, "episode"),
        }
    }
}

impl<I> ToTypesString for I
where
    I: IntoIterator<Item = SearchType>,
{
    fn to_types_string(self) -> String {
        self.into_iter()
            .map(|ty| ty.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}
