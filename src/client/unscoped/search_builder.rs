use std::borrow::Cow;

use log::trace;
use reqwest::Method;

#[cfg(feature = "async")]
use crate::client::request_builder::AsyncRequestBuilder;
#[cfg(feature = "sync")]
use crate::client::request_builder::SyncRequestBuilder;
use crate::{
    client::{
        private,
        request_builder::{BaseRequestBuilderContainer, RequestBuilder, TryFromEmptyResponse},
        API_SEARCH_ENDPOINT,
    },
    error::Result,
    model::{
        search::{
            AlbumSearchResults, ArtistSearchResults, SearchResults, SearchResultsObject, ToTypesString,
            TrackSearchResults, DEFAULT_SEARCH_TYPES_STRING,
        },
        CountryCode,
    },
};

const SEARCH_QUERY: &str = "q";
const SEARCH_TYPE: &str = "type";
const SEARCH_LIMIT: &str = "limit";
const SEARCH_OFFSET: &str = "offset";
const SEARCH_MARKET: &str = "market";

struct SearchRequestBuilder<TClient>(RequestBuilder<TClient, SearchResultsObject>);

impl TryFromEmptyResponse for SearchResultsObject {}
impl TryFromEmptyResponse for TrackSearchResults {}
impl TryFromEmptyResponse for AlbumSearchResults {}
impl TryFromEmptyResponse for ArtistSearchResults {}

/// A builder for a search in Spotify's catalog. New instances are returned by the
/// [search-function](super::UnscopedClient::search) in [UnscopedClient](super::UnscopedClient)
pub struct SearchBuilder<TClient> {
    client: TClient,
    query: String,
    types: Cow<'static, str>,
    limit: u32,
    offset: u32,
    market: Option<String>,
}

impl<TClient> BaseRequestBuilderContainer<TClient, SearchResultsObject> for SearchRequestBuilder<TClient> {
    fn new<S>(method: Method, base_url: S, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new(method, base_url, client))
    }

    fn new_with_body<S>(method: Method, base_url: S, body: (), client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new_with_body(method, base_url, body, client))
    }

    fn take_base_builder(self) -> RequestBuilder<TClient, SearchResultsObject> {
        self.0
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TClient, SearchResultsObject> {
        &mut self.0
    }
}

impl<C> SearchBuilder<C>
where
    C: Clone,
{
    pub(crate) fn new(client: C, query: String) -> Self {
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

    fn build_search_builder(&self) -> SearchRequestBuilder<C> {
        let builder = SearchRequestBuilder::new(Method::GET, API_SEARCH_ENDPOINT, self.client.clone())
            .append_query(SEARCH_QUERY, self.query.clone())
            .append_query(SEARCH_TYPE, self.types.clone())
            .append_query(SEARCH_LIMIT, self.limit.to_string())
            .append_query(SEARCH_OFFSET, self.offset.to_string());

        if let Some(market) = self.market.as_deref() {
            builder.append_query(SEARCH_MARKET, market.to_string())
        } else {
            builder
        }
    }
}

#[cfg(feature = "async")]
impl<TClient> SearchBuilder<TClient>
where
    TClient: private::BuildHttpRequestAsync + private::AccessTokenExpiryAsync + Clone + Send + Sync,
{
    /// Send the search and return a collection of results.
    pub async fn send_async(self) -> Result<SearchResults> {
        let search_results = self.build_search_builder().send_async().await?;
        trace!("Search results object: {search_results:?}");

        Ok(SearchResults { inner: search_results })
    }
}

#[cfg(feature = "sync")]
impl<TClient> SearchBuilder<TClient>
where
    TClient: private::BuildHttpRequestSync + private::AccessTokenExpirySync + Clone,
{
    /// Send the search and return a collection of results.
    pub fn send_sync(self) -> Result<SearchResults> {
        let search_results = self.build_search_builder().send_sync()?;
        trace!("Search results object: {search_results:?}");

        Ok(SearchResults { inner: search_results })
    }
}
