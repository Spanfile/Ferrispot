mod private {
    use serde::Deserialize;

    use crate::model::{
        page::PageObject,
        search::{AlbumObject, ArtistObject},
        track::TrackObject,
    };

    #[derive(Debug, Deserialize)]
    pub struct SearchResultsObject {
        pub tracks: Option<PageObject<TrackObject>>,
        pub artists: Option<PageObject<ArtistObject>>,
        pub albums: Option<PageObject<AlbumObject>>,
        // playlists: Page<Playlist>,
        // shows: Page<Show>,
        // episodes: Page<Episode>,
    }
}

use std::{convert::Infallible, marker::PhantomData};

use serde::Deserialize;

pub(crate) use self::private::SearchResultsObject;
use super::{
    album::{AlbumObject, FullAlbum},
    artist::{ArtistObject, FullArtist},
    page::{Page, PageInformation, PageObject},
    track::{FullTrack, TrackObject},
    ItemType,
};

// TODO: it'd be really cool if this was a const fn or smth
/// The default search types.
pub const DEFAULT_SEARCH_TYPES_STRING: &str = "album,artist,playlist,track,show,episode";
/// The default search limit, i.e. how many items there are in each page.
pub const DEFAULT_SEARCH_LIMIT: u32 = 20;
/// The default search offset.
pub const DEFAULT_SEARCH_OFFSET: u32 = 0;

/// Trait for converting an object to a string used in Spotify's search types. This is currently implemented for all
/// iterators of [ItemType]-enums.
pub trait ToTypesString: crate::private::Sealed {
    fn to_types_string(self) -> String;
}

/// First pages of search results from a [search](crate::client::unscoped::UnscopedClient::search).
#[derive(Debug)]
pub struct SearchResults {
    pub(crate) inner: SearchResultsObject,
}

/// Continuation page of search results from a [search](crate::client::unscoped::UnscopedClient::search) that contains
/// only tracks.
///
/// This object is retrieved only through requesting the [next page](Page::next_page) from an existing page of results.
/// You won't be interacting objects of this type directly.
#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct TrackSearchResults {
    tracks: PageObject<TrackObject>,
}

/// Continuation page of search results from a [search](crate::client::unscoped::UnscopedClient::search) that contains
/// only artists.
///
/// This object is retrieved only through requesting the [next page](Page::next_page) from an existing page of results.
/// You won't be interacting objects of this type directly.
#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct ArtistSearchResults {
    artists: PageObject<ArtistObject>,
}

/// Continuation page of search results from a [search](crate::client::unscoped::UnscopedClient::search) that contains
/// only albums.
///
/// This object is retrieved only through requesting the [next page](Page::next_page) from an existing page of results.
/// You won't be interacting objects of this type directly.
#[derive(Debug, Deserialize)]
#[doc(hidden)]
pub struct AlbumSearchResults {
    albums: PageObject<AlbumObject>,
}

impl TryFrom<SearchResultsObject> for SearchResults {
    type Error = Infallible;

    fn try_from(value: SearchResultsObject) -> Result<Self, Self::Error> {
        Ok(Self { inner: value })
    }
}

impl SearchResults {
    /// Return the tracks in these search results as a [Page] of [FullTracks](FullTrack).
    ///
    /// If no tracks matched the search query, this will return None. Therefore, the returned page will always contain
    /// some items.
    pub fn tracks(self) -> Option<Page<TrackSearchResults, FullTrack>> {
        self.inner.tracks.and_then(|page| {
            if !<PageObject<TrackObject> as PageInformation<FullTrack>>::items(&page).is_empty() {
                Some(Page {
                    inner: TrackSearchResults { tracks: page },
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
    pub fn artists(self) -> Option<Page<ArtistSearchResults, FullArtist>> {
        self.inner.artists.and_then(|page| {
            if !<PageObject<ArtistObject> as PageInformation<FullArtist>>::items(&page).is_empty() {
                Some(Page {
                    inner: ArtistSearchResults { artists: page },
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
    pub fn albums(self) -> Option<Page<AlbumSearchResults, FullAlbum>> {
        self.inner.albums.and_then(|page| {
            if !<PageObject<AlbumObject> as PageInformation<FullAlbum>>::items(&page).is_empty() {
                Some(Page {
                    inner: AlbumSearchResults { albums: page },
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

    fn next(self) -> Option<String> {
        <PageObject<TrackObject> as PageInformation<FullTrack>>::next(self.tracks)
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

    fn next(self) -> Option<String> {
        <PageObject<ArtistObject> as PageInformation<FullArtist>>::next(self.artists)
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

    fn next(self) -> Option<String> {
        <PageObject<AlbumObject> as PageInformation<FullAlbum>>::next(self.albums)
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
