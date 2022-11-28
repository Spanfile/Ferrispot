#[cfg(feature = "async")]
mod async_client {
    use std::ops::Deref;

    #[derive(Clone)]
    pub struct AsyncClient(pub(crate) reqwest::Client);

    impl super::HttpClient for AsyncClient {
        fn new() -> Self {
            Self(reqwest::Client::new())
        }
    }

    impl Deref for AsyncClient {
        type Target = reqwest::Client;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[cfg(feature = "sync")]
mod sync_client {
    use std::ops::Deref;

    #[derive(Clone)]
    pub struct SyncClient(pub(crate) reqwest::blocking::Client);

    impl super::HttpClient for SyncClient {
        fn new() -> Self {
            Self(reqwest::blocking::Client::new())
        }
    }

    impl Deref for SyncClient {
        type Target = reqwest::blocking::Client;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

use log::{error, warn};
use reqwest::{
    header::{self, HeaderMap},
    Method, StatusCode, Url,
};
use serde::Serialize;

#[cfg(feature = "async")]
pub use self::async_client::AsyncClient;
#[cfg(feature = "sync")]
pub use self::sync_client::SyncClient;
use crate::{
    error::{Error, Result},
    model::error::{ApiErrorMessage, ApiErrorResponse},
};

pub trait HttpClient {
    fn new() -> Self;
}

/// Every Spotify client implements this trait.
#[cfg(feature = "async")]
pub trait BuildHttpRequestAsync: crate::private::Sealed {
    /// Returns a new async [RequestBuilder](reqwest::RequestBuilder) with any necessary information (e.g.
    /// authentication headers) filled in. You probably shouldn't call this function directly; instead use
    /// [send_http_request](crate::client::private::SendHttpRequest::send_http_request).
    fn build_http_request(&self, method: Method, url: Url) -> reqwest::RequestBuilder;
}

/// Every Spotify client implements this trait.
#[cfg(feature = "sync")]
pub trait BuildHttpRequestSync: crate::private::Sealed {
    /// Returns a new async [RequestBuilder](reqwest::blocking::RequestBuilder) with any necessary information (e.g.
    /// authentication headers) filled in. You probably shouldn't call this function directly; instead use
    /// [send_http_request](crate::client::private::SendHttpRequest::send_http_request).
    fn build_http_request(&self, method: Method, url: Url) -> reqwest::blocking::RequestBuilder;
}

/// Every Spotify client implements this trait.
#[cfg(feature = "async")]
pub trait SendHttpRequestAsync<'a>: BuildHttpRequestAsync + AccessTokenExpiryAsync
where
    Self: 'a,
{
    fn send_http_request(&'a self, method: Method, url: Url) -> PrivateRequestBuilder<'a, Self, ()>;
}

/// Every Spotify client implements this trait.
#[cfg(feature = "sync")]
pub trait SendHttpRequestSync<'a>: BuildHttpRequestSync + AccessTokenExpirySync
where
    Self: 'a,
{
    fn send_http_request(&'a self, method: Method, url: Url) -> PrivateRequestBuilder<'a, Self, ()>;
}

/// Every Spotify client implements this trait.
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AccessTokenExpiryAsync: crate::private::Sealed {
    // if specialisation was a thing, this function could be refactored into two generic trait impls
    async fn handle_access_token_expired(&self) -> Result<AccessTokenExpiryResult>;
}

/// Every Spotify client implements this trait.
#[cfg(feature = "sync")]
pub trait AccessTokenExpirySync: crate::private::Sealed {
    fn handle_access_token_expired(&self) -> Result<AccessTokenExpiryResult>;
}

/// Result to having tried to refresh a client's access token.
#[derive(Debug, PartialEq, Eq)]
pub enum AccessTokenExpiryResult {
    /// Refreshing the token succeeded
    Ok,
    /// Refreshing an access token is not applicable to this client
    Inapplicable,
}

pub struct PrivateRequestBuilder<'a, C, B>
where
    C: ?Sized,
    B: Serialize + Send,
{
    client: &'a C,
    method: Method,
    url: Url,
    body: Option<B>,
}

#[cfg(feature = "async")]
impl<'a, C> SendHttpRequestAsync<'a> for C
where
    C: BuildHttpRequestAsync + AccessTokenExpiryAsync + Sync + 'a,
{
    fn send_http_request(&'a self, method: Method, url: Url) -> PrivateRequestBuilder<'a, Self, ()> {
        PrivateRequestBuilder {
            client: self,
            method,
            url,
            body: None,
        }
    }
}

#[cfg(feature = "sync")]
impl<'a, C> SendHttpRequestSync<'a> for C
where
    C: BuildHttpRequestSync + AccessTokenExpirySync + Sync + 'a,
{
    fn send_http_request(&'a self, method: Method, url: Url) -> PrivateRequestBuilder<'a, Self, ()> {
        PrivateRequestBuilder {
            client: self,
            method,
            url,
            body: None,
        }
    }
}

impl<'a, C, B> PrivateRequestBuilder<'a, C, B>
where
    C: ?Sized,
    B: Serialize + Send,
{
    pub fn body<T>(self, body: T) -> PrivateRequestBuilder<'a, C, T>
    where
        T: Serialize + Send,
    {
        PrivateRequestBuilder {
            client: self.client,
            method: self.method,
            url: self.url,
            body: Some(body), // once told me
        }
    }
}

#[cfg(feature = "async")]
impl<'a, C, B> PrivateRequestBuilder<'a, C, B>
where
    C: BuildHttpRequestAsync + AccessTokenExpiryAsync + ?Sized,
    B: Serialize + Send,
{
    pub async fn send_async(self) -> Result<reqwest::Response> {
        loop {
            let mut request = self.client.build_http_request(self.method.clone(), self.url.clone());

            if let Some(body) = &self.body {
                request = request.json(body);
            }

            let response = request.send().await?;

            match response.status() {
                StatusCode::FORBIDDEN => {
                    error!("Got 403 Forbidden response");
                    return Err(Error::Forbidden);
                }

                StatusCode::UNAUTHORIZED => {
                    warn!("Got 401 Unauthorized response");
                    let error_response = response.json().await?;
                    handle_api_error_response(error_response)?;

                    // handle_api_error_response handles all other errors except the access token being expired
                    if self.client.handle_access_token_expired().await? == AccessTokenExpiryResult::Inapplicable {
                        warn!("Refreshing access tokens is inapplicable to this client");
                        return Err(Error::AccessTokenExpired);
                    }
                }

                StatusCode::TOO_MANY_REQUESTS => {
                    let headers = response.headers();
                    let retry_after = extract_rate_limit_retry_after(headers)?;

                    super::rate_limit_sleep_async(retry_after).await?;
                }

                // all other responses, even erroneous ones, are returned to the caller
                _ => return Ok(response),
            }
        }
    }
}

#[cfg(feature = "sync")]
impl<'a, C, B> PrivateRequestBuilder<'a, C, B>
where
    C: BuildHttpRequestSync + AccessTokenExpirySync + ?Sized,
    B: Serialize + Send,
{
    pub fn send_sync(self) -> Result<reqwest::blocking::Response> {
        loop {
            let mut request = self.client.build_http_request(self.method.clone(), self.url.clone());

            if let Some(body) = &self.body {
                request = request.json(body);
            }

            let response = request.send()?;

            match response.status() {
                StatusCode::FORBIDDEN => {
                    error!("Got 403 Forbidden response");
                    return Err(Error::Forbidden);
                }

                StatusCode::UNAUTHORIZED => {
                    warn!("Got 401 Unauthorized response");
                    let error_response = response.json()?;
                    handle_api_error_response(error_response)?;

                    // handle_api_error_response handles all other errors except the access token being expired
                    if self.client.handle_access_token_expired()? == AccessTokenExpiryResult::Inapplicable {
                        warn!("Refreshing access tokens is inapplicable to this client");
                        return Err(Error::AccessTokenExpired);
                    }
                }

                StatusCode::TOO_MANY_REQUESTS => {
                    let headers = response.headers();
                    let retry_after = extract_rate_limit_retry_after(headers)?;

                    super::rate_limit_sleep_sync(retry_after)?;
                }

                // all other responses, even erroneous ones, are returned to the caller
                _ => return Ok(response),
            }
        }
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
            Err(Error::UnhandledSpotifyError(401, format!("{:?}", other)))
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
