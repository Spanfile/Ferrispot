use std::borrow::Cow;

use reqwest::Method;

use crate::{
    client::request_builder::{BaseRequestBuilderContainer, RequestBuilder, TryFromEmptyResponse},
    model::{
        search::{
            AlbumSearchResults, ArtistSearchResults, SearchResults, SearchResultsObject, ToTypesString,
            TrackSearchResults, DEFAULT_SEARCH_LIMIT, DEFAULT_SEARCH_OFFSET, DEFAULT_SEARCH_TYPES_STRING,
        },
        CountryCode,
    },
};

const SEARCH_QUERY: &str = "q";
const SEARCH_TYPE: &str = "type";
const SEARCH_LIMIT: &str = "limit";
const SEARCH_OFFSET: &str = "offset";
const SEARCH_MARKET: &str = "market";

impl TryFromEmptyResponse for SearchResultsObject {}
impl TryFromEmptyResponse for TrackSearchResults {}
impl TryFromEmptyResponse for AlbumSearchResults {}
impl TryFromEmptyResponse for ArtistSearchResults {}

/// A builder for a search in Spotify's catalog. New instances are returned by the
/// [search-function](super::UnscopedClient::search) in [UnscopedClient](super::UnscopedClient)
pub struct SearchBuilder<TClient>(RequestBuilder<TClient, SearchResultsObject, (), SearchResults>);

impl<TClient> BaseRequestBuilderContainer<TClient, SearchResultsObject, (), SearchResults> for SearchBuilder<TClient> {
    fn new<S>(method: Method, base_url: S, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(
            RequestBuilder::new(method, base_url, client)
                .append_query(SEARCH_TYPE, DEFAULT_SEARCH_TYPES_STRING)
                .append_query(SEARCH_LIMIT, DEFAULT_SEARCH_LIMIT.to_string())
                .append_query(SEARCH_OFFSET, DEFAULT_SEARCH_OFFSET.to_string()),
        )
    }

    fn new_with_body<S>(method: Method, base_url: S, body: (), client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new_with_body(method, base_url, body, client))
    }

    fn take_base_builder(self) -> RequestBuilder<TClient, SearchResultsObject, (), SearchResults> {
        self.0
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TClient, SearchResultsObject, (), SearchResults> {
        &mut self.0
    }
}

impl<C> SearchBuilder<C>
where
    C: Clone,
{
    pub(crate) fn query(self, query: String) -> Self {
        self.append_query(SEARCH_QUERY, query)
    }

    /// Set specific Spotify item types to search for. The `types` parameter can be any iterator of
    /// [ItemType](crate::model::ItemType)-enums.
    ///
    /// By default, all types are searched for.
    pub fn types<T>(self, types: T) -> Self
    where
        T: ToTypesString,
    {
        self.append_query(SEARCH_TYPE, types.to_types_string())
    }

    /// The maximum number of results to return in each item type.
    ///
    /// Default: 20. Maximum: 50.
    pub fn limit(self, limit: u32) -> Self {
        self.append_query(SEARCH_LIMIT, limit.to_string())
    }

    /// The index of the first result to return. By combining this with [limit](SearchBuilder::limit), you may request
    /// new pages of content.
    ///
    /// Default: 0.
    pub fn offset(self, offset: u32) -> Self {
        self.append_query(SEARCH_OFFSET, offset.to_string())
    }

    /// Specify a country such that content that is available in that market will be returned. If using an
    /// user-authenticated client, the country associated with the corresponding user account will take priority over
    /// this parameter.
    pub fn market(self, market: CountryCode) -> Self {
        self.append_query(SEARCH_MARKET, market.to_string())
    }
}
