use std::time::Duration;

use serde::Deserialize;

use super::{track::FullTrack, ExternalUrls, ItemType};
use crate::util::duration_millis;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlaybackState {
    device: Device,
    repeat_state: RepeatState,
    shuffle_state: bool,

    #[serde(flatten)]
    currently_playing: CurrentlyPlayingItem,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CurrentlyPlayingItem {
    timestamp: u64, // TODO: this is an unix epoch
    is_playing: bool,
    actions: Actions,

    #[serde(flatten)]
    public_playing_track: Option<PublicPlayingItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PublicPlayingItem {
    context: Context,
    #[serde(rename = "progress_ms", with = "duration_millis")]
    progress: Duration,
    #[serde(flatten)]
    item: PlayingType,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Context {
    #[serde(rename = "type")]
    context_type: ItemType,
    #[serde(default)]
    external_urls: ExternalUrls,
    uri: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Actions {
    pub disallows: Disallows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Disallows {
    #[serde(default)]
    interrupting_playback: bool,
    #[serde(default)]
    pausing: bool,
    #[serde(default)]
    resuming: bool,
    #[serde(default)]
    seeking: bool,
    #[serde(default)]
    skipping_next: bool,
    #[serde(default)]
    skipping_prev: bool,
    #[serde(default)]
    toggling_repeat_context: bool,
    #[serde(default)]
    toggling_shuffle: bool,
    #[serde(default)]
    toggling_repeat_track: bool,
    #[serde(default)]
    transferring_playback: bool,
}

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

    /// Whether controlling this device is restricted. At present if this is `true` when no Web API commands will be
    /// accepted by this device.
    pub fn is_restricted(&self) -> bool {
        self.is_restricted
    }

    /// The type of the device.
    pub fn device_type(&self) -> DeviceType {
        self.device_type
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

    pub fn currently_playing_item(&self) -> &CurrentlyPlayingItem {
        &self.currently_playing
    }

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
    /// The item's context.
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
