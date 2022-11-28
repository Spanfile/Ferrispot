use std::borrow::Cow;

use log::trace;
use reqwest::{Method, Url};

use crate::{
    client::{private, API_SEARCH_ENDPOINT},
    error::Result,
    model::{
        search::{SearchResults, ToTypesString, DEFAULT_SEARCH_TYPES_STRING},
        CountryCode,
    },
};

/// A builder for a search in Spotify's catalog. New instances are returned by the
/// [asynchronous search-function](super::UnscopedAsyncClient::search) in
/// [UnscopedAsyncClient](super::UnscopedAsyncClient) or the [synchronous
/// search-function](super::UnscopedSyncClient::search) in [UnscopedSyncClient](super::UnscopedSyncClient)
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
    /// Default: 20. Maximum: 50.
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
    /// user-authenticated client, the country associated with the corresponding user account will take priority over
    /// this parameter.
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
