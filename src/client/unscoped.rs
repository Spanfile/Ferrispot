use super::{
    private::{self, SendHttpRequest},
    API_SEARCH_ENDPOINT, API_TRACKS_ENDPOINT,
};
use crate::{
    error::Result,
    model::{
        country_code::CountryCode,
        search::{SearchResults, SearchType, ToTypesString},
        track::{FullTrack, TrackObject},
    },
};
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Url};
use serde::Deserialize;
use std::fmt::{Display, Write};

/// All unscoped Spotify endpoints.
///
/// The functions in this trait do not require user authentication to use. All Spotify clients implement this trait.
#[async_trait]
pub trait UnscopedClient<'a>: private::SendHttpRequest
where
    Self: Sized,
{
    /// Get Spotify catalog information for a single track identified by its unique Spotify ID.
    ///
    /// An optional market country may be specified. If specified, only content that is available in that market will be
    /// returned. If using an user-authenticated client (i.e.
    /// [AuthorizationCodeUserClient](crate::client::AuthorizationCodeUserClient) or
    /// [ImplicitGrantUserClient](crate::client::ImplicitGrantUserClient)), the country associated with the
    /// corresponding user account will take priority over this parameter.
    async fn track<T>(&self, track_id: T, market: Option<CountryCode>) -> Result<FullTrack>
    where
        T: Display + Send,
    {
        // TODO: gonna need a way more robust way of constructing the URLs
        let mut url = format!("{}/{}", API_TRACKS_ENDPOINT, track_id);

        if let Some(market) = market {
            write!(&mut url, "?market={}", market).expect(
                "failed to build API track endpoint URL (this is likely caused by the system, e.g. failing to \
                 allocate memory)",
            );
        }

        let response = self
            .send_http_request(
                Method::GET,
                Url::parse(&url).expect("failed to build tracks endpoint URL"),
            )
            .await?;
        debug!("Track response: {:?}", response);

        // TODO: is this really the way to return an error from an error response?
        response.error_for_status_ref()?;

        let track_object: TrackObject = response.json().await?;
        debug!("Track body: {:#?}", track_object);

        let full_track: FullTrack = track_object.into();
        Ok(full_track)
    }

    /// Get Spotify catalog information for multiple tracks based on their Spotify IDs.
    ///
    /// Up to 50 IDs may be given. In case some IDs cannot be found, they will be omitted from the result.
    ///
    /// An optional market country may be specified. If specified, only content that is available in that market will be
    /// returned. If using an user-authenticated client (i.e.
    /// [AuthorizationCodeUserClient](crate::client::AuthorizationCodeUserClient) or
    /// [ImplicitGrantUserClient](crate::client::ImplicitGrantUserClient)), the country associated with the
    /// corresponding user account will take priority over this parameter.
    async fn tracks<I, T>(&self, track_ids: I, market: Option<CountryCode>) -> Result<Vec<FullTrack>>
    where
        I: IntoIterator<Item = T> + Send,
        <I as IntoIterator>::IntoIter: Send,
        T: Display + Send,
    {
        #[derive(Debug, Deserialize)]
        struct TracksResponse {
            tracks: Vec<Option<TrackObject>>,
        }

        let mut params = vec![(
            "ids",
            track_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join(","),
        )];

        if let Some(market) = market {
            params.push(("market", market.to_string()));
        }

        let response = self
            .send_http_request(
                Method::GET,
                Url::parse_with_params(API_TRACKS_ENDPOINT, params)
                    .expect("failed to parse API tracks endpoint as URL (this is a bug in the library)"),
            )
            .await?;

        debug!("Tracks response: {:?}", response);

        // TODO: is this really the way to return an error from an error response?
        response.error_for_status_ref()?;

        let tracks_object: TracksResponse = response.json().await?;
        debug!("Tracks body: {:#?}", tracks_object);

        let full_tracks: Vec<_> = tracks_object
            .tracks
            .into_iter()
            .filter_map(|obj| obj.map(FullTrack::from))
            .collect();
        Ok(full_tracks)
    }

    fn search<S>(&'a self, query: S) -> SearchBuilder<'a, Self, S>
    where
        S: AsRef<str>,
    {
        SearchBuilder {
            client: self,
            query,
            types: [
                SearchType::Album,
                SearchType::Artist,
                SearchType::Episode,
                SearchType::Playlist,
                SearchType::Show,
                SearchType::Track,
            ]
            .to_types_string(),
            limit: 20,
            offset: 0,
            market: None,
        }
    }
}

#[async_trait]
impl<C> UnscopedClient<'_> for C where C: private::SendHttpRequest + Sync {}

pub struct SearchBuilder<'a, C, S>
where
    C: SendHttpRequest,
    S: AsRef<str>,
{
    client: &'a C,
    query: S,
    types: String,
    limit: u32,
    offset: u32,
    market: Option<String>,
}

impl<C, S> SearchBuilder<'_, C, S>
where
    C: SendHttpRequest,
    S: AsRef<str>,
{
    pub fn types<T>(self, types: T) -> Self
    where
        T: ToTypesString,
    {
        Self {
            types: types.to_types_string(),
            ..self
        }
    }

    pub fn limit(self, limit: u32) -> Self {
        Self { limit, ..self }
    }

    pub fn offset(self, offset: u32) -> Self {
        Self { offset, ..self }
    }

    pub fn market(self, market: CountryCode) -> Self {
        Self {
            market: Some(market.to_string()),
            ..self
        }
    }

    pub async fn send(self) -> Result<SearchResults> {
        let limit = self.limit.to_string();
        let offset = self.offset.to_string();

        let mut params = vec![
            ("q", self.query.as_ref()),
            ("type", &self.types),
            ("limit", &limit),
            ("offset", &offset),
        ];

        if let Some(market) = self.market.as_deref() {
            params.push(("market", market));
        };

        let url = Url::parse_with_params(API_SEARCH_ENDPOINT, params)
            .expect("failed to parse API tracks endpoint as URL (this is a bug in the library)");

        let response = self.client.send_http_request(Method::GET, url).await?;
        response.error_for_status_ref()?;

        let search_results: SearchResults = response.json().await?;
        debug!("Search results object: {:?}", search_results);

        Ok(search_results)
    }
}
