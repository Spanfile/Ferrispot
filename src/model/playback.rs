use std::time::Duration;

use serde::Deserialize;

use super::{id::PlayableContext, track::FullTrack, ExternalUrls, ItemType};
use crate::{prelude::IdTrait, util::duration_millis};

/// A device in an user's account that may be used for playback.
#[derive(Debug, Clone, Eq, Deserialize)]
pub struct Device {
    name: String,
    // TODO: aspotify says this and the volume can be nonexistent for whatever reason but I haven't ever seen that
    // happen so?
    id: String,
    volume_percent: u8,
    is_active: bool,
    is_private_session: bool,
    is_restricted: bool,
    #[serde(rename = "type")]
    device_type: DeviceType,
}

/// A device's type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum DeviceType {
    Computer,
    Tablet,
    Smartphone,
    Speaker,
    TV,
    AVR,
    STB,
    AudioDongle,
    GameConsole,
    CastVideo,
    CastAudio,
    Automobile,
    Unknown,
}

/// Current playback state. Contains information about which device is playing, what the repeat and shuffle states are
/// and which item is currently playing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlaybackState {
    device: Device,
    repeat_state: RepeatState,
    shuffle_state: bool,

    #[serde(flatten)]
    currently_playing: CurrentlyPlayingItem,
}

/// Currently playing item.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CurrentlyPlayingItem {
    timestamp: u64, // TODO: this is an unix epoch
    is_playing: bool,
    actions: Actions,

    #[serde(flatten)]
    public_playing_track: Option<PublicPlayingItem>,
}

/// A public playing item.
///
/// Public refers to the playing item and its context being publicly available through the API. The item is not
/// considered public when, but not limited to:
/// - The user has enabled a private session.
/// - The playing context is private, e.g. a private playlist.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PublicPlayingItem {
    context: Context,
    #[serde(rename = "progress_ms", with = "duration_millis")]
    progress: Duration,
    #[serde(flatten)]
    item: PlayingType,
}

/// The context of the current playback (i.e. album, artist, playlist or show).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Context {
    #[serde(rename = "type")]
    context_type: ItemType,
    #[serde(default)]
    external_urls: ExternalUrls,
    uri: PlayableContext<'static>,
}

/// What actions can be taken on the current playing item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Actions {
    /// Disallowed actions on the current playing item.
    pub disallows: Disallows,
}

/// Disallowed actions on the current playing item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Disallows {
    #[serde(default)]
    pub interrupting_playback: bool,
    #[serde(default)]
    pub pausing: bool,
    #[serde(default)]
    pub resuming: bool,
    #[serde(default)]
    pub seeking: bool,
    #[serde(default)]
    pub skipping_next: bool,
    #[serde(default)]
    pub skipping_prev: bool,
    #[serde(default)]
    pub toggling_repeat_context: bool,
    #[serde(default)]
    pub toggling_shuffle: bool,
    #[serde(default)]
    pub toggling_repeat_track: bool,
    #[serde(default)]
    pub transferring_playback: bool,
}

/// The kind of item that is playing.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", tag = "currently_playing_type", content = "item")]
#[non_exhaustive]
pub enum PlayingType {
    Track(FullTrack),
    // TODO:
    // Episode
    // Ad
    // Unknown
}

/// Possible item repeat states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepeatState {
    Off,
    Track,
    Context,
}

impl Device {
    /// The name of the device.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The device ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The current volume as a percentage between 0 and 100 inclusive.
    pub fn volume_percent(&self) -> u8 {
        self.volume_percent
    }

    /// If this device is the currently active device.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// If this device is currently in a private session.
    pub fn is_private_session(&self) -> bool {
        self.is_private_session
    }

    /// Whether controlling this device is restricted. If this is `true`, no Web API commands will be accepted by this
    /// device.
    pub fn is_restricted(&self) -> bool {
        self.is_restricted
    }

    /// The type of the device.
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PlaybackState {
    /// The device currently playing.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// The device currently playing. Take ownership of the `Device` object.
    pub fn take_device(self) -> Device {
        self.device
    }

    /// The current playback's repeat state.
    pub fn repeat_state(&self) -> RepeatState {
        self.repeat_state
    }

    /// The current playback's shuffle state.
    pub fn shuffle_state(&self) -> bool {
        self.shuffle_state
    }

    /// The currently playing item.
    pub fn currently_playing_item(&self) -> &CurrentlyPlayingItem {
        &self.currently_playing
    }

    /// The currently playing item. Take ownership of the value.
    pub fn take_currently_playing_item(self) -> CurrentlyPlayingItem {
        self.currently_playing
    }
}

impl CurrentlyPlayingItem {
    /// The item's timestamp.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Whether or not the item is playing.
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// The actions that may be taken on the item.
    pub fn actions(&self) -> Actions {
        self.actions
    }

    /// The currently playing public item.
    pub fn public_playing_item(&self) -> Option<&PublicPlayingItem> {
        self.public_playing_track.as_ref()
    }

    /// The currently playing public item. Take ownership of the value.
    pub fn take_public_playing_item(self) -> Option<PublicPlayingItem> {
        self.public_playing_track
    }
}

impl PublicPlayingItem {
    /// The item's playback context (i.e. album, artist, playlist or show).
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// The item's playback progress.
    pub fn progress(&self) -> Duration {
        self.progress
    }

    /// The playing item.
    pub fn item(&self) -> &PlayingType {
        &self.item
    }

    /// The playing item. Take ownership of the item.
    pub fn take_item(self) -> PlayingType {
        self.item
    }
}

impl RepeatState {
    pub fn as_str(self) -> &'static str {
        match self {
            RepeatState::Off => "off",
            RepeatState::Track => "track",
            RepeatState::Context => "context",
        }
    }
}

impl Context {
    pub fn external_urls(&self) -> &ExternalUrls {
        &self.external_urls
    }

    pub fn id(&self) -> PlayableContext {
        self.uri.as_borrowed()
    }
}
