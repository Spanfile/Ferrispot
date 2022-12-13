use std::borrow::Cow;
#[cfg(feature = "async")]
use std::{future::Future, pin::Pin};

use log::{error, trace, warn};
use reqwest::{Method, StatusCode};

use super::request_builder::{private::BaseRequestBuilderContainer, RequestBuilder};
use crate::{
    client::{
        API_CURRENTLY_PLAYING_ITEM_ENDPOINT, API_PLAYBACK_STATE_ENDPOINT, API_PLAYER_DEVICES_ENDPOINT,
        API_PLAYER_NEXT_ENDPOINT, API_PLAYER_PAUSE_ENDPOINT, API_PLAYER_PLAY_ENDPOINT, API_PLAYER_PREVIOUS_ENDPOINT,
        API_PLAYER_QUEUE_ENDPOINT, API_PLAYER_REPEAT_ENDPOINT, API_PLAYER_SEEK_ENDPOINT, API_PLAYER_SHUFFLE_ENDPOINT,
        API_PLAYER_VOLUME_ENDPOINT,
    },
    error::{Error, Result},
    model::{
        error::{ApiErrorMessage, ApiErrorResponse},
        id::{IdTrait, PlayableContext, PlayableItem},
        playback::{CurrentlyPlayingItem, Device, PlaybackState, RepeatState},
    },
};

mod private {
    use serde::{Deserialize, Serialize};

    use crate::model::playback::Device;

    #[derive(Debug, Serialize)]
    pub struct PlayItemsBody {
        pub uris: Vec<String>,
    }

    #[derive(Debug, Serialize)]
    pub struct PlayContextBody {
        pub context_uri: String,
    }

    // TODO
    #[derive(Debug, Deserialize)]
    pub struct DevicesResponse {
        pub devices: Vec<Device>,
    }
}

const DEVICE_ID_QUERY: &str = "device_id";
const REPEAT_STATE_QUERY: &str = "repeat_state";
const SHUFFLE_QUERY: &str = "shuffle";
const VOLUME_PERCENT_QUERY: &str = "volume_percent";
const SEEK_POSITION_QUERY: &str = "position_ms";
const QUEUE_URI_QUERY: &str = "uri";

pub struct BasePlayerControlRequestBuilder<C, TBody>(RequestBuilder<(), C, TBody>);

pub type PlayItemsRequestBuilder<C> = BasePlayerControlRequestBuilder<C, private::PlayItemsBody>;
pub type PlayContextRequestBuilder<C> = BasePlayerControlRequestBuilder<C, private::PlayContextBody>;
pub type PlayerControlRequestBuilder<C> = BasePlayerControlRequestBuilder<C, ()>;

