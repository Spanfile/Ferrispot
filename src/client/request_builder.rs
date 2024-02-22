//! Contains the various request builders and the request builder functionality traits.
// TODO: docs about using request builders?

mod private {
    use std::borrow::Cow;

    use reqwest::Method;

    #[cfg(feature = "async")]
    use super::AsyncResponseHandler;
    use super::RequestBuilder;
    #[cfg(feature = "sync")]
    use super::SyncResponseHandler;
    use crate::error::{Error, Result};

    pub trait BaseRequestBuilderContainer<TClient, TResponse, TBody = (), TReturn = TResponse>
    where
        Self: Sized,
    {
        fn new<S>(method: Method, base_url: S, client: TClient) -> Self
        where
            S: Into<Cow<'static, str>>;

        fn new_with_body<S>(method: Method, base_url: S, body: TBody, client: TClient) -> Self
        where
            S: Into<Cow<'static, str>>;

        fn take_base_builder(self) -> RequestBuilder<TClient, TResponse, TBody, TReturn>;
        fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TClient, TResponse, TBody, TReturn>;

        fn replace_body<F>(self, replacer: F) -> Self
        where
            F: FnOnce(TBody) -> TBody,
        {
            let common = self.take_base_builder();
            if let Some(body) = common.body {
                let new_body = (replacer)(body);
                Self::new_with_body(common.method, common.base_url, new_body, common.client)
            } else {
                Self::new(common.method, common.base_url, common.client)
            }
        }

        fn append_query<S>(mut self, key: &'static str, value: S) -> Self
        where
            S: Into<Cow<'static, str>>,
        {
            self.get_base_builder_mut().query_params.insert(key, value.into());
            self
        }

        #[cfg(feature = "async")]
        fn with_async_response_handler(mut self, handler: AsyncResponseHandler) -> Self {
            self.get_base_builder_mut().async_response_handler = handler;
            self
        }

        #[cfg(feature = "sync")]
        fn with_sync_response_handler(mut self, handler: SyncResponseHandler) -> Self {
            self.get_base_builder_mut().sync_response_handler = handler;
            self
        }
    }

    // TODO: I really do not like having to use this trait but not doing so would require, yet again, stabilised
    // specialisation
    /// This trait allows graceful handling of empty 200 responses vs actually empty 204 responses. In case the Spotify
    /// API returns a 204, the return type is assumed to be the unit type. serde/serde_json won't deserialize the unit
    /// type from an empty string (the nonexistent body in the response) so instead this trait is used to bypass the
    /// serialization and return an appropriate response for the 204.
    pub trait TryFromEmptyResponse
    where
        Self: Sized,
    {
        /// Return an appropriate result for an empty response (204).
        fn try_from_empty_response() -> Result<Self> {
            Err(Error::EmptyResponse)
        }
    }

    impl TryFromEmptyResponse for () {
        /// Return a successful result containing the unit type.
        fn try_from_empty_response() -> Result<Self> {
            Ok(())
        }
    }

    impl<T> TryFromEmptyResponse for Option<T> {
        /// Return a successful result containing `None`.
        fn try_from_empty_response() -> Result<Self> {
            Ok(None)
        }
    }

    impl<T> TryFromEmptyResponse for Vec<T> {
        /// Return a successful result containing an empty vector.
        fn try_from_empty_response() -> Result<Self> {
            Ok(Vec::new())
        }
    }
}

mod catalog_item_builder;
mod player_control_builder;
mod search_builder;

use std::{borrow::Cow, collections::HashMap, fmt::Debug, marker::PhantomData};
#[cfg(feature = "async")]
use std::{future::Future, pin::Pin};

use log::{error, info, trace, warn};
use reqwest::{header, header::HeaderMap, Method, StatusCode, Url};
use serde::{de::DeserializeOwned, Serialize};

pub(crate) use self::private::{BaseRequestBuilderContainer, TryFromEmptyResponse};
pub use self::{
    catalog_item_builder::CatalogItemRequestBuilder,
    player_control_builder::{
        BasePlayerControlRequestBuilder, PlayContextRequestBuilder, PlayItemsRequestBuilder,
        PlayerControlRequestBuilder,
    },
    search_builder::SearchBuilder,
};
use crate::{
    client::private::AccessTokenExpiryResult,
    error::{Error, Result},
    model::error::{ApiErrorMessage, ApiErrorResponse},
};

#[cfg(feature = "async")]
pub(crate) type AsyncResponseHandler =
    Box<dyn FnOnce(reqwest::Response) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send>> + Send>;

#[cfg(feature = "sync")]
pub(crate) type SyncResponseHandler =
    Box<dyn FnOnce(reqwest::blocking::Response) -> Result<reqwest::blocking::Response> + Send>;

#[cfg(feature = "async")]
fn async_response_handler_noop(
    resp: reqwest::Response,
) -> Pin<Box<dyn Future<Output = Result<reqwest::Response>> + Send>> {
    Box::pin(async move {
        match resp.error_for_status() {
            Ok(resp) => Ok(resp),
            Err(e) => Err(response_error_to_unhandled_code(e)),
        }
    })
}

#[cfg(feature = "sync")]
fn sync_response_handler_noop(resp: reqwest::blocking::Response) -> Result<reqwest::blocking::Response> {
    match resp.error_for_status() {
        Ok(resp) => Ok(resp),
        Err(e) => Err(response_error_to_unhandled_code(e)),
    }
}

fn response_error_to_unhandled_code(err: reqwest::Error) -> Error {
    match err.status() {
        Some(status) => Error::UnhandledSpotifyResponseStatusCode(status.as_u16()),
        None => err.into(),
    }
}

/// Options available in each request builder.
pub trait BaseRequestBuilder<TClient, TResponse, TBody, TReturn>
where
    Self: private::BaseRequestBuilderContainer<TClient, TResponse, TBody, TReturn> + Sized,
{
    /// Whether or not to react to being rate limited by waiting the wanted time in the response. Defaults to `true`.
    fn react_to_rate_limit(mut self, react_to_rate_limit: bool) -> Self {
        self.get_base_builder_mut().react_to_rate_limit = react_to_rate_limit;
        self
    }

    /// Whether or not to automatically refresh the client's access token, if applicable, when it expires. Defaults to
    /// `true`.
    fn auto_refresh_access_token(mut self, auto_refresh_access_token: bool) -> Self {
        self.get_base_builder_mut().auto_refresh_access_token = auto_refresh_access_token;
        self
    }
}

fn handle_403_forbidden_api_response(error_response: ApiErrorResponse) -> Result<()> {
    warn!("Error response: {error_response:?}");

    match error_response.error.message {
        ApiErrorMessage::RestrictionViolated => Err(Error::Restricted),
        ApiErrorMessage::PremiumRequired => Err(Error::PremiumRequired),

        // TODO: test what actually happens when the user revokes the app's access while the app is
        // running
        _ => Err(Error::Forbidden),
    }
}

/// Returns Ok if the given API error response is because of an expired token. Else, returns an error based on the API
/// error response.
fn is_api_error_expired_access_token(error_response: ApiErrorResponse) -> Result<()> {
    match error_response.error.message {
        ApiErrorMessage::TokenExpired => {
            warn!("Access token expired, attempting to refresh");
            Ok(())
        }

        ApiErrorMessage::PermissionsMissing => {
            error!("Missing required scope for the endpoint");
            Err(Error::MissingScope)
        }

        other => {
            error!("Unexpected Spotify error: {:?}", other);
            Err(Error::UnhandledSpotifyResponseStatusCode(401))
        }
    }
}

fn extract_rate_limit_retry_after(headers: &HeaderMap) -> Result<u64> {
    if let Some(wait_time) = headers
        .get(header::RETRY_AFTER)
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| header_str.parse::<u64>().ok())
    {
        warn!(
            "Got 429 rate-limit response from Spotify with Retry-After: {}",
            wait_time
        );

        Ok(wait_time)
    } else {
        warn!("Invalid rate-limit response");
        Err(Error::InvalidRateLimitResponse)
    }
}

