use super::{private, API_CURRENTLY_PLAYING_TRACK_ENDPOINT, API_PLAYBACK_STATE_ENDPOINT};
use crate::{
    client::{
        API_PLAYER_DEVICES_ENDPOINT, API_PLAYER_PAUSE_ENDPOINT, API_PLAYER_PLAY_ENDPOINT, API_PLAYER_QUEUE_ENDPOINT,
        API_PLAYER_REPEAT_ENDPOINT, API_PLAYER_SHUFFLE_ENDPOINT, API_PLAYER_VOLUME_ENDPOINT,
    },
    error::{Error, Result},
    model::{
        error::{ApiErrorMessage, ApiErrorResponse},
        id::{IdTrait, PlayableContext, PlayableItem},
        playback::{CurrentlyPlayingTrack, Device, PlaybackState, RepeatState},
    },
};
use async_trait::async_trait;
use log::{debug, error, warn};
use reqwest::{Method, Response, StatusCode, Url};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// All scoped Spotify endpoints. The functions in this trait require user authentication, since they're specific to a
/// certain user. [AuthorizationCodeUserClient](crate::client::authorization_code::AuthorizationCodeUserClient) and
/// [ImplicitGrantUserClient](crate::client::implicit_grant::ImplicitGrantUserClient) implement this trait.
#[async_trait]
pub trait ScopedClient<'a>: private::SendHttpRequest<'a> + private::AccessTokenExpiry {
    /// Get information about the user’s current playback state, including track or episode, progress, and active
    /// device.
    ///
    /// This returns a superset of the [currently playing track](Self::currently_playing_track).
    ///
    /// Required scope: [UserReadPlaybackState](crate::scope::Scope::UserReadPlaybackState).
    async fn playback_state(&'a self) -> Result<Option<PlaybackState>> {
        let response = self
            .send_http_request(
                Method::GET,
                Url::parse(API_PLAYBACK_STATE_ENDPOINT)
                    .expect("failed to build playback state endpoint URL: invalid base URL (this is likely a bug)"),
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
    ///
    /// Required scope: [UserReadCurrentlyPlaying](crate::scope::Scope::UserReadCurrentlyPlaying).
    async fn currently_playing_track(&'a self) -> Result<Option<CurrentlyPlayingTrack>> {
        let response = self
            .send_http_request(
                Method::GET,
                Url::parse(API_CURRENTLY_PLAYING_TRACK_ENDPOINT).expect(
                    "failed to build currently playing track endpoint URL: invalid base URL (this is likely a bug)",
                ),
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

    /// Start playing a collection of playable items in order; tracks or episodes.
    ///
    /// If `device_id` is supplied, playback will be targeted on that device. If not supplied, playback will be targeted
    /// on the user's currently active device. In case no device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    async fn play_items<I, P>(&'a self, items: I, device_id: Option<&str>) -> Result<()>
    where
        I: IntoIterator<Item = P> + Send,
        <I as IntoIterator>::IntoIter: Send,
        P: Into<PlayableItem<'a>>,
    {
        #[derive(Debug, Serialize)]
        struct Body<'a> {
            uris: Vec<Cow<'a, str>>,
        }

        let url = build_play_url(API_PLAYER_PLAY_ENDPOINT, &[("device_id", device_id)]);

        // first gather all the IDs into a vector
        let tracks: Vec<_> = items.into_iter().map(|id| id.into()).collect();

        // then create the body which borrows the IDs. this method has to allocate two vectors but whatcha gonna do.
        // TODO
        let body = Body {
            uris: tracks.iter().map(|id| id.uri()).collect(),
        };

        debug!("Play body: {:#?}", body);

        let response = self.send_http_request(Method::PUT, url).body(body).send().await?;
        debug!("Play response: {:?}", response);

        handle_player_control_response(response).await
    }

    // TODO: offset
    /// Start playing a context; album, artist, playlist or show.
    ///
    /// If `device_id` is supplied, playback will be targeted on that device. If not supplied, playback will be targeted
    /// on the user's currently active device.
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    async fn play_context(&'a self, context: PlayableContext<'a>, device_id: Option<&str>) -> Result<()> {
        #[derive(Debug, Serialize)]
        struct Body<'a> {
            context_uri: Cow<'a, str>,
        }

        let url = build_play_url(API_PLAYER_PLAY_ENDPOINT, &[("device_id", device_id)]);

        let body = Body {
            context_uri: context.uri(),
        };

        debug!("Play body: {:#?}", body);

        let response = self.send_http_request(Method::PUT, url).body(body).send().await?;
        debug!("Play response: {:?}", response);

        handle_player_control_response(response).await
    }

    /// Resume current playback.
    ///
    /// If `device_id` is supplied, playback will be targeted on that device. If not supplied, playback will be targeted
    /// on the user's currently active device.
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    async fn resume(&'a self, device_id: Option<&str>) -> Result<()> {
        let url = build_play_url(API_PLAYER_PLAY_ENDPOINT, &[("device_id", device_id)]);
        let response = self.send_http_request(Method::PUT, url).send().await?;
        debug!("Resume response: {:?}", response);

        handle_player_control_response(response).await
    }

    /// Pause current playback.
    ///
    /// If `device_id` is supplied, playback will be targeted on that device. If not supplied, playback will be targeted
    /// on the user's currently active device.
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    async fn pause(&'a self, device_id: Option<&str>) -> Result<()> {
        let url = build_play_url(API_PLAYER_PAUSE_ENDPOINT, &[("device_id", device_id)]);
        let response = self.send_http_request(Method::PUT, url).send().await?;
        debug!("Pause response: {:?}", response);

        handle_player_control_response(response).await
    }

    /// Set the repeat state for the current playback.
    ///
    /// If `device_id` is supplied, playback will be targeted on that device. If not supplied, playback will be targeted
    /// on the user's currently active device.
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    async fn repeat_state(&'a self, repeat_state: RepeatState, device_id: Option<&str>) -> Result<()> {
        let url = build_play_url(
            API_PLAYER_REPEAT_ENDPOINT,
            &[("repeat_state", Some(repeat_state.as_str())), ("device_id", device_id)],
        );

        let response = self.send_http_request(Method::PUT, url).send().await?;
        debug!("Set repeat state response: {:?}", response);

        handle_player_control_response(response).await
    }

    /// Set the shuffle mode for the current playback.
    ///
    /// If `device_id` is supplied, playback will be targeted on that device. If not supplied, playback will be targeted
    /// on the user's currently active device.
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    async fn shuffle(&'a self, shuffle: bool, device_id: Option<&str>) -> Result<()> {
        let url = build_play_url(
            API_PLAYER_SHUFFLE_ENDPOINT,
            &[
                ("shuffle", Some(if shuffle { "true" } else { "false" })),
                ("device_id", device_id),
            ],
        );

        let response = self.send_http_request(Method::PUT, url).send().await?;
        debug!("Set shuffle response: {:?}", response);

        handle_player_control_response(response).await
    }

    /// Set the volume for the current playback. `volume_percent` is an integer between 0 and 100 inclusive.
    ///
    /// If `device_id` is supplied, playback will be targeted on that device. If not supplied, playback will be targeted
    /// on the user's currently active device.
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    async fn volume<U>(&'a self, volume_percent: U, device_id: Option<&str>) -> Result<()>
    where
        U: Into<u8> + Send,
    {
        let volume_percent = volume_percent.into().to_string();
        let url = build_play_url(
            API_PLAYER_VOLUME_ENDPOINT,
            &[("volume_percent", Some(&volume_percent)), ("device_id", device_id)],
        );

        let response = self.send_http_request(Method::PUT, url).send().await?;
        debug!("Set volume response: {:?}", response);

        handle_player_control_response(response).await
    }

    /// Add a playable item to the end of the current playback queue.
    ///
    /// If `device_id` is supplied, playback will be targeted on that device. If not supplied, playback will be targeted
    /// on the user's currently active device.
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    async fn add_to_queue(&'a self, item: PlayableItem<'a>, device_id: Option<&str>) -> Result<()> {
        let uri = item.uri();
        let url = build_play_url(
            API_PLAYER_QUEUE_ENDPOINT,
            &[("uri", Some(&uri)), ("device_id", device_id)],
        );

        let response = self.send_http_request(Method::POST, url).send().await?;
        debug!("Add to queue response: {:?}", response);

        handle_player_control_response(response).await
    }

    /// Get information about the user's available devices.
    ///
    /// Required scope: [UserReadPlaybackState](crate::scope::Scope::UserReadPlaybackState).
    async fn devices(&'a self) -> Result<Vec<Device>> {
        #[derive(Debug, Deserialize)]
        struct DevicesResponse {
            devices: Vec<Device>,
        }

        let url = build_play_url(API_PLAYER_DEVICES_ENDPOINT, &[]);

        let response = self.send_http_request(Method::GET, url).send().await?;
        debug!("Devices response: {:?}", response);

        // TODO: is this really the way to return an error from an error response?
        response.error_for_status_ref()?;

        let devices_response: DevicesResponse = response.json().await?;
        debug!("Devices: {:?}", devices_response);

        Ok(devices_response.devices)
    }
}

fn build_play_url(endpoint: &str, params: &[(&'static str, Option<&str>)]) -> Url {
    let params: Vec<_> = params
        .iter()
        .filter_map(|(key, value)| value.map(|value| (key, value)))
        .collect();

    // this will fail only if the endpoint is an invalid URL, which would mean a bug in the library
    Url::parse_with_params(endpoint, &params)
        .expect("failed to build player URL: invalid base URL (this is likely a bug)")
}

async fn handle_player_control_response(response: Response) -> Result<()> {
    match response.status() {
        StatusCode::NO_CONTENT => Ok(()),

        StatusCode::NOT_FOUND => {
            warn!("Got 404 Not Found to play call");
            let error_response: ApiErrorResponse = response.json().await?;

            match error_response.error.message {
                ApiErrorMessage::NoActiveDevice => {
                    warn!("Player call failed: no active device");
                    Err(Error::NoActiveDevice)
                }

                other => {
                    error!("Unexpected Spotify error response to player call: {:?}", other);
                    Err(Error::UnhandledSpotifyError(401, format!("{:?}", other)))
                }
            }
        }

        other => {
            warn!("Got unexpected response status to player call: {}", other);
            Ok(())
        }
    }
}
