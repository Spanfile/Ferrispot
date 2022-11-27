//! Clients for every OAuth2 authorization flow Spotify supports.
//!
//! # Which authorization flow to use?
//!
//! - If you require access to user-specific scoped endpoints, or cannot store the application client secret safely, use
//!   the [authorization code flow client](authorization_code). The client supports PKCE if the application client
//!   secret is unavailable.
//! - If you do not require access to user-specific scoped endpoints and have the application client secret available,
//!   use the [client credentials flow client](SpotifyClientWithSecret).
//!
//! Every client requires an application client ID. You can create a new application in the
//! [Spotify developer dashboard](https://developer.spotify.com/dashboard), from where you get the application's client
//! ID and secret. If the secret can be safely stored in your environment, you may use the client credentials flow by
//! building a [SpotifyClientWithSecret] which can access all [unscoped endpoints](UnscopedClient). From there, you can
//! retrieve a user-authorized [authorization code flow client](authorization_code) which can access all [scoped
//! endpoints](ScopedClient) in addition to the [unscoped endpoints](UnscopedClient).
//!
//! However, if the client secret cannot be safely stored in your environment, you may still access all
//! [unscoped](UnscopedClient) and [scoped endpoints](ScopedClient) by using the [authorization code flow with
//! PKCE](SpotifyClient::authorization_code_client_with_pkce). The [implicit grant flow is also
//! supported](SpotifyClient::implicit_grant_client), but it is not recommended for use.
//!
//! [Spotify documentation on authorization.](https://developer.spotify.com/documentation/general/guides/authorization/)

// TODO: this table would be really neat to have if rustfmt didn't mess it up
// | Authorization flow | [Access user resources](ScopedClient) | Requires secret key | [Access token
// refresh](AccessTokenRefresh) | |-|-|-|-|
// | [AuthorizationCodeUserClient with PKCE](authorization_code) | Yes | No | Yes |
// | [AuthorizationCodeUserClient](authorization_code) | Yes | Yes | Yes |
// | [ImplicitGrantUserClient](implicit_grant) | Yes | No | No |
// | [Client credentials](SpotifyClientWithSecret) | No | Yes | Yes |

pub mod authorization_code;
pub mod implicit_grant;

pub(crate) mod scoped;
pub(crate) mod unscoped;

pub(crate) mod private {
    use async_trait::async_trait;
    use log::{error, warn};
    use reqwest::{header, Method, RequestBuilder, Response, StatusCode, Url};
    use serde::Serialize;

    use crate::{
        error::{Error, Result},
        model::error::{ApiErrorMessage, ApiErrorResponse},
    };

    /// Every Spotify client implements this trait.
    pub trait BuildHttpRequest: crate::private::Sealed {
        /// Returns a new [RequestBuilder](reqwest::RequestBuilder) with any necessary information (e.g. authentication
        /// headers) filled in. You probably shouldn't call this function directly; instead use
        /// [send_http_request](crate::client::private::SendHttpRequest::send_http_request).
        fn build_http_request(&self, method: Method, url: Url) -> RequestBuilder;
    }