/// Asynchronous request builder functionality, namely sending the request and processing its response asynchronously.
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncRequestBuilder<TClient, TResponse, TBody, TReturn>
where
    Self: BaseRequestBuilder<TClient, TResponse, TBody, TReturn>,
    TBody: Debug + Serialize + Send,
    TResponse: Debug + DeserializeOwned + TryFromEmptyResponse + Send + Sync,
    TReturn: TryFrom<TResponse> + Send + Sync,
    TClient: super::private::BuildHttpRequestAsync + super::private::AccessTokenExpiryAsync + Send + Sync,
    Error: From<<TReturn as TryFrom<TResponse>>::Error>,
{
    /// Send the request asynchronously and process the response, extracting the result object from the body.
    async fn send_async(self) -> Result<TReturn> {
        let common = self.take_base_builder();
        let url = common.build_url();

        loop {
            let mut request = common.client.build_http_request(common.method.clone(), url.clone());

            if let Some(body) = &common.body {
                trace!("Request body: {:?}", body);
                request = request.json(body);
            } else {
                // Spotify requires that all empty POST and PUT requests have Content-Length set to 0. I've previously
                // supposedly observed that reqwest doesn't set Content-Length, even when there's a body, so we have to
                // set it ourselves when there's an empty body. in hindsight it seems silly reqwest doesn't set
                // Content-Length but I guess it makes sense if it's streaming the body or smth. setting a default
                // Content-Length to 0 for every request also doesn't work since then it's set to 0 even when there's a
                // body, which causes issues
                if common.method == Method::POST || common.method == Method::PUT {
                    request = request.header(header::CONTENT_LENGTH, header::HeaderValue::from_static("0"));
                }
            }

            let response = request.send().await?;

            match response.status() {
                StatusCode::BAD_REQUEST => {
                    error!("Got 400 Bad Request response");
                    let error_response = response.text().await?;
                    warn!("Error response: {error_response}");

                    return Err(Error::UnhandledSpotifyResponseStatusCode(400));
                }

                StatusCode::FORBIDDEN => {
                    error!("Got 403 Forbidden response");
                    let error_response: ApiErrorResponse = response.json().await?;
                    handle_403_forbidden_api_response(error_response)?
                }

                StatusCode::UNAUTHORIZED => {
                    warn!("Got 401 Unauthorized response");
                    let error_response = response.json().await?;
                    is_api_error_expired_access_token(error_response)?;

                    // is_api_error_expired_access_token handles all other errors except the access token being expired
                    if !common.auto_refresh_access_token
                        || common.client.handle_access_token_expired().await? == AccessTokenExpiryResult::Inapplicable
                    {
                        warn!(
                            "Refreshing access tokens is disabled for this request, or is inapplicable to this client"
                        );

                        return Err(Error::AccessTokenExpired);
                    }
                }

                StatusCode::TOO_MANY_REQUESTS => {
                    let headers = response.headers();
                    let retry_after = extract_rate_limit_retry_after(headers)?;

                    if common.react_to_rate_limit {
                        info!("Got rate limited, waiting {retry_after} seconds...");
                        super::rate_limit_sleep_async(retry_after).await?;
                    } else {
                        warn!("Got rate limited {retry_after} seconds and reacting to rate limits is disabled");
                        return Err(Error::RateLimit(retry_after));
                    }
                }

                _ => {
                    let response = (common.async_response_handler)(response).await;
                    trace!("Handled response: {response:?}");

                    let response = response?;

                    // bypass serialization for 204 responses, since it's possible the return type is the unit type, but
                    // serde/serde_json won't deserialize the unit type from an empty string, instead failing with an
                    // EOF error
                    let response_body = if response.status() == StatusCode::NO_CONTENT {
                        TResponse::try_from_empty_response()?
                    } else {
                        response.json().await?
                    };

                    trace!("Body: {response_body:?}");
                    return Ok(response_body.try_into()?);
                }
            }
        }
    }
}

