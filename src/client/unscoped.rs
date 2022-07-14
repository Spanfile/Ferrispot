use super::{private::ClientBase, API_TRACKS_ENDPOINT, API_URL};
use crate::{
    error::Result,
    model::track::{FullTrack, TrackObject},
};
use async_trait::async_trait;
use log::debug;
use reqwest::Method;

#[async_trait]
pub trait UnscopedClient: ClientBase {
    async fn track(&self, track_id: &str) -> Result<FullTrack>;
}

#[async_trait]
impl<C> UnscopedClient for C
where
    C: ClientBase + Sync,
{
    async fn track(&self, track_id: &str) -> Result<FullTrack> {
        let request = self
            .build_http_request(Method::GET, format!("{}{}{}", API_URL, API_TRACKS_ENDPOINT, track_id))
            .await;
        let response = request.send().await?;

        let track_object: TrackObject = response.json().await?;
        debug!("Track response: {:#?}", track_object);

        let full_track: FullTrack = track_object.into();
        Ok(full_track)
    }
}
