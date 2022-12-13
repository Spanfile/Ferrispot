use log::warn;
use reqwest::{Method, StatusCode};

#[cfg(feature = "async")]
use crate::client::request_builder::AsyncResponseHandler;
#[cfg(feature = "sync")]
use crate::client::request_builder::SyncResponseHandler;
use crate::{
    client::{
        object,
        request_builder::{BaseRequestBuilderContainer, CatalogItemRequestBuilder, SearchBuilder},
        API_SEARCH_ENDPOINT, API_TRACKS_ENDPOINT,
    },
    error::Error,
    model::{
        id::{Id, IdTrait, TrackId},
        track::FullTrack,
    },
};

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
    fn tracks<'a, I>(&'a self, tracks: I) -> CatalogItemRequestBuilder<Self, object::TracksResponse, Vec<FullTrack>>
    where
        I: IntoIterator<Item = Id<'a, TrackId>>,
    {
        CatalogItemRequestBuilder::new(Method::GET, API_TRACKS_ENDPOINT, self.clone()).append_query(
            object::TRACKS_IDS_QUERY,
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
        SearchBuilder::new(Method::GET, API_SEARCH_ENDPOINT, self.clone()).query(query.into())
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