/// Synchronous request builder functionality, namely sending the request and processing its response synchronously.
#[cfg(feature = "sync")]
pub trait SyncRequestBuilder<TClient, TResponse, TBody, TReturn>
where
    Self: BaseRequestBuilder<TClient, TResponse, TBody, TReturn>,
    TBody: Debug + Serialize,
    TResponse: Debug + DeserializeOwned + TryFromEmptyResponse,
    TReturn: TryFrom<TResponse>,
    TClient: super::private::BuildHttpRequestSync + super::private::AccessTokenExpirySync,
    Error: From<<TReturn as TryFrom<TResponse>>::Error>,
{
    /// Send the request synchronously and process the response, extracting the result object from the body.
    fn send_sync(self) -> Result<TReturn> {
        let common = self.take_base_builder();
        let url = common.build_url();

        loop {
            let mut request = common.client.build_http_request(common.method.clone(), url.clone());

            if let Some(body) = &common.body {
                trace!("Request body: {:?}", body);
                request = request.json(body);
            } else {
                // Spotify requires that all empty POST and PUT requests have Content-Length set to 0. I've previously
                // supposedly observed that reqwest doesn't set Content-Length, even when there's a body, so we have to
                // set it ourselves when there's an empty body. in hindsight it seems silly reqwest doesn't set
                // Content-Length but I guess it makes sense if it's streaming the body or smth. setting a default
                // Content-Length to 0 for every request also doesn't work since then it's set to 0 even when there's a
                // body, which causes issues
                if common.method == Method::POST || common.method == Method::PUT {
                    request = request.header(header::CONTENT_LENGTH, header::HeaderValue::from_static("0"));
                }
            }

            let response = request.send()?;

            match response.status() {
                StatusCode::BAD_REQUEST => {
                    error!("Got 400 Bad Request response");
                    let error_response = response.text()?;
                    warn!("Error response: {error_response}");

                    return Err(Error::UnhandledSpotifyResponseStatusCode(400));
                }

                StatusCode::FORBIDDEN => {
                    error!("Got 403 Forbidden response");
                    let error_response: ApiErrorResponse = response.json()?;
                    handle_403_forbidden_api_response(error_response)?
                }

                StatusCode::UNAUTHORIZED => {
                    warn!("Got 401 Unauthorized response");
                    let error_response = response.json()?;
                    warn!("Error response: {error_response:?}");

                    is_api_error_expired_access_token(error_response)?;

                    // is_api_error_expired_access_token handles all other errors except the access token being expired
                    if !common.auto_refresh_access_token
                        || common.client.handle_access_token_expired()? == AccessTokenExpiryResult::Inapplicable
                    {
                        warn!(
                            "Refreshing access tokens is disabled for this request, or is inapplicable to this client"
                        );

                        return Err(Error::AccessTokenExpired);
                    }
                }

                StatusCode::TOO_MANY_REQUESTS => {
                    let headers = response.headers();
                    let retry_after = extract_rate_limit_retry_after(headers)?;

                    if common.react_to_rate_limit {
                        info!("Got rate limited, waiting {retry_after} seconds...");
                        super::rate_limit_sleep_sync(retry_after)?;
                    } else {
                        warn!("Got rate limited ({retry_after}) and reacting to rate limits is disabled");
                        return Err(Error::RateLimit(retry_after));
                    }
                }

                _ => {
                    let response = (common.sync_response_handler)(response);
                    trace!("Handled response: {response:?}");

                    let response = response?;

                    // bypass serialization for 204 responses, since it's possible the return type is the unit type, but
                    // serde/serde_json won't deserialize the unit type from an empty string, instead failing with an
                    // EOF error
                    let response_body = if response.status() == StatusCode::NO_CONTENT {
                        TResponse::try_from_empty_response()?
                    } else {
                        response.json()?
                    };

                    trace!("Body: {response_body:?}");
                    return Ok(response_body.try_into()?);
                }
            }
        }
    }
}