/// All scoped Spotify endpoints. The functions in this trait require user authentication, since they're specific to a
/// certain user. The clients
/// [AuthorizationCodeUserClient](crate::client::authorization_code::AuthorizationCodeUserClient) and
/// [ImplicitGrantUserClient](crate::client::implicit_grant::ImplicitGrantUserClient) implement this trait.
pub trait ScopedClient
where
    Self: Clone + Sized,
{
    /// Get information about the user’s current playback state, including track or episode, progress, and active
    /// device.
    ///
    /// This function returns a superset of the [currently playing item](Self::currently_playing_item).
    ///
    /// Required scope: [UserReadPlaybackState](crate::scope::Scope::UserReadPlaybackState).
    fn playback_state(&self) -> RequestBuilder<Option<PlaybackState>, Self> {
        RequestBuilder::new(Method::GET, API_PLAYBACK_STATE_ENDPOINT, self.clone())
    }

    /// Get the item currently being played on the user's Spotify account.
    ///
    /// Required scope: [UserReadCurrentlyPlaying](crate::scope::Scope::UserReadCurrentlyPlaying).
    fn currently_playing_item(&self) -> RequestBuilder<Option<CurrentlyPlayingItem>, Self> {
        RequestBuilder::new(Method::GET, API_CURRENTLY_PLAYING_ITEM_ENDPOINT, self.clone())
    }

    /// Get information about the user's available devices.
    ///
    /// Required scope: [UserReadPlaybackState](crate::scope::Scope::UserReadPlaybackState).
    fn devices(&self) -> RequestBuilder<Vec<Device>, Self> {
        RequestBuilder::new(Method::GET, API_PLAYER_DEVICES_ENDPOINT, self.clone())
    }

    /// Start playing a collection of playable items in order; tracks or episodes.
    ///
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn play_items<'a, I, P>(&'a self, items: I) -> PlayItemsRequestBuilder<Self>
    where
        I: IntoIterator<Item = P>,
        P: Into<PlayableItem<'a>>,
    {
        let tracks: Vec<_> = items.into_iter().map(|id| id.into()).collect();
        let body = private::PlayItemsBody {
            uris: tracks.iter().map(|id| id.as_uri().to_string()).collect(),
        };

        trace!("Play body: {:?}", body);
        let mut builder =
            PlayItemsRequestBuilder::new_with_body(Method::POST, API_PLAYER_PLAY_ENDPOINT, body, self.clone());

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }

    // TODO: offset
    /// Start playing a context; album, artist, playlist or show.
    ///
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will return an
    /// [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn play_context<'a>(&'a self, context: PlayableContext<'a>) -> PlayContextRequestBuilder<Self> {
        let body = private::PlayContextBody {
            context_uri: context.as_uri().to_string(),
        };

        trace!("Play body: {:?}", body);
        let mut builder =
            PlayContextRequestBuilder::new_with_body(Method::POST, API_PLAYER_PLAY_ENDPOINT, body, self.clone());

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }

    /// Resume current playback.
    ///
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn resume(&self) -> PlayerControlRequestBuilder<Self> {
        let mut builder = PlayerControlRequestBuilder::new(Method::PUT, API_PLAYER_PLAY_ENDPOINT, self.clone());

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }

    /// Pause current playback.
    ///
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn pause(&self) -> PlayerControlRequestBuilder<Self> {
        let mut builder = PlayerControlRequestBuilder::new(Method::PUT, API_PLAYER_PAUSE_ENDPOINT, self.clone());

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }

    /// Set the repeat state for the current playback.
    ///
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn repeat_state(&self, repeat_state: RepeatState) -> PlayerControlRequestBuilder<Self> {
        let mut builder = PlayerControlRequestBuilder::new(Method::PUT, API_PLAYER_REPEAT_ENDPOINT, self.clone())
            .append_query(REPEAT_STATE_QUERY, repeat_state.as_str());

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }

    /// Set the shuffle mode for the current playback.
    ///
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn shuffle(&self, shuffle: bool) -> PlayerControlRequestBuilder<Self> {
        let mut builder = PlayerControlRequestBuilder::new(Method::PUT, API_PLAYER_SHUFFLE_ENDPOINT, self.clone())
            .append_query(SHUFFLE_QUERY, if shuffle { "true" } else { "false" });

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }

    /// Set the volume for the current playback. `volume_percent` is an integer between 0 and 100 inclusive.
    ///
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn volume<U>(&self, volume_percent: U) -> PlayerControlRequestBuilder<Self>
    where
        U: Into<u8>,
    {
        let volume_percent = volume_percent.into().to_string();
        let mut builder = PlayerControlRequestBuilder::new(Method::PUT, API_PLAYER_VOLUME_ENDPOINT, self.clone())
            .append_query(VOLUME_PERCENT_QUERY, volume_percent);

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }

    /// Skip to the next track in the user's queue.
    ///
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn next(&self) -> PlayerControlRequestBuilder<Self> {
        let mut builder = PlayerControlRequestBuilder::new(Method::POST, API_PLAYER_NEXT_ENDPOINT, self.clone());

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }

    /// Skip to the next track in the user's queue.
    ///
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn previous(&self) -> PlayerControlRequestBuilder<Self> {
        let mut builder = PlayerControlRequestBuilder::new(Method::POST, API_PLAYER_PREVIOUS_ENDPOINT, self.clone());

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }

    /// Seeks to the given position in the user’s currently playing track. `position` is the position in milliseconds to
    /// seek to. Passing in a position that is greater than the length of the track will cause the player to start
    /// playing the next song.
    //
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn seek<U>(&self, position: U) -> PlayerControlRequestBuilder<Self>
    where
        U: Into<u64>,
    {
        let position = position.into().to_string();
        let mut builder = PlayerControlRequestBuilder::new(Method::PUT, API_PLAYER_SEEK_ENDPOINT, self.clone())
            .append_query(SEEK_POSITION_QUERY, position);

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }

    /// Add a playable item to the end of the current playback queue.
    ///
    /// A Spotify device ID in the user's account may be supplied with the [`device_id`-function in the request builder
    /// this function returns](BasePlayerControlRequestBuilder::device_id) such that playback will be targeted on that
    /// device. If no device is given, playback will be targeted on the user's currently active device. In case no
    /// device is active and no device is given, the function will
    /// return an [Error::NoActiveDevice](crate::error::Error::NoActiveDevice).
    ///
    /// Required scope: [UserModifyPlaybackState](crate::scope::Scope::UserModifyPlaybackState).
    fn add_to_queue<'a>(&'a self, item: PlayableItem<'a>) -> PlayerControlRequestBuilder<Self> {
        let mut builder = PlayerControlRequestBuilder::new(Method::POST, API_PLAYER_QUEUE_ENDPOINT, self.clone())
            .append_query(QUEUE_URI_QUERY, item.as_uri().to_string());

        #[cfg(feature = "async")]
        {
            builder = builder.with_async_response_handler(Box::new(handle_player_control_response_async));
        }

        #[cfg(feature = "sync")]
        {
            builder = builder.with_sync_response_handler(Box::new(handle_player_control_response_sync));
        }

        builder
    }
}

