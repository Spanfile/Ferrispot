use super::{private::ClientBase, API_TRACKS_ENDPOINT};
use crate::{
    error::Result,
    model::{
        country_code::CountryCode,
        track::{FullTrack, TrackObject},
    },
};
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Url};
use serde::Deserialize;
use std::fmt::{Display, Write};

/// All unscoped Spotify endpoints.
///
/// The functions in this trait do not require user authentication to use. All Spotify clients implement this trait.
#[async_trait]
pub trait UnscopedClient: ClientBase {
    /// Get Spotify catalog information for a single track identified by its unique Spotify ID.
    ///
    /// An optional market country may be specified. If specified, only content that is available in that market will be
    /// returned. If using an user-authenticated client (i.e.
    /// [AuthorizationCodeUserClient](crate::client::AuthorizationCodeUserClient) or
    /// [ImplicitGrantUserClient](crate::client::ImplicitGrantUserClient)), the country associated with the
    /// corresponding user account will take priority over this parameter.
    async fn track<T>(&self, track_id: T, market: Option<CountryCode>) -> Result<FullTrack>
    where
        T: Display + Send;

    /// Get Spotify catalog information for multiple tracks based on their Spotify IDs.
    ///
    /// Up to 50 IDs may be given. In case some IDs cannot be found, they will be omitted from the result.
    ///
    /// An optional market country may be specified. If specified, only content that is available in that market will be
    /// returned. If using an user-authenticated client (i.e.
    /// [AuthorizationCodeUserClient](crate::client::AuthorizationCodeUserClient) or
    /// [ImplicitGrantUserClient](crate::client::ImplicitGrantUserClient)), the country associated with the
    /// corresponding user account will take priority over this parameter.
    async fn tracks<I, T>(&self, track_ids: I, market: Option<CountryCode>) -> Result<Vec<FullTrack>>
    where
        I: IntoIterator<Item = T> + Send,
        <I as IntoIterator>::IntoIter: Send,
        T: Display + Send;
}

#[async_trait]
impl<C> UnscopedClient for C
where
    C: ClientBase + Sync,
{
    async fn track<T>(&self, track_id: T, market: Option<CountryCode>) -> Result<FullTrack>
    where
        T: Display + Send,
    {
        // TODO: gonna need a way more robust way of constructing the URLs
        let mut url = format!("{}/{}", API_TRACKS_ENDPOINT, track_id);

        if let Some(market) = market {
            // this will fail only if something goes terribly wrong
            write!(&mut url, "?market={}", market).unwrap();
        }

        let request = self.build_http_request(Method::GET, url).await;
        let response = request.send().await?;
        debug!("Track response: {:?}", response);

        // TODO: is this really the way to return an error from an error response?
        response.error_for_status_ref()?;

        let track_object: TrackObject = response.json().await?;
        debug!("Track body: {:#?}", track_object);

        let full_track: FullTrack = track_object.into();
        Ok(full_track)
    }

    async fn tracks<I, T>(&self, track_ids: I, market: Option<CountryCode>) -> Result<Vec<FullTrack>>
    where
        I: IntoIterator<Item = T> + Send,
        <I as IntoIterator>::IntoIter: Send,
        T: Display + Send,
    {
        #[derive(Debug, Deserialize)]
        struct TracksResponse {
            tracks: Vec<Option<TrackObject>>,
        }

        let mut params = vec![(
            "ids",
            track_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(","),
        )];

        if let Some(market) = market {
            params.push(("market", market.to_string()));
        }

        let request = self
            .build_http_request(
                Method::GET,
                Url::parse_with_params(API_TRACKS_ENDPOINT, params)
                    // TODO: can this fail because of malformed IDs?
                    .unwrap(),
            )
            .await;

        let response = request.send().await?;
        debug!("Tracks response: {:?}", response);

        // TODO: is this really the way to return an error from an error response?
        response.error_for_status_ref()?;

        let tracks_object: TracksResponse = response.json().await?;
        debug!("Tracks body: {:#?}", tracks_object);

        let full_tracks: Vec<_> = tracks_object
            .tracks
            .into_iter()
            .filter_map(|obj| obj.map(FullTrack::from))
            .collect();
        Ok(full_tracks)
    }
}
