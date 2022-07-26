use super::{
    album::{AlbumObject, FullAlbum},
    artist::{ArtistObject, FullArtist},
    page::{Page, PageInformation, PageObject},
    track::{FullTrack, TrackObject},
    ItemType,
};

use crate::client::private::SendHttpRequest;

use serde::Deserialize;
use std::marker::PhantomData;

// TODO: it'd be really cool if this was a const fn or smth
/// The default search types.
pub const DEFAULT_SEARCH_TYPES_STRING: &str = "album,artist,playlist,track,show,episode";

/// Trait for converting an object to a string used in Spotify's search types. This is currently implemented for all
/// iterators of [ItemType]-enums.
pub trait ToTypesString: crate::private::Sealed {
    fn to_types_string(self) -> String;
}

/// First pages of search results from a [search](crate::client::unscoped::UnscopedClient::search).
#[derive(Debug)]
pub struct SearchResults<'a, C>
where
    C: SendHttpRequest<'a>,
{
    pub(crate) inner: SearchResultsObject,
    pub(crate) client: &'a C,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SearchResultsObject {
    tracks: Option<PageObject<TrackObject, FullTrack>>,
    artists: Option<PageObject<ArtistObject, FullArtist>>,
    albums: Option<PageObject<AlbumObject, FullAlbum>>,
    // playlists: Page<Playlist>,
    // shows: Page<Show>,
    // episodes: Page<Episode>,
}

/// Continuation page of search results from a [search](crate::client::unscoped::UnscopedClient::search) that contains
/// only tracks.
///
/// This object is retrieved only through requesting the [next page](Page::next_page) from an existing page of results.
/// You won't be interacting objects of this type directly.
#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct TrackSearchResults {
    tracks: PageObject<TrackObject, FullTrack>,
}

/// Continuation page of search results from a [search](crate::client::unscoped::UnscopedClient::search) that contains
/// only artists.
///
/// This object is retrieved only through requesting the [next page](Page::next_page) from an existing page of results.
/// You won't be interacting objects of this type directly.
#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct ArtistSearchResults {
    artists: PageObject<ArtistObject, FullArtist>,
}

/// Continuation page of search results from a [search](crate::client::unscoped::UnscopedClient::search) that contains
/// only albums.
///
/// This object is retrieved only through requesting the [next page](Page::next_page) from an existing page of results.
/// You won't be interacting objects of this type directly.
#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct AlbumSearchResults {
    albums: PageObject<AlbumObject, FullAlbum>,
}

impl<'a, C> SearchResults<'a, C>
where
    C: SendHttpRequest<'a>,
{
    /// Return the tracks in these search results as a [Page] of [FullTracks](FullTrack).
    ///
    /// If no tracks matched the search query, this will return None. Therefore, the returned page will always contain
    /// some items.
    pub fn tracks(self) -> Option<Page<'a, TrackSearchResults, FullTrack, C>> {
        self.inner.tracks.and_then(|page| {
            if !page.items().is_empty() {
                Some(Page {
                    inner: TrackSearchResults { tracks: page },
                    client: self.client,
                    phantom: PhantomData,
                })
            } else {
                None
            }
        })
    }

    /// Return the artists in these search results as a [Page] of [FullArtists](FullArtist).
    ///
    /// If no artists matched the search query, this will return None. Therefore, the returned page will always contain
    /// some items.
    pub fn artists(self) -> Option<Page<'a, ArtistSearchResults, FullArtist, C>> {
        self.inner.artists.and_then(|page| {
            if !page.items().is_empty() {
                Some(Page {
                    inner: ArtistSearchResults { artists: page },
                    client: self.client,
                    phantom: PhantomData,
                })
            } else {
                None
            }
        })
    }

    /// Return the albums in these search results as a [Page] of [FullAlbums](FullAlbum).
    ///
    /// If no albums matched the search query, this will return None. Therefore, the returned page will always contain
    /// some items.
    pub fn albums(self) -> Option<Page<'a, AlbumSearchResults, FullAlbum, C>> {
        self.inner.albums.and_then(|page| {
            if !page.items().is_empty() {
                Some(Page {
                    inner: AlbumSearchResults { albums: page },
                    client: self.client,
                    phantom: PhantomData,
                })
            } else {
                None
            }
        })
    }
}

impl crate::private::Sealed for TrackSearchResults {}
impl crate::private::Sealed for ArtistSearchResults {}
impl crate::private::Sealed for AlbumSearchResults {}

impl PageInformation<FullTrack> for TrackSearchResults {
    type Items = Vec<FullTrack>;

    fn items(&self) -> Self::Items {
        self.tracks.items()
    }

    fn take_items(self) -> Self::Items {
        self.tracks.take_items()
    }

    fn next(&self) -> Option<&str> {
        self.tracks.next()
    }
}

impl PageInformation<FullArtist> for ArtistSearchResults {
    type Items = Vec<FullArtist>;

    fn items(&self) -> Self::Items {
        self.artists.items()
    }

    fn take_items(self) -> Self::Items {
        self.artists.take_items()
    }

    fn next(&self) -> Option<&str> {
        self.artists.next()
    }
}

impl PageInformation<FullAlbum> for AlbumSearchResults {
    type Items = Vec<FullAlbum>;

    fn items(&self) -> Self::Items {
        self.albums.items()
    }

    fn take_items(self) -> Self::Items {
        self.albums.take_items()
    }

    fn next(&self) -> Option<&str> {
        self.albums.next()
    }
}

// this is a bit cursed but hey
impl<I> crate::private::Sealed for I where I: IntoIterator<Item = ItemType> {}

impl<I> ToTypesString for I
where
    I: IntoIterator<Item = ItemType> + crate::private::Sealed,
{
    fn to_types_string(self) -> String {
        self.into_iter()
            .map(|ty| ty.to_string())
            .collect::<Vec<String>>()
            .join(",")
    }
}