impl<C, TBody> BaseRequestBuilderContainer<(), C, TBody> for BasePlayerControlRequestBuilder<C, TBody> {
    fn new<S>(method: Method, base_url: S, client: C) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new(method, base_url, client))
    }

    fn new_with_body<S>(method: Method, base_url: S, body: TBody, client: C) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new_with_body(method, base_url, body, client))
    }

    fn take_base_builder(self) -> RequestBuilder<(), C, TBody> {
        self.0
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<(), C, TBody> {
        &mut self.0
    }
}

impl<C, TReturn> BasePlayerControlRequestBuilder<C, TReturn> {
    /// Target playback on a certain Spotify device in the user's account.
    pub fn device_id<S>(self, device_id: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.append_query(DEVICE_ID_QUERY, device_id.into())
    }
}

#[cfg(feature = "async")]
fn handle_player_control_response_async(
    response: reqwest::Response,
) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send>> {
    Box::pin(async move {
        match response.status() {
            StatusCode::NO_CONTENT => Ok(response),

            StatusCode::NOT_FOUND => {
                warn!("Got 404 Not Found to play call");
                let error_response: ApiErrorResponse = response.json().await?;

                match error_response.error.message {
                    ApiErrorMessage::NoActiveDevice | ApiErrorMessage::NotFound => {
                        warn!("Player call failed: no active device or playback failed on active device");
                        Err(Error::NoActiveDevice)
                    }

                    other => {
                        error!("Unexpected Spotify error response to player call: {:?}", other);
                        Err(Error::UnhandledSpotifyResponseStatusCode(404))
                    }
                }
            }

            other => {
                warn!("Got unexpected response status to player call: {}", other);
                let body = response.text().await?;
                warn!("Response body: {body}");

                Err(Error::UnhandledSpotifyResponseStatusCode(other.as_u16()))
            }
        }
    })
}

#[cfg(feature = "sync")]
fn handle_player_control_response_sync(response: reqwest::blocking::Response) -> Result<reqwest::blocking::Response> {
    match response.status() {
        StatusCode::NO_CONTENT => Ok(response),

        StatusCode::NOT_FOUND => {
            warn!("Got 404 Not Found to play call");
            let error_response: ApiErrorResponse = response.json()?;

            match error_response.error.message {
                ApiErrorMessage::NoActiveDevice => {
                    warn!("Player call failed: no active device");
                    Err(Error::NoActiveDevice)
                }

                other => {
                    error!("Unexpected Spotify error response to player call: {:?}", other);
                    Err(Error::UnhandledSpotifyResponseStatusCode(404))
                }
            }
        }

        other => {
            warn!("Got unexpected response status to player call: {}", other);
            let body = response.text()?;
            warn!("Response body: {body}");

            Err(Error::UnhandledSpotifyResponseStatusCode(other.as_u16()))
        }
    }
}