    /// Every Spotify client implements this trait.
    pub trait SendHttpRequest<'a>: BuildHttpRequest + AccessTokenExpiry
    where
        Self: 'a,
    {
        /// Builds an HTTP request, sends it, and handles rate limiting and possible access token refreshes.
        fn send_http_request(&'a self, method: Method, url: Url) -> PrivateRequestBuilder<'a, Self, ()>;
    }

    /// Every Spotify client implements this trait.
    #[async_trait]
    pub trait AccessTokenExpiry: crate::private::Sealed {
        // if specialisation was a thing, this function could be refactored into two generic trait impls
        async fn handle_access_token_expired(&self) -> Result<AccessTokenExpiryResult>;
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
        C: BuildHttpRequest + ?Sized,
        B: Serialize + Send,
    {
        client: &'a C,
        method: Method,
        url: Url,
        body: Option<B>,
    }

    impl<'a, C> SendHttpRequest<'a> for C
    where
        C: BuildHttpRequest + AccessTokenExpiry + Sync + 'a,
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
        C: BuildHttpRequest + AccessTokenExpiry + ?Sized,
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

        pub async fn send(self) -> Result<Response> {
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
                        let error_response: ApiErrorResponse = response.json().await?;

                        match error_response.error.message {
                            ApiErrorMessage::PermissionsMissing => {
                                error!("Missing required scope for the endpoint");
                                return Err(Error::MissingScope);
                            }

                            ApiErrorMessage::TokenExpired => {
                                warn!("Access token expired, attempting to refresh");

                                if self.client.handle_access_token_expired().await?
                                    == AccessTokenExpiryResult::Inapplicable
                                {
                                    warn!("Refreshing access tokens is inapplicable to this client");
                                    return Err(Error::AccessTokenExpired);
                                }
                            }

                            other => {
                                error!("Unexpected Spotify error: {:?}", other);
                                return Err(Error::UnhandledSpotifyError(401, format!("{:?}", other)));
                            }
                        }
                    }

                    StatusCode::TOO_MANY_REQUESTS => {
                        let headers = response.headers();
                        if let Some(wait_time) = headers
                            .get(header::RETRY_AFTER)
                            .and_then(|header| header.to_str().ok())
                            .and_then(|header_str| header_str.parse::<u64>().ok())
                        {
                            warn!(
                                "Got 429 rate-limit response from Spotify with Retry-After: {}",
                                wait_time
                            );

                            super::rate_limit_sleep(wait_time).await?;
                        } else {
                            warn!("Invalid rate-limit response");
                            return Err(Error::InvalidRateLimitResponse);
                        }
                    }

                    // all other responses, even erroneous ones, are returned to the caller
                    _ => return Ok(response),
                }
            }
        }
    }
}

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use const_format::concatcp;
use log::debug;
use reqwest::{header, Client as AsyncClient, Method, RequestBuilder, StatusCode, Url};
use serde::Deserialize;

use self::{
    authorization_code::{AuthorizationCodeUserClient, AuthorizationCodeUserClientBuilder},
    implicit_grant::ImplicitGrantUserClientBuilder,
};
pub use self::{
    scoped::ScopedClient,
    unscoped::{SearchBuilder, UnscopedClient},
};
use crate::{
    error::{Error, Result},
    model::error::{AuthenticationErrorKind, AuthenticationErrorResponse},
};

const RANDOM_STATE_LENGTH: usize = 16;
const PKCE_VERIFIER_LENGTH: usize = 128; // maximum Spotify allows

const API_BASE_URL: &str = "https://api.spotify.com/v1/";

// unscoped endpoints
const API_TRACKS_ENDPOINT: &str = concatcp!(API_BASE_URL, "tracks");
const API_SEARCH_ENDPOINT: &str = concatcp!(API_BASE_URL, "search");

// scoped endpoints
const API_PLAYBACK_STATE_ENDPOINT: &str = concatcp!(API_BASE_URL, "me/player");
const API_CURRENTLY_PLAYING_TRACK_ENDPOINT: &str = concatcp!(API_BASE_URL, "me/player/currently-playing");
const API_PLAYER_PLAY_ENDPOINT: &str = concatcp!(API_BASE_URL, "me/player/play");
const API_PLAYER_PAUSE_ENDPOINT: &str = concatcp!(API_BASE_URL, "me/player/pause");
const API_PLAYER_REPEAT_ENDPOINT: &str = concatcp!(API_BASE_URL, "me/player/repeat");
const API_PLAYER_SHUFFLE_ENDPOINT: &str = concatcp!(API_BASE_URL, "me/player/shuffle");
const API_PLAYER_VOLUME_ENDPOINT: &str = concatcp!(API_BASE_URL, "me/player/volume");
const API_PLAYER_QUEUE_ENDPOINT: &str = concatcp!(API_BASE_URL, "me/player/queue");
const API_PLAYER_DEVICES_ENDPOINT: &str = concatcp!(API_BASE_URL, "me/player/devices");

