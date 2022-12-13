use std::{borrow::Cow, fmt::Debug, marker::PhantomData};
#[cfg(feature = "async")]
use std::{future::Future, pin::Pin};

use log::{error, trace, warn};
use reqwest::{header, header::HeaderMap, Method, StatusCode, Url};
use serde::{de::DeserializeOwned, Serialize};

use self::private::BaseRequestBuilderContainer;
use crate::{
    client::private::AccessTokenExpiryResult,
    error::{Error, Result},
    model::error::{ApiErrorMessage, ApiErrorResponse},
};

pub(crate) mod private {
    use std::borrow::Cow;

    use reqwest::Method;

    #[cfg(feature = "async")]
    use super::AsyncResponseHandler;
    use super::RequestBuilder;
    #[cfg(feature = "sync")]
    use super::SyncResponseHandler;

    pub trait BaseRequestBuilderContainer<TReturn, C, TBody = ()>
    where
        Self: Sized,
    {
        fn new<S>(method: Method, base_url: S, client: C) -> Self
        where
            S: Into<Cow<'static, str>>;

        fn new_with_body<S>(method: Method, base_url: S, body: TBody, client: C) -> Self
        where
            S: Into<Cow<'static, str>>;

        fn take_base_builder(self) -> RequestBuilder<TReturn, C, TBody>;
        fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TReturn, C, TBody>;

        fn append_query<S>(mut self, key: &'static str, value: S) -> Self
        where
            S: Into<Cow<'static, str>>,
        {
            self.get_base_builder_mut().query_params.push((key, value.into()));
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
}

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
    Box::pin(async move { Ok(resp) })
}

#[cfg(feature = "sync")]
fn sync_response_handler_noop(resp: reqwest::blocking::Response) -> Result<reqwest::blocking::Response> {
    Ok(resp)
}

pub trait BaseRequestBuilder<TReturn, C, TBody>
where
    Self: private::BaseRequestBuilderContainer<TReturn, C, TBody> + Sized,
{
    // TODO: implement
    /// Whether or not to react to being rate limited by waiting the wanted time in the response. Defaults to `true`.
    fn react_to_rate_limit(mut self, react_to_rate_limit: bool) -> Self {
        self.get_base_builder_mut().react_to_rate_limit = react_to_rate_limit;
        self
    }

    // TODO: implement
    /// Whether or not to automatically refresh the client's access token, if applicable, when it expires. Defaults to
    /// `true`.
    fn auto_refresh_access_token(mut self, auto_refresh_access_token: bool) -> Self {
        self.get_base_builder_mut().auto_refresh_access_token = auto_refresh_access_token;
        self
    }
}

// TODO: this is a terrible function name
/// Returns Ok if the given API error response is because of an expired token. Else, returns an error based on the API
/// error response.
fn handle_api_error_response(error_response: ApiErrorResponse) -> Result<()> {
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

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncRequestBuilder<TReturn, C, TBody>
where
    Self: BaseRequestBuilder<TReturn, C, TBody>,
    TBody: Serialize + Send,
    TReturn: Debug + DeserializeOwned + Send + Sync,
    C: super::private::BuildHttpRequestAsync + super::private::AccessTokenExpiryAsync + Send + Sync,
{
    async fn send_async(self) -> Result<TReturn> {
        let common = self.take_base_builder();
        let url = common.build_url();

        loop {
            let mut request = common.client.build_http_request(common.method.clone(), url.clone());

            if let Some(body) = &common.body {
                request = request.json(body);
            // Spotify requires that all empty POST and PUT have a Content-Length header set to 0. reqwest doesn't do it
            // so we have to do it ourselves. and before you ask, it cannot be set as a default header in the client
            // because reqwest doesn't seem to set it at all for requests, so setting it to default 0 means it'll always
            // be 0, even if there's a body, which causes issues
            } else if common.method == Method::POST || common.method == Method::PUT {
                request = request.header(header::CONTENT_LENGTH, header::HeaderValue::from_static("0"));
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
                    return Err(Error::Forbidden);
                }

                StatusCode::UNAUTHORIZED => {
                    warn!("Got 401 Unauthorized response");
                    let error_response = response.json().await?;
                    handle_api_error_response(error_response)?;

                    // handle_api_error_response handles all other errors except the access token being expired
                    if common.client.handle_access_token_expired().await? == AccessTokenExpiryResult::Inapplicable {
                        warn!("Refreshing access tokens is inapplicable to this client");
                        return Err(Error::AccessTokenExpired);
                    }
                }

                StatusCode::TOO_MANY_REQUESTS => {
                    let headers = response.headers();
                    let retry_after = extract_rate_limit_retry_after(headers)?;

                    super::rate_limit_sleep_async(retry_after).await?;
                }

                _ => {
                    let response = response.error_for_status().map_err(super::response_to_error);
                    trace!("Response: {response:?}");

                    let response = (common.async_response_handler)(response?).await?;

                    let response_body = response.json().await?;
                    trace!("Body: {response_body:?}");

                    return Ok(response_body);
                }
            }
        }
    }
}

#[cfg(feature = "sync")]
pub trait SyncRequestBuilder<TReturn, C, TBody>
where
    Self: BaseRequestBuilder<TReturn, C, TBody>,
    TBody: Serialize,
    TReturn: Debug + DeserializeOwned,
    C: super::private::BuildHttpRequestSync + super::private::AccessTokenExpirySync,
{
    fn send_sync(self) -> Result<TReturn> {
        let common = self.take_base_builder();
        let url = common.build_url();

        loop {
            let mut request = common.client.build_http_request(common.method.clone(), url.clone());

            if let Some(body) = &common.body {
                request = request.json(body);
            // Spotify requires that all empty POST and PUT have a Content-Length header set to 0. reqwest doesn't do it
            // so we have to do it ourselves. and before you ask, it cannot be set as a default header in the client
            // because reqwest doesn't seem to set it at all for requests, so setting it to default 0 means it'll always
            // be 0, even if there's a body, which causes issues
            } else if common.method == Method::POST || common.method == Method::PUT {
                request = request.header(header::CONTENT_LENGTH, header::HeaderValue::from_static("0"));
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
                    return Err(Error::Forbidden);
                }

                StatusCode::UNAUTHORIZED => {
                    warn!("Got 401 Unauthorized response");
                    let error_response = response.json()?;
                    handle_api_error_response(error_response)?;

                    // handle_api_error_response handles all other errors except the access token being expired
                    if common.client.handle_access_token_expired()? == AccessTokenExpiryResult::Inapplicable {
                        warn!("Refreshing access tokens is inapplicable to this client");
                        return Err(Error::AccessTokenExpired);
                    }
                }

                StatusCode::TOO_MANY_REQUESTS => {
                    let headers = response.headers();
                    let retry_after = extract_rate_limit_retry_after(headers)?;

                    super::rate_limit_sleep_sync(retry_after)?;
                }

                _ => {
                    let response = response.error_for_status().map_err(super::response_to_error);
                    trace!("Response: {response:?}");

                    let response = (common.sync_response_handler)(response?)?;

                    let response_body = response.json()?;
                    trace!("Body: {response_body:?}");

                    return Ok(response_body);
                }
            }
        }
    }
}

pub struct RequestBuilder<TReturn, C, TBody = ()> {
    client: C,
    method: Method,
    base_url: Cow<'static, str>,
    query_params: Vec<(&'static str, Cow<'static, str>)>,
    body: Option<TBody>,