/// A "base" request builder that doesn't include any special functionality. The commonly available options are
/// available in the [BaseRequestBuilder]-trait. Asynchronous builders implement the [AsyncRequestBuilder]-trait for
/// sending the request and retrieving its response. Synchronous builders implement the [SyncRequestBuilder]-trait for
/// the same functionality.
pub struct RequestBuilder<TClient, TResponse, TBody = (), TReturn = TResponse> {
    client: TClient,
    method: Method,
    base_url: Cow<'static, str>,
    query_params: HashMap<&'static str, Cow<'static, str>>,
    body: Option<TBody>,

    #[cfg(feature = "async")]
    async_response_handler: AsyncResponseHandler,
    #[cfg(feature = "sync")]
    sync_response_handler: SyncResponseHandler,

    react_to_rate_limit: bool,
    auto_refresh_access_token: bool,

    phantom: PhantomData<(TReturn, TResponse)>,
}

impl<TClient, TResponse, TBody, TReturn> RequestBuilder<TClient, TResponse, TBody, TReturn> {
    fn build_url(&self) -> Url {
        Url::parse_with_params(&self.base_url, &self.query_params)
            .unwrap_or_else(|_| panic!("failed to build URL from base: {}", self.base_url))
    }
}

impl<TClient, TResponse, TBody, TReturn> private::BaseRequestBuilderContainer<TClient, TResponse, TBody, TReturn>
    for RequestBuilder<TClient, TResponse, TBody, TReturn>
{
    fn new<S>(method: Method, base_url: S, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            client,
            method,
            base_url: base_url.into(),
            query_params: HashMap::new(),
            body: None,

            #[cfg(feature = "async")]
            async_response_handler: Box::new(async_response_handler_noop),
            #[cfg(feature = "sync")]
            sync_response_handler: Box::new(sync_response_handler_noop),

            react_to_rate_limit: true,
            auto_refresh_access_token: true,

            phantom: PhantomData,
        }
    }

    fn new_with_body<S>(method: Method, base_url: S, body: TBody, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            body: Some(body),
            ..Self::new(method, base_url, client)
        }
    }

    fn take_base_builder(self) -> RequestBuilder<TClient, TResponse, TBody, TReturn> {
        self
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TClient, TResponse, TBody, TReturn> {
        self
    }
}