// accounts
const ACCOUNTS_BASE_URL: &str = "https://accounts.spotify.com/";
const ACCOUNTS_AUTHORIZE_ENDPOINT: &str = concatcp!(ACCOUNTS_BASE_URL, "authorize");
const ACCOUNTS_API_TOKEN_ENDPOINT: &str = concatcp!(ACCOUNTS_BASE_URL, "api/token");

/// Clients that have automatically refreshable access tokens implement this trait.
/// [SpotifyClientWithSecret](SpotifyClientWithSecret) and
/// [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient) implement this trait.
///
/// Note that [ImplicitGrantUserClient](implicit_grant::ImplicitGrantUserClient) does *not* implement
/// this trait, since even though it has an access token, it cannot be automatically refreshed.
#[async_trait]
pub trait AccessTokenRefresh: crate::private::Sealed {
    /// Request a new access token from Spotify and save it internally in the client.
    async fn refresh_access_token(&self) -> Result<()>;
}

/// A base Spotify client that does *not* have a client secret.
///
/// This client by itself cannot be used to access the Spotify API, since it has no way of authenticating itself to the
/// API. However, it can be used to retrieve either user-authenticated client; [AuthorizationCodeUserClient with
/// PKCE](authorization_code::AuthorizationCodeUserClient) or an
/// [ImplicitGrantUserClient](implicit_grant::ImplicitGrantUserClient).
///
/// This client uses `Arc` and interior mutability internally, so you do not need to wrap it in an `Arc` or a `Mutex` in
/// order to reuse it.
#[derive(Debug, Clone)]
pub struct SpotifyClient {
    inner: Arc<SpotifyClientRef>,
    http_client: AsyncClient,
}

#[derive(Debug)]
struct SpotifyClientRef {
    client_id: String,
}

/// A base Spotify client that has a client secret.
///
/// This client can be used to access all [unscoped Spotify endpoints](UnscopedClient). It can also be used to retrieve
/// an user-authenticated [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient) that can access
/// all [scoped endpoints](ScopedClient).
///
/// This client uses `Arc` and interior mutability internally, so you do not need to wrap it in an `Arc` or a `Mutex` in
/// order to reuse it.
#[derive(Debug, Clone)]
pub struct SpotifyClientWithSecret {
    inner: Arc<SpotifyClientWithSecretRef>,
    http_client: AsyncClient,
}

#[derive(Debug)]
struct SpotifyClientWithSecretRef {
    client_id: String,
    // there's no use to store the client secret here (it's already in the HTTP client), but there might be in the
    // future
    // client_secret: String,
    access_token: RwLock<String>,
}

/// Builder for [SpotifyClient](SpotifyClient).
#[derive(Debug, Clone)]
pub struct SpotifyClientBuilder {
    client_id: String,
}

/// Builder for [SpotifyClientWithSecret](SpotifyClientWithSecret).
#[derive(Debug, Clone)]
pub struct SpotifyClientWithSecretBuilder {
    client_id: String,
    client_secret: String,
}

#[derive(Debug, Deserialize)]
struct ClientTokenResponse {
    access_token: String,

    // these fields are in the response but the library doesn't need them. keep them here for logging purposes
    #[allow(dead_code)]
    token_type: String,
    #[allow(dead_code)]
    expires_in: u32,
}

impl SpotifyClient {
    /// Returns a new builder for an [ImplicitGrantUserClient](implicit_grant::ImplicitGrantUserClient).
    ///
    /// # Note
    ///
    /// The implicit grant user client is not recommended for use. The access token is returned in the callback URL
    /// instead of through a trusted channel, and the token cannot be automatically refreshed. It is recommended to use
    /// the [authorization code flow with PKCE flow](SpotifyClient::authorization_code_client_with_pkce) instead.
    pub fn implicit_grant_client<S>(&self, redirect_uri: S) -> ImplicitGrantUserClientBuilder
    where
        S: Into<String>,
    {
        ImplicitGrantUserClientBuilder::new(redirect_uri.into(), Arc::clone(&self.inner), self.http_client.clone())
    }

