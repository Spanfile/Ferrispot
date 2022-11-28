use std::{
    borrow::Cow,
    fmt::{Display, Write},
};

use log::trace;
use reqwest::{Method, Url};
use serde::Deserialize;

use super::{private, API_SEARCH_ENDPOINT, API_TRACKS_ENDPOINT};
use crate::{
    error::{ConversionError, Result},
    model::{
        search::{SearchResults, ToTypesString, DEFAULT_SEARCH_TYPES_STRING},
        track::{FullTrack, TrackObject},
        CountryCode,
    },
};

/// All unscoped Spotify endpoints. The functions in this trait do not require user authentication to use. All
/// asynchronous Spotify clients implement this trait.
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait UnscopedAsyncClient<'a>: private::SendHttpRequestAsync<'a> + private::AccessTokenExpiryAsync
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
                Url::parse(&url).expect("failed to build tracks endpoint URL: invalid base URL (this is likely a bug)"),
            )
            .send_async()
            .await?
            .error_for_status()
            .map_err(super::response_to_error)?;

        trace!("Track response: {:?}", response);

        let track_object: TrackObject = response.json().await?;
        trace!("Track body: {:?}", track_object);

        let full_track: FullTrack = track_object.try_into()?;
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
                    .expect("failed to parse API tracks endpoint as URL: invalid base URL (this is likely a bug)"),
            )
            .send_async()
            .await?
            .error_for_status()
            .map_err(super::response_to_error)?;

        trace!("Tracks response: {:?}", response);

        let tracks_object: TracksResponse = response.json().await?;
        trace!("Tracks body: {:?}", tracks_object);

        let full_tracks: Vec<_> = tracks_object
            .tracks
            .into_iter()
            .filter_map(|obj| obj.map(FullTrack::try_from))
            .collect::<std::result::Result<Vec<_>, ConversionError>>()?;

        Ok(full_tracks)
    }

    /// Get Spotify catalog information about albums, artists, playlists, tracks, shows or episodes that match a keyword
    /// string.
    ///
    /// This function returns a [SearchBuilder](self::SearchBuilder) that you can use to configure the various search
    /// parameters and finally send the search query and get the results back.
    fn search<S>(&'a self, query: S) -> SearchBuilder<'a, Self, S>
    where
        S: AsRef<str>,
    {
        SearchBuilder::new(self, query)
    }
}

/// A builder for a search in Spotify's catalog. New instances are returned by the
/// [search-function](UnscopedAsyncClient::search) in [UnscopedAsyncClient](UnscopedAsyncClient).
pub struct SearchBuilder<'a, C, S>
where
    C: ?Sized,
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
    C: ?Sized,
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

    fn build_search_url(&self) -> Url {
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

        Url::parse_with_params(API_SEARCH_ENDPOINT, params)
            .expect("failed to parse API tracks endpoint as URL: invalid base URL (this is likely a bug)")
    }
}

#[cfg(feature = "async")]
impl<'a, C, S> SearchBuilder<'a, C, S>
where
    C: private::SendHttpRequestAsync<'a>,
    S: AsRef<str>,
{
    /// Send the search and return a collection of results.
    pub async fn send_async(self) -> Result<SearchResults> {
        let response = self
            .client
            .send_http_request(Method::GET, self.build_search_url())
            .send_async()
            .await?;

        trace!("Search results response: {:?}", response);

        let search_results = response.json().await?;
        trace!("Search results object: {:?}", search_results);

        Ok(SearchResults { inner: search_results })
    }
}

#[cfg(feature = "sync")]
impl<'a, C, S> SearchBuilder<'a, C, S>
where
    C: private::SendHttpRequestSync<'a>,
    S: AsRef<str>,
{
    /// Send the search and return a collection of results.
    pub fn send_sync(self) -> Result<SearchResults> {
        let response = self
            .client
            .send_http_request(Method::GET, self.build_search_url())
            .send_sync()?;

        trace!("Search results response: {:?}", response);

        let search_results = response.json()?;
        trace!("Search results object: {:?}", search_results);

        Ok(SearchResults { inner: search_results })
    }
}