impl<TBuilder, TClient, TResponse, TBody, TReturn> BaseRequestBuilder<TClient, TResponse, TBody, TReturn> for TBuilder where
    TBuilder: BaseRequestBuilderContainer<TClient, TResponse, TBody, TReturn>
{
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl<TBuilder, TClient, TResponse, TBody, TReturn> AsyncRequestBuilder<TClient, TResponse, TBody, TReturn> for TBuilder
where
    TBuilder: BaseRequestBuilder<TClient, TResponse, TBody, TReturn>,
    TBody: Debug + Serialize + Send,
    TResponse: Debug + DeserializeOwned + TryFromEmptyResponse + Send + Sync,
    TReturn: TryFrom<TResponse> + Send + Sync,
    TClient: super::private::BuildHttpRequestAsync + super::private::AccessTokenExpiryAsync + Send + Sync,
    Error: From<<TReturn as TryFrom<TResponse>>::Error>,
{
}

#[cfg(feature = "sync")]
impl<TBuilder, TClient, TResponse, TBody, TReturn> SyncRequestBuilder<TClient, TResponse, TBody, TReturn> for TBuilder
where
    TBuilder: BaseRequestBuilder<TClient, TResponse, TBody, TReturn>,
    TBody: Debug + Serialize,
    TResponse: Debug + DeserializeOwned + TryFromEmptyResponse,
    TReturn: TryFrom<TResponse>,
    TClient: super::private::BuildHttpRequestSync + super::private::AccessTokenExpirySync,
    Error: From<<TReturn as TryFrom<TResponse>>::Error>,
{
}