    /// Returns a new builder for an [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient)
    /// that uses PKCE.
    ///
    /// PKCE is required for strong authentication when the client secret cannot be securely stored in the environment.
    pub fn authorization_code_client_with_pkce<S>(&self, redirect_uri: S) -> AuthorizationCodeUserClientBuilder
    where
        S: Into<String>,
    {
        AuthorizationCodeUserClientBuilder::new(
            redirect_uri.into(),
            self.inner.client_id.clone(),
            self.http_client.clone(),
        )
        .with_pkce()
    }

    /// Returns a new [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient) that uses PKCE and
    /// an existing refresh token.
    ///
    /// The refresh token will be used to retrieve a new access token before the client is returned. PKCE is required
    /// for strong authentication when the client secret cannot be securely stored in the environment.
    pub async fn authorization_code_client_with_refresh_token_and_pkce<S>(
        &self,
        refresh_token: S,
    ) -> Result<AuthorizationCodeUserClient>
    where
        S: Into<String>,
    {
        AuthorizationCodeUserClient::new_with_refresh_token(
            self.http_client.clone(),
            refresh_token.into(),
            Some(self.inner.client_id.clone()),
        )
        .await
    }
}

impl SpotifyClientWithSecret {
    /// Returns a new builder for an
    /// [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient).
    pub fn authorization_code_client<S>(&self, redirect_uri: S) -> AuthorizationCodeUserClientBuilder
    where
        S: Into<String>,
    {
        AuthorizationCodeUserClientBuilder::new(
            redirect_uri.into(),
            self.inner.client_id.clone(),
            self.http_client.clone(),
        )
    }

    /// Returns a new [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient) that
    /// uses an existing refresh token.
    ///
    /// The refresh token will be used to retrieve a new access token before the client is returned.
    pub async fn authorization_code_client_with_refresh_token<S>(
        &self,
        refresh_token: S,
    ) -> Result<AuthorizationCodeUserClient>
    where
        S: Into<String>,
    {
        AuthorizationCodeUserClient::new_with_refresh_token(self.http_client.clone(), refresh_token.into(), None).await
    }
}

impl SpotifyClientBuilder {
    pub fn new<S>(client_id: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            client_id: client_id.into(),
        }
    }

    pub fn client_secret<S>(self, client_secret: S) -> SpotifyClientWithSecretBuilder
    where
        S: Into<String>,
    {
        SpotifyClientWithSecretBuilder {
            client_id: self.client_id,
            client_secret: client_secret.into(),
        }
    }

    pub fn build(self) -> SpotifyClient {
        SpotifyClient {
            inner: Arc::new(SpotifyClientRef {
                client_id: self.client_id,
            }),
            http_client: AsyncClient::new(),
        }
    }
}

impl SpotifyClientWithSecretBuilder {
    fn get_async_http_client(&self) -> AsyncClient {
        let mut default_headers = header::HeaderMap::new();
        default_headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&build_authorization_header(&self.client_id, &self.client_secret))
                // this can only fail if the header value contains non-ASCII characters, which cannot happen since the
                // given header value is in base64
                .expect(
                    "failed to insert authorization header into header map: non-ASCII characters in value (this is \
                     likely a bug)",
                ),
        );

        AsyncClient::builder()
            .default_headers(default_headers)
            .build()
            // this can only fail due to a system error or system misconfiguration
            .expect("failed to build blocking HTTP client: system error or system misconfiguration")
    }

    pub async fn build(self) -> Result<SpotifyClientWithSecret> {
        debug!("Requesting access token for client credentials flow");
        let token_request_form = &[("grant_type", "client_credentials")];

        let http_client = self.get_async_http_client();
        let response = http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(token_request_form)
            .send()
            .await?;

        let response = extract_authentication_error(response).await.map_err(|err| {
            if let Error::UnhandledAuthenticationError(AuthenticationErrorKind::InvalidClient, description) = err {
                Error::InvalidClient(description)
            } else {
                err
            }
        })?;

        let token_response: ClientTokenResponse = response.json().await?;
        debug!("Got token response for client credentials flow: {:?}", token_response);

        Ok(SpotifyClientWithSecret {
            inner: Arc::new(SpotifyClientWithSecretRef {
                client_id: self.client_id,
                // client_secret: self.client_secret,
                access_token: RwLock::new(token_response.access_token),
            }),
            http_client,
        })
    }
}

