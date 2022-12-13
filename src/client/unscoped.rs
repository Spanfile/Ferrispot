mod search_builder;

use std::borrow::Cow;

use reqwest::Method;

pub use self::search_builder::SearchBuilder;
use super::{request_builder::RequestBuilder, API_TRACKS_ENDPOINT};
use crate::{
    client::request_builder::private::BaseRequestBuilderContainer,
    model::{
        id::{Id, IdTrait, TrackId},
        track::FullTrack,
        CountryCode,
    },
};

mod private {
    use serde::Deserialize;

    use crate::{
        error::ConversionError,
        model::track::{FullTrack, TrackObject},
    };

    #[derive(Debug, Deserialize)]
    pub struct TracksResponse {
        tracks: Vec<Option<TrackObject>>,
    }

    impl TracksResponse {
        pub fn full_tracks(self) -> std::result::Result<Vec<FullTrack>, ConversionError> {
            self.tracks
                .into_iter()
                .filter_map(|obj| obj.map(FullTrack::try_from))
                .collect::<std::result::Result<Vec<_>, ConversionError>>()
        }
    }
}

const TRACKS_IDS_QUERY: &str = "ids";
const MARKET_QUERY: &str = "market";

pub struct CatalogItemRequestBuilder<TReturn, C>(RequestBuilder<TReturn, C, ()>);

/// All unscoped Spotify endpoints. The functions in this trait do not require user authentication to use. All Spotify
/// clients implement this trait.
pub trait UnscopedClient
where
    Self: Clone + Sized,
{
    /// Get Spotify catalog information for a single track identified by its unique Spotify ID.
    ///
    /// An optional market country may be specified with the [`market`-function in the request builder this function
    /// returns](CatalogItemRequestBuilder::market). Only content that is available in that market will be returned and
    /// [track relinking](crate::model::track#track-equality-and-track-relinking) may be applied.
    fn track<'a>(&'a self, track: Id<'a, TrackId>) -> CatalogItemRequestBuilder<FullTrack, Self> {
        // TODO: 404 to NonexistentTrack
        CatalogItemRequestBuilder::new(
            Method::GET,
            format!("{}/{}", API_TRACKS_ENDPOINT, track.as_str()),
            self.clone(),
        )
    }

    /// Get Spotify catalog information for multiple tracks based on their Spotify IDs.
    ///
    /// Up to 50 IDs may be given. In case some IDs cannot be found, they will be omitted from the result.
    ///
    /// An optional market country may be specified with the [`market`-function in the request builder this function
    /// returns](CatalogItemRequestBuilder::market). Only content that is available in that market will be returned and
    /// [track relinking](crate::model::track#track-equality-and-track-relinking) may be applied.
    fn tracks<'a, I>(&'a self, tracks: I) -> CatalogItemRequestBuilder<Vec<FullTrack>, Self>
    where
        I: IntoIterator<Item = Id<'a, TrackId>>,
    {
        CatalogItemRequestBuilder::new(Method::GET, API_TRACKS_ENDPOINT, self.clone()).append_query(
            TRACKS_IDS_QUERY,
            tracks
                .into_iter()
                .map(|id| id.as_str().to_owned())
                .collect::<Vec<_>>()
                .join(","),
        )
    }

    /// Get Spotify catalog information about albums, artists, playlists, tracks, shows or episodes that match a keyword
    /// string.
    ///
    /// This function returns a [SearchBuilder](self::SearchBuilder) that you can use to configure the various search
    /// parameters and finally send the search query and get the results back.
    fn search<S>(&self, query: S) -> SearchBuilder<'_, Self, S>
    where
        S: AsRef<str>,
    {
        SearchBuilder::new(self, query)
    }
}

impl<TReturn, C> BaseRequestBuilderContainer<TReturn, C, ()> for CatalogItemRequestBuilder<TReturn, C> {
    fn new<S>(method: Method, base_url: S, client: C) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new(method, base_url, client))
    }

    fn new_with_body<S>(method: Method, base_url: S, body: (), client: C) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new_with_body(method, base_url, body, client))
    }

    fn take_base_builder(self) -> RequestBuilder<TReturn, C, ()> {
        self.0
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TReturn, C, ()> {
        &mut self.0
    }
}

impl<TReturn, C> CatalogItemRequestBuilder<TReturn, C> {
    /// Specify a target market country for this request. Only content that is available in that market will be returned
    /// and [track relinking](crate::model::track#track-equality-and-track-relinking) may be applied.
    pub fn market(self, market: CountryCode) -> Self {
        self.append_query(MARKET_QUERY, market.to_string())
    }
}
