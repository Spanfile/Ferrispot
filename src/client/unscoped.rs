mod search_builder;

use std::borrow::Cow;

use log::warn;
use reqwest::{Method, StatusCode};

use self::private::TracksResponse;
pub use self::search_builder::SearchBuilder;
use super::{request_builder::RequestBuilder, API_TRACKS_ENDPOINT};
#[cfg(feature = "async")]
use crate::client::request_builder::AsyncResponseHandler;
#[cfg(feature = "sync")]
use crate::client::request_builder::SyncResponseHandler;
use crate::{
    client::request_builder::BaseRequestBuilderContainer,
    error::Error,
    model::{
        id::{Id, IdTrait, TrackId},
        track::FullTrack,
        CountryCode,
    },
};

mod private {
    use serde::Deserialize;

    use crate::{
        client::request_builder::TryFromEmptyResponse,
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

    impl TryFrom<TracksResponse> for Vec<FullTrack> {
        type Error = ConversionError;

        fn try_from(value: TracksResponse) -> Result<Self, Self::Error> {
            value
                .tracks
                .into_iter()
                .filter_map(|obj| obj.map(FullTrack::try_from))
                .collect::<std::result::Result<Vec<_>, ConversionError>>()
        }
    }

    impl TryFromEmptyResponse for TracksResponse {}
    impl TryFromEmptyResponse for FullTrack {}
    impl TryFromEmptyResponse for TrackObject {}
    impl TryFromEmptyResponse for Vec<FullTrack> {}
}

const TRACKS_IDS_QUERY: &str = "ids";
const MARKET_QUERY: &str = "market";

pub struct CatalogItemRequestBuilder<TClient, TResponse, TReturn = TResponse>(
    RequestBuilder<TClient, TResponse, (), TReturn>,
);

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
    fn track<'a>(&'a self, track: Id<'a, TrackId>) -> CatalogItemRequestBuilder<Self, FullTrack> {
        let mut builder = CatalogItemRequestBuilder::new(
            Method::GET,
            format!("{}/{}", API_TRACKS_ENDPOINT, track.as_str()),
            self.clone(),
        );

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(track_response_handler_async_fn(track.as_owned()));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(track_response_handler_sync_fn(track.as_owned()));
        }

        builder
    }

    /// Get Spotify catalog information for multiple tracks based on their Spotify IDs.
    ///
    /// Up to 50 IDs may be given. In case some IDs cannot be found, they will be omitted from the result.
    ///
    /// An optional market country may be specified with the [`market`-function in the request builder this function
    /// returns](CatalogItemRequestBuilder::market). Only content that is available in that market will be returned and
    /// [track relinking](crate::model::track#track-equality-and-track-relinking) may be applied.
    fn tracks<'a, I>(&'a self, tracks: I) -> CatalogItemRequestBuilder<Self, TracksResponse, Vec<FullTrack>>
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
    fn search<S>(&self, query: S) -> SearchBuilder<Self>
    where
        S: Into<String>,
    {
        SearchBuilder::new(self.clone(), query.into())
    }
}

impl<TClient, TResponse, TReturn> BaseRequestBuilderContainer<TClient, TResponse, (), TReturn>
    for CatalogItemRequestBuilder<TClient, TResponse, TReturn>
{
    fn new<S>(method: Method, base_url: S, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new(method, base_url, client))
    }

    fn new_with_body<S>(method: Method, base_url: S, body: (), client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new_with_body(method, base_url, body, client))
    }

    fn take_base_builder(self) -> RequestBuilder<TClient, TResponse, (), TReturn> {
        self.0
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TClient, TResponse, (), TReturn> {
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

#[cfg(feature = "async")]
fn track_response_handler_async_fn(track_id: Id<'static, TrackId>) -> AsyncResponseHandler {
    Box::new(move |response| {
        Box::pin(async move {
            match response.status() {
                StatusCode::OK => Ok(response),

                StatusCode::NOT_FOUND => {
                    warn!("Got 404 Not Found to track call");
                    Err(Error::NonexistentTrack(track_id))
                }

                other => Err(Error::UnhandledSpotifyResponseStatusCode(other.as_u16())),
            }
        })
    })
}

#[cfg(feature = "sync")]
fn track_response_handler_sync_fn(track_id: Id<'static, TrackId>) -> SyncResponseHandler {
    Box::new(move |response| match response.status() {
        StatusCode::OK => Ok(response),

        StatusCode::NOT_FOUND => {
            warn!("Got 404 Not Found to track call");
            Err(Error::NonexistentTrack(track_id))
        }

        other => Err(Error::UnhandledSpotifyResponseStatusCode(other.as_u16())),
    })
}
