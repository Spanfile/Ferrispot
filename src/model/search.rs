use super::{
    album::{AlbumObject, FullAlbum},
    artist::{ArtistObject, FullArtist},
    page::Page,
    track::{FullTrack, TrackObject},
    ItemType,
};

use serde::Deserialize;

// TODO: it'd be really cool if this was a const fn or smth
/// The default search types.
pub const DEFAULT_SEARCH_TYPES_STRING: &str = "album,artist,playlist,track,show,episode";

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

impl<I> ToTypesString for I
where
    I: IntoIterator<Item = ItemType>,
{
    fn to_types_string(self) -> String {
        self.into_iter()
            .map(|ty| ty.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}
