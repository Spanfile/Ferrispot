use serde::{Deserialize, Serialize};

use crate::{
    client::request_builder::TryFromEmptyResponse,
    error::ConversionError,
    model::{
        playback::Device,
        track::{FullTrack, TrackObject},
        user::{PublicUser, User},
    },
};

pub const DEVICE_ID_QUERY: &str = "device_id";
pub const REPEAT_STATE_QUERY: &str = "repeat_state";
pub const SHUFFLE_QUERY: &str = "shuffle";
pub const VOLUME_PERCENT_QUERY: &str = "volume_percent";
pub const SEEK_POSITION_QUERY: &str = "position_ms";
pub const QUEUE_URI_QUERY: &str = "uri";

pub const TRACKS_IDS_QUERY: &str = "ids";
pub const MARKET_QUERY: &str = "market";

#[derive(Debug, Serialize)]
pub struct PlayItemsBody {
    pub uris: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PlayContextBody {
    pub context_uri: String,
    pub offset: PlayContextOffset,
}

#[derive(Debug, Serialize)]
pub struct PlayContextOffset {
    pub position: Option<u32>,
    // TODO: support URI offsets
    pub uri: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DevicesResponse {
    pub devices: Vec<Device>,
}

#[derive(Debug, Deserialize)]
pub struct TracksResponse {
    tracks: Vec<Option<TrackObject>>,
}

impl TracksResponse {
    pub fn full_tracks(self) -> std::result::Result<Vec<FullTrack>, ConversionError> {
        self.tracks
            .into_iter()
            .filter_map(|obj| obj.map(FullTrack::try_from))
            .collect::<std::result::Result<Vec<_>, ConversionError>>()
    }
}

impl TryFrom<TracksResponse> for Vec<FullTrack> {
    type Error = ConversionError;

    fn try_from(value: TracksResponse) -> Result<Self, Self::Error> {
        value
            .tracks
            .into_iter()
            .filter_map(|obj| obj.map(FullTrack::try_from))
            .collect::<std::result::Result<Vec<_>, ConversionError>>()
    }
}

impl From<DevicesResponse> for Vec<Device> {
    fn from(response: DevicesResponse) -> Self {
        response.devices
    }
}

// TryFromEmptyResponse already has blanket implementations for Option and Vec; implement it for every other object
// (can't have a blanket implementation for everything since specialisation isn't a thing yet)
impl TryFromEmptyResponse for DevicesResponse {}
impl TryFromEmptyResponse for TracksResponse {}
impl TryFromEmptyResponse for FullTrack {}
impl TryFromEmptyResponse for TrackObject {}
impl TryFromEmptyResponse for User {}
impl TryFromEmptyResponse for PublicUser {}
