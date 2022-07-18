use super::{
    id::{PlayableContext, PlayableItem},
    track::{FullTrack, TrackObject},
    ExternalUrls, ItemType,
};
use crate::util::duration_millis;
use serde::Deserialize;
use std::{iter::IntoIterator, time::Duration};

pub enum Play<'a> {
    Context(PlayableContext<'a>),
    Items(PlayTracks<'a>),
}

pub struct PlayTracks<'a>(pub(crate) Box<dyn Iterator<Item = PlayableItem<'a>> + Send>);

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
    currently_playing: CurrentlyPlayingTrack,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CurrentlyPlayingTrack {
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
    item: PlayingTypeObject,
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
pub enum PlayingType {
    Track(FullTrack),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case", tag = "currently_playing_type", content = "item")]
enum PlayingTypeObject {
    Track(TrackObject),
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

impl<'a, I> From<I> for PlayTracks<'a>
where
    I: IntoIterator<Item = PlayableItem<'a>> + 'static,
    <I as IntoIterator>::IntoIter: Send,
{
    fn from(iter: I) -> Self {
        PlayTracks(Box::new(iter.into_iter()))
    }
}

impl Device {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn volume_percent(&self) -> u8 {
        self.volume_percent
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_private_session(&self) -> bool {
        self.is_private_session
    }

    pub fn is_restricted(&self) -> bool {
        self.is_restricted
    }

    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }
}

impl PlaybackState {
    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn repeat_state(&self) -> RepeatState {
        self.repeat_state
    }

    pub fn shuffle_state(&self) -> bool {
        self.shuffle_state
    }

    pub fn currently_playing_item(&self) -> &CurrentlyPlayingTrack {
        &self.currently_playing
    }
}

impl CurrentlyPlayingTrack {
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn actions(&self) -> Actions {
        self.actions
    }

    pub fn public_playing_item(&self) -> Option<&PublicPlayingItem> {
        self.public_playing_track.as_ref()
    }
}

impl PublicPlayingItem {
    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn progress(&self) -> Duration {
        self.progress
    }

    pub fn item(&self) -> PlayingType {
        match &self.item {
            PlayingTypeObject::Track(track_obj) => PlayingType::Track(track_obj.to_owned().into()),
        }
    }
}
