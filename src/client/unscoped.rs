use super::{
    private::{self, SendHttpRequest},
    API_SEARCH_ENDPOINT, API_TRACKS_ENDPOINT,
};
use crate::{
    error::Result,
    model::{
        country_code::CountryCode,
        search::{SearchResults, ToTypesString, DEFAULT_SEARCH_TYPES_STRING},
        track::{FullTrack, TrackObject},
    },
};
use async_trait::async_trait;
use log::debug;
use reqwest::{Method, Url};
use serde::Deserialize;
use std::{
    borrow::Cow,
    fmt::{Display, Write},
};

/// All unscoped Spotify endpoints. The functions in this trait do not require user authentication to use. All Spotify
/// clients implement this trait.
#[async_trait]
pub trait UnscopedClient<'a>: private::SendHttpRequest<'a> + private::AccessTokenExpiry
where
    Self: Sized,
{
    /// Get Spotify catalog information for a single track identified by its unique Spotify ID.
    ///
    /// An optional market country may be specified. If specified, only content that is available in that market will be
    /// returned. If using an user-authenticated client (i.e.
    /// [AuthorizationCodeUserClient](crate::client::authorization_code::AuthorizationCodeUserClient) or
    /// [ImplicitGrantUserClient](crate::client::implicit_grant::ImplicitGrantUserClient)), the country associated with
    /// the corresponding user account will take priority over this parameter.
    async fn track<T>(&'a self, track_id: T, market: Option<CountryCode>) -> Result<FullTrack>
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
            .send()
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
    /// [AuthorizationCodeUserClient](crate::client::authorization_code::AuthorizationCodeUserClient) or
    /// [ImplicitGrantUserClient](crate::client::implicit_grant::ImplicitGrantUserClient)), the country associated with
    /// the corresponding user account will take priority over this parameter.
    async fn tracks<I, T>(&'a self, track_ids: I, market: Option<CountryCode>) -> Result<Vec<FullTrack>>
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
            .send()
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

    /// Get Spotify catalog information about albums, artists, playlists, tracks, shows or episodes that match a keyword
    /// string.
    ///
    /// This function returns a [SearchBuilder](self::SearchBuilder) that you can use to configure the various search
    /// parameters and finally send the search and get the results back.
    fn search<S>(&'a self, query: S) -> SearchBuilder<'a, Self, S>
    where
        S: AsRef<str>,
    {
        SearchBuilder::new(self, query)
    }
}

#[async_trait]
impl<'a, C> UnscopedClient<'a> for C where C: private::SendHttpRequest<'a> + Sync {}

/// A builder for a search in Spotify's catalog. New instances are returned by the
/// [search-function](UnscopedClient::search) in [UnscopedClient](UnscopedClient).
pub struct SearchBuilder<'a, C, S>
where
    C: SendHttpRequest<'a>,
    S: AsRef<str>,
{
    client: &'a C,
    query: S,
    types: Cow<'a, str>,
    limit: u32,
    offset: u32,
    market: Option<String>,
}

impl<'a, C, S> SearchBuilder<'a, C, S>
where
    C: SendHttpRequest<'a>,
    S: AsRef<str>,
{
    pub(crate) fn new(client: &'a C, query: S) -> Self {
        Self {
            client,
            query,
            types: Cow::Borrowed(DEFAULT_SEARCH_TYPES_STRING),
            limit: 20,
            offset: 0,
            market: None,
        }
    }

    /// Set specific Spotify item types to search for. The `types` parameter can be any iterator of
    /// [ItemType](crate::model::ItemType)-enums.
    ///
    /// By default, all types are searched for.
    pub fn types<T>(self, types: T) -> Self
    where
        T: ToTypesString,
    {
        Self {
            types: Cow::Owned(types.to_types_string()),
            ..self
        }
    }

    /// The maximum number of results to return in each item type.
    ///
    /// Default: 20.
    /// Maximum: 50.
    pub fn limit(self, limit: u32) -> Self {
        Self { limit, ..self }
    }

    /// The index of the first result to return. By combining this with [limit](SearchBuilder::limit), you may request
    /// new pages of content.
    ///
    /// Default: 0.
    pub fn offset(self, offset: u32) -> Self {
        Self { offset, ..self }
    }

    /// Specify a country such that content that is available in that market will be returned. If using an
    /// user-authenticated client (i.e.
    /// [AuthorizationCodeUserClient](crate::client::authorization_code::AuthorizationCodeUserClient) or
    /// [ImplicitGrantUserClient](crate::client::implicit_grant::ImplicitGrantUserClient)), the country associated with
    /// the corresponding user account will take priority over this parameter.
    pub fn market(self, market: CountryCode) -> Self {
        Self {
            market: Some(market.to_string()),
            ..self
        }
    }

    /// Send the search and return a collection of results.
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

        let response = self.client.send_http_request(Method::GET, url).send().await?;
        response.error_for_status_ref()?;

        let search_results: SearchResults = response.json().await?;
        debug!("Search results object: {:?}", search_results);

        Ok(search_results)
    }
}
