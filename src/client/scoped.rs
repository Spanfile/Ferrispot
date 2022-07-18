use super::{private, API_CURRENTLY_PLAYING_TRACK_ENDPOINT, API_PLAYBACK_STATE_ENDPOINT};
use crate::{
    client::API_PLAYER_PLAY_ENDPOINT,
    error::Result,
    model::{
        id::{IdTrait, PlayableContext, PlayableItem},
        playback::{CurrentlyPlayingTrack, PlaybackState},
    },
};
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, StatusCode, Url};
use serde::Serialize;
use std::borrow::Cow;

/// All scoped Spotify endpoints.
///
/// The functions in this trait require user authentication, since they're specific to a certain user.
/// [AuthorizationCodeUserClient](crate::client::AuthorizationCodeUserClient) and
/// [ImplicitGrantUserClient](crate::client::ImplicitGrantUserClient) implement this trait.
#[async_trait]
pub trait ScopedClient<'a>:
    private::SendHttpRequest<'a> + private::AccessTokenExpiry + private::UserAuthenticatedClient
{
    /// Get information about the user’s current playback state, including track or episode, progress, and active
    /// device.
    ///
    /// This returns a superset of the [currently playing track](Self::currently_playing_track).
    async fn playback_state(&'a self) -> Result<Option<PlaybackState>> {
        let response = self
            .send_http_request(
                Method::GET,
                Url::parse(API_PLAYBACK_STATE_ENDPOINT).expect("failed to build playback state endpoint URL"),
            )
            .send()
            .await?;
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

    /// Get the object currently being played on the user's Spotify account.
    async fn currently_playing_track(&'a self) -> Result<Option<CurrentlyPlayingTrack>> {
        let response = self
            .send_http_request(
                Method::GET,
                Url::parse(API_CURRENTLY_PLAYING_TRACK_ENDPOINT)
                    .expect("failed to build currently playing track endpoint URL"),
            )
            .send()
            .await?;

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

    async fn play_items<I, P>(&'a self, tracks: I, device_id: Option<&str>) -> Result<()>
    where
        I: IntoIterator<Item = P> + Send,
        <I as IntoIterator>::IntoIter: Send,
        P: Into<PlayableItem<'a>>,
    {
        #[derive(Debug, Serialize)]
        struct Body<'a> {
            uris: Vec<Cow<'a, str>>,
        }

        let url = if let Some(device_id) = device_id {
            Url::parse_with_params(API_PLAYER_PLAY_ENDPOINT, [("device_id", device_id)])
                .expect("failed to build player play endpoint URL")
        } else {
            Url::parse(API_PLAYER_PLAY_ENDPOINT).expect("failed to build player play endpoint URL")
        };

        // first gather all the IDs into a vector
        let tracks: Vec<_> = tracks.into_iter().map(|id| id.into()).collect();

        // then create the body which borrows the IDs. this method has to allocate two vectors but whatcha gonna do.
        // TODO
        let body = Body {
            uris: tracks.iter().map(|id| id.uri()).collect(),
        };

        debug!("Play body: {:#?}", body);

        let response = self.send_http_request(Method::PUT, url).body(body).send().await?;
        debug!("Play response: {:?}", response);

        // TODO: is this really the way to return an error from an error response?
        response.error_for_status_ref()?;

        Ok(())
    }

    async fn play_context(&'a self, context: PlayableContext<'a>, device_id: Option<&str>) -> Result<()> {
        todo!()
    }
}

#[async_trait]
impl<'a, C> ScopedClient<'a> for C where C: private::SendHttpRequest<'a> + private::UserAuthenticatedClient + Sync {}