impl crate::private::Sealed for SpotifyClientWithSecret {}

impl private::BuildHttpRequest for SpotifyClientWithSecret {
    fn build_http_request(&self, method: Method, url: Url) -> RequestBuilder {
        let access_token = self.inner.access_token.read().expect("access token rwlock poisoned");
        self.http_client.request(method, url).bearer_auth(access_token.as_str())
    }
}

#[async_trait]
impl<'a> UnscopedClient<'a> for SpotifyClientWithSecret {}

#[async_trait]
impl AccessTokenRefresh for SpotifyClientWithSecret {
    async fn refresh_access_token(&self) -> Result<()> {
        debug!("Refreshing access token for client credentials flow");
        let token_request_form = &[("grant_type", "client_credentials")];

        let response = self
            .http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(token_request_form)
            .send()
            .await?;

        let response = extract_authentication_error(response).await.map_err(|err| {
            if let Error::UnhandledAuthenticationError(AuthenticationErrorKind::InvalidGrant, description) = err {
                Error::InvalidClient(description)
            } else {
                err
            }
        })?;

        let token_response: ClientTokenResponse = response.json().await?;
        debug!("Got token response for client credentials flow: {:?}", token_response);

        *self.inner.access_token.write().expect("access token rwlock poisoned") = token_response.access_token;

        Ok(())
    }
}

#[async_trait]
impl private::AccessTokenExpiry for SpotifyClientWithSecret {
    async fn handle_access_token_expired(&self) -> Result<private::AccessTokenExpiryResult> {
        self.refresh_access_token().await?;
        Ok(private::AccessTokenExpiryResult::Ok)
    }
}

fn build_authorization_header(client_id: &str, client_secret: &str) -> String {
    let auth = format!("{}:{}", client_id, client_secret);
    format!("Basic {}", base64::encode(auth))
}

/// Takes a response for an authentication request and if its status is 400, parses its body as an authentication error.
/// On success returns the given response without modifying it.
async fn extract_authentication_error(response: reqwest::Response) -> Result<reqwest::Response> {
    if let StatusCode::BAD_REQUEST = response.status() {
        let error_response: AuthenticationErrorResponse = response.json().await?;
        Err(error_response.into_unhandled_error())
    } else {
        Ok(response)
    }
}

/// Return a rate limit error since no sleep utility has been enabled.
#[cfg(all(not(feature = "tokio_sleep"), not(feature = "async_std_sleep")))]
async fn rate_limit_sleep(sleep_time: u64) -> Result<()> {
    Err(crate::error::Error::RateLimit(sleep_time))
}

// sleeping with tokio takes precedence over async_std so if the user enables both features for some reason, they get
// tokio sleep
/// Sleep for the specified amount of time using tokio's sleep function.
#[cfg(feature = "tokio_sleep")]
async fn rate_limit_sleep(sleep_time: u64) -> Result<()> {
    tokio::time::sleep(std::time::Duration::from_secs(sleep_time)).await;
    Ok(())
}

/// Sleep for the specified amount of time using async_std's sleep function.
#[cfg(all(feature = "async_std_sleep", not(feature = "tokio_sleep")))]
async fn rate_limit_sleep(sleep_time: u64) -> Result<()> {
    async_std::task::sleep(std::time::Duration::from_secs(sleep_time)).await;
    Ok(())
}
