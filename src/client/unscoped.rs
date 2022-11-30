mod search_builder;

use log::trace;
use reqwest::{Method, StatusCode, Url};
use serde::Deserialize;

pub use self::search_builder::SearchBuilder;
use super::{private, API_TRACKS_ENDPOINT};
use crate::{
    error::{ConversionError, Error, Result},
    model::{
        id::{Id, IdTrait, TrackId},
        track::{FullTrack, TrackObject},
        CountryCode,
    },
};

#[derive(Debug, Deserialize)]
struct TracksResponse {
    tracks: Vec<Option<TrackObject>>,
}

/// All unscoped Spotify endpoints. The functions in this trait do not require user authentication to use. All
/// asynchronous Spotify clients implement this trait.
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait UnscopedAsyncClient<'a>: private::SendHttpRequestAsync<'a> + private::AccessTokenExpiryAsync
where
    Self: Sized,
{
    /// Get Spotify catalog information for a single track identified by its unique Spotify ID.
    ///
    /// An optional market country may be specified. If specified, only content that is available in that market will be
    /// returned and track relinking may be applied. See [the documentation about track
    /// relinking](crate::model::track#track-equality-and-track-relinking) for more information.
    ///
    /// This function's synchronous counterpart is [UnscopedSyncClient::track](UnscopedSyncClient::track).
    async fn track(&'a self, track_id: Id<'a, TrackId>, market: Option<CountryCode>) -> Result<FullTrack> {
        let response = self
            .send_http_request(Method::GET, build_track_url(&track_id, market))
            .send_async()
            .await?
            .error_for_status()
            .map_err(|e| match e.status() {
                Some(StatusCode::NOT_FOUND) => Error::NonexistentTrack(track_id.as_static()),
                _ => e.into(),
            })?;

        trace!("Track response: {:?}", response);

        let track_object: TrackObject = response.json().await?;
        trace!("Track body: {:?}", track_object);

        let full_track: FullTrack = track_object.try_into()?;
        Ok(full_track)
    }

    /// Get Spotify catalog information for multiple tracks based on their Spotify IDs.
    ///
    /// Up to 50 IDs may be given. In case some IDs cannot be found, they will be omitted from the result.
    ///
    /// An optional market country may be specified. If specified, only content that is available in that market will be
    /// returned and track relinking may be applied. See [the documentation about track
    /// relinking](crate::model::track#track-equality-and-track-relinking) for more information.
    ///
    /// This function's synchronous counterpart is [UnscopedSyncClient::tracks](UnscopedSyncClient::tracks).
    async fn tracks<I>(&'a self, track_ids: I, market: Option<CountryCode>) -> Result<Vec<FullTrack>>
    where
        I: IntoIterator<Item = Id<'a, TrackId>> + Send,
        <I as IntoIterator>::IntoIter: Send,
    {
        let response = self
            .send_http_request(Method::GET, build_tracks_url(track_ids, market))
            .send_async()
            .await?
            .error_for_status()
            .map_err(super::response_to_error)?;

        trace!("Tracks response: {:?}", response);

        let tracks_response: TracksResponse = response.json().await?;
        trace!("Tracks body: {:?}", tracks_response);

        Ok(tracks_response.full_tracks()?)
    }

    /// Get Spotify catalog information about albums, artists, playlists, tracks, shows or episodes that match a keyword
    /// string.
    ///
    /// This function returns a [SearchBuilder](self::SearchBuilder) that you can use to configure the various search
    /// parameters and finally send the search query and get the results back.
    ///
    /// This function's synchronous counterpart is [UnscopedSyncClient::search](UnscopedSyncClient::search).
    fn search<S>(&'a self, query: S) -> SearchBuilder<'a, Self, S>
    where
        S: AsRef<str>,
    {
        SearchBuilder::new(self, query)
    }
}

/// All unscoped Spotify endpoints. The functions in this trait do not require user authentication to use. All
/// synchronous Spotify clients implement this trait.
#[cfg(feature = "sync")]
pub trait UnscopedSyncClient<'a>: private::SendHttpRequestSync<'a> + private::AccessTokenExpirySync
where
    Self: Sized,
{
    /// Get Spotify catalog information for a single track identified by its unique Spotify ID.
    ///
    /// An optional market country may be specified. If specified, only content that is available in that market will be
    /// returned and track relinking may be applied. See [the documentation about track
    /// relinking](crate::model::track#track-equality-and-track-relinking) for more information.
    ///
    /// This function's asynchronous counterpart is [UnscopedAsyncClient::track](UnscopedAsyncClient::track).
    fn track(&'a self, track_id: Id<TrackId>, market: Option<CountryCode>) -> Result<FullTrack> {
        let response = self
            .send_http_request(Method::GET, build_track_url(&track_id, market))
            .send_sync()?
            .error_for_status()
            .map_err(|e| match e.status() {
                Some(StatusCode::NOT_FOUND) => Error::NonexistentTrack(track_id.as_static()),
                _ => e.into(),
            })?;

        trace!("Track response: {:?}", response);

        let track_object: TrackObject = response.json()?;
        trace!("Track body: {:?}", track_object);

        let full_track: FullTrack = track_object.try_into()?;
        Ok(full_track)
    }

    /// Get Spotify catalog information for multiple tracks based on their Spotify IDs.
    ///
    /// Up to 50 IDs may be given. In case some IDs cannot be found, they will be omitted from the result.
    ///
    /// An optional market country may be specified. If specified, only content that is available in that market will be
    /// returned and track relinking may be applied. See [the documentation about track
    /// relinking](crate::model::track#track-equality-and-track-relinking) for more information.
    ///
    /// This function's asynchronous counterpart is [UnscopedAsyncClient::tracks](UnscopedAsyncClient::tracks).
    fn tracks<I>(&'a self, track_ids: I, market: Option<CountryCode>) -> Result<Vec<FullTrack>>
    where
        I: IntoIterator<Item = Id<'a, TrackId>>,
    {
        let response = self
            .send_http_request(Method::GET, build_tracks_url(track_ids, market))
            .send_sync()?
            .error_for_status()
            .map_err(super::response_to_error)?;

        trace!("Tracks response: {:?}", response);

        let tracks_response: TracksResponse = response.json()?;
        trace!("Tracks body: {:?}", tracks_response);

        Ok(tracks_response.full_tracks()?)
    }

    /// Get Spotify catalog information about albums, artists, playlists, tracks, shows or episodes that match a keyword
    /// string.
    ///
    /// This function returns a [SearchBuilder](self::SearchBuilder) that you can use to configure the various search
    /// parameters and finally send the search query and get the results back.
    ///
    /// This function's asynchronous counterpart is [UnscopedAsyncClient::search](UnscopedAsyncClient::search).
    fn search<S>(&'a self, query: S) -> SearchBuilder<'a, Self, S>
    where
        S: AsRef<str>,
    {
        SearchBuilder::new(self, query)
    }
}

impl TracksResponse {
    fn full_tracks(self) -> std::result::Result<Vec<FullTrack>, ConversionError> {
        self.tracks
            .into_iter()
            .filter_map(|obj| obj.map(FullTrack::try_from))
            .collect::<std::result::Result<Vec<_>, ConversionError>>()
    }
}

fn build_track_url(track_id: &Id<TrackId>, market: Option<CountryCode>) -> Url {
    // TODO: not really a fan of how the path has to be string formatted and then parsed into an URL
    if let Some(market) = market {
        Url::parse_with_params(
            &format!("{}/{}", API_TRACKS_ENDPOINT, track_id.id()),
            &[("market", market.to_string())],
        )
    } else {
        Url::parse(&format!("{}/{}", API_TRACKS_ENDPOINT, track_id.id()))
    }
    .expect("failed to build API track endpoint URL (this is likely a bug in the library)")
}

fn build_tracks_url<'a, I>(tracks: I, market: Option<CountryCode>) -> Url
where
    I: IntoIterator<Item = Id<'a, TrackId>>,
{
    let mut params = vec![(
        "ids",
        tracks
            .into_iter()
            .map(|id| id.id().to_owned())
            .collect::<Vec<_>>()
            .join(","),
    )];

    if let Some(market) = market {
        params.push(("market", market.to_string()));
    }

    Url::parse_with_params(API_TRACKS_ENDPOINT, params)
        .expect("failed to parse API tracks endpoint as URL: invalid base URL (this is likely a bug)")
}
