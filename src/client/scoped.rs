use super::{private, API_CURRENTLY_PLAYING_TRACK_ENDPOINT, API_PLAYBACK_STATE_ENDPOINT};
use crate::{
    client::API_PLAYER_PLAY_ENDPOINT,
    error::Result,
    model::{
        id::Id,
        playback::{CurrentlyPlayingTrack, Play, PlaybackState},
    },
};
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, StatusCode, Url};
use serde::Serialize;

/// All scoped Spotify endpoints.
///
/// The functions in this trait require user authentication, since they're specific to a certain user.
/// [AuthorizationCodeUserClient](crate::client::AuthorizationCodeUserClient) and
/// [ImplicitGrantUserClient](crate::client::ImplicitGrantUserClient) implement this trait.
#[async_trait]
pub trait ScopedClient<'a>:
    private::SendHttpRequest<'a> + private::AccessTokenExpiry + private::UserAuthenticatedClient
{
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

    async fn play(&'a self, play: Play<'_>, device_id: Option<&str>) -> Result<()> {
        #[derive(Debug, Serialize)]
        struct Body {
            context_uri: Option<String>,
            uris: Option<Vec<String>>,
        }

        let url = if let Some(device_id) = device_id {
            Url::parse_with_params(API_PLAYER_PLAY_ENDPOINT, [("device_id", device_id)])
                .expect("failed to build player play endpoint URL")
        } else {
            Url::parse(API_PLAYER_PLAY_ENDPOINT).expect("failed to build player play endpoint URL")
        };

        let body = match play {
            Play::Context(context) => Body {
                context_uri: Some(context.format_uri()),
                uris: None,
            },

            Play::Items(items) => Body {
                context_uri: None,
                uris: Some(items.0.map(|id| id.format_uri()).collect()),
            },
        };

        let response = self.send_http_request(Method::PUT, url).body(body).send().await?;
        debug!("Play response: {:?}", response);

        // TODO: is this really the way to return an error from an error response?
        response.error_for_status_ref()?;

        Ok(())
    }
}

#[async_trait]
impl<'a, C> ScopedClient<'a> for C where C: private::SendHttpRequest<'a> + private::UserAuthenticatedClient + Sync {}
