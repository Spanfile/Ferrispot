use super::{
    private::{ClientBase, UserAuthenticatedClient},
    API_CURRENTLY_PLAYING_TRACK_ENDPOINT, API_PLAYBACK_STATE_ENDPOINT,
};
use crate::{
    error::Result,
    model::playback::{CurrentlyPlayingTrack, PlaybackState},
};
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, StatusCode};

/// All scoped Spotify endpoints.
///
/// The functions in this trait require user authentication, since they're specific to a certain user.
/// [AuthorizationCodeUserClient](crate::client::AuthorizationCodeUserClient) and
/// [ImplicitGrantUserClient](crate::client::ImplicitGrantUserClient) implement this trait.
#[async_trait]
pub trait ScopedClient: ClientBase + UserAuthenticatedClient {
    async fn playback_state(&self) -> Result<Option<PlaybackState>>;
    async fn currently_playing_track(&self) -> Result<Option<CurrentlyPlayingTrack>>;
}

#[async_trait]
impl<T> ScopedClient for T
where
    T: ClientBase + UserAuthenticatedClient + Sync,
{
    async fn playback_state(&self) -> Result<Option<PlaybackState>> {
        let request = self.build_http_request(Method::GET, API_PLAYBACK_STATE_ENDPOINT).await;
        let response = request.send().await?;
        debug!("Playback state response: {:?}", response);

        // TODO: is this really the way to return an error from an error response?
        response.error_for_status_ref()?;

        if response.status() == StatusCode::NO_CONTENT {
            return Ok(None);
        }

        let playback_state = response.json().await?;
        debug!("Playback state body: {:#?}", playback_state);

        Ok(Some(playback_state))
    }

    async fn currently_playing_track(&self) -> Result<Option<CurrentlyPlayingTrack>> {
        let request = self
            .build_http_request(Method::GET, API_CURRENTLY_PLAYING_TRACK_ENDPOINT)
            .await;

        let response = request.send().await?;
        debug!("Currently playing track response: {:?}", response);

        // TODO: is this really the way to return an error from an error response?
        response.error_for_status_ref()?;

        if response.status() == StatusCode::NO_CONTENT {
            return Ok(None);
        }

        let currently_playing_trtack = response.json().await?;
        debug!("Currently playing track body: {:#?}", currently_playing_trtack);

        Ok(Some(currently_playing_trtack))
    }
}
