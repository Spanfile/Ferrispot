use super::{private::ClientBase, API_TRACKS_ENDPOINT};
use crate::{
    error::Result,
    model::track::{FullTrack, TrackObject},
};
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Url};
use serde::Deserialize;
use std::fmt::Display;

#[async_trait]
pub trait UnscopedClient: ClientBase {
    async fn track<T>(&self, track_id: T) -> Result<FullTrack>
    where
        T: Display + Send;

    async fn tracks<I, T>(&self, track_ids: I) -> Result<Vec<FullTrack>>
    where
        I: IntoIterator<Item = T> + Send,
        <I as IntoIterator>::IntoIter: Send,
        T: Display + Send;
}

#[async_trait]
impl<C> UnscopedClient for C
where
    C: ClientBase + Sync,
{
    async fn track<T>(&self, track_id: T) -> Result<FullTrack>
    where
        T: Display + Send,
    {
        // TODO: gonna need a way more robust way of constructing the URLs
        let request = self
            .build_http_request(Method::GET, format!("{}/{}", API_TRACKS_ENDPOINT, track_id))
            .await;

        let response = request.send().await?;

        let track_object: TrackObject = response.json().await?;
        debug!("Track response: {:#?}", track_object);

        let full_track: FullTrack = track_object.into();
        Ok(full_track)
    }

    async fn tracks<I, T>(&self, track_ids: I) -> Result<Vec<FullTrack>>
    where
        I: IntoIterator<Item = T> + Send,
        <I as IntoIterator>::IntoIter: Send,
        T: Display + Send,
    {
        #[derive(Debug, Deserialize)]
        struct TracksResponse {
            tracks: Vec<TrackObject>,
        }

        let request = self
            .build_http_request(
                Method::GET,
                Url::parse_with_params(
                    API_TRACKS_ENDPOINT,
                    [(
                        "ids",
                        track_ids
                            .into_iter()
                            .map(|id| id.to_string())
                            .collect::<Vec<String>>()
                            .join(","),
                    )],
                )
                // TODO: can this fail because of malformed IDs?
                .unwrap(),
            )
            .await;

        let response = request.send().await?;
        let tracks_object: TracksResponse = response.json().await?;
        debug!("Tracks response: {:#?}", tracks_object);

        let full_tracks: Vec<_> = tracks_object.tracks.into_iter().map(FullTrack::from).collect();
        Ok(full_tracks)
    }
}