    #[cfg(feature = "async")]
    async_response_handler: AsyncResponseHandler,
    #[cfg(feature = "sync")]
    sync_response_handler: SyncResponseHandler,

    react_to_rate_limit: bool,
    auto_refresh_access_token: bool,

    phantom: PhantomData<TReturn>,
}

impl<TReturn, C, TBody> RequestBuilder<TReturn, C, TBody> {
    fn build_url(&self) -> Url {
        Url::parse_with_params(&self.base_url, &self.query_params)
            .unwrap_or_else(|_| panic!("failed to build URL from base: {}", self.base_url))
    }
}

impl<TReturn, C, TBody> private::BaseRequestBuilderContainer<TReturn, C, TBody> for RequestBuilder<TReturn, C, TBody> {
    fn new<S>(method: Method, base_url: S, client: C) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            client,
            method,
            base_url: base_url.into(),
            query_params: Vec::new(),
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

    fn new_with_body<S>(method: Method, base_url: S, body: TBody, client: C) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            body: Some(body),
            ..Self::new(method, base_url, client)
        }
    }

    fn take_base_builder(self) -> RequestBuilder<TReturn, C, TBody> {
        self
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TReturn, C, TBody> {
        self
    }
}

impl<TBuilder, TReturn, C, TBody> BaseRequestBuilder<TReturn, C, TBody> for TBuilder where
    TBuilder: BaseRequestBuilderContainer<TReturn, C, TBody>
{
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl<TBuilder, TReturn, C, TBody> AsyncRequestBuilder<TReturn, C, TBody> for TBuilder
where
    TBuilder: BaseRequestBuilder<TReturn, C, TBody>,
    TBody: Serialize + Send,
    TReturn: Debug + DeserializeOwned + Send + Sync,
    C: for<'a> super::private::SendHttpRequestAsync<'a> + super::private::AccessTokenExpiryAsync + Send + Sync,
{
}

#[cfg(feature = "sync")]
impl<TBuilder, TReturn, C, TBody> SyncRequestBuilder<TReturn, C, TBody> for TBuilder
where
    TBuilder: BaseRequestBuilder<TReturn, C, TBody>,
    TBody: Serialize,
    TReturn: Debug + DeserializeOwned,
    C: for<'a> super::private::SendHttpRequestSync<'a> + super::private::AccessTokenExpirySync,
{
}
