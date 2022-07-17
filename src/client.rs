pub(crate) mod authorization_code;
pub(crate) mod implicit_grant;
pub(crate) mod scoped;
pub(crate) mod unscoped;

pub(crate) mod private {
    use crate::{
        error::{Error, Result},
        model::error::{ApiErrorMessage, ApiErrorResponse},
    };
    use async_trait::async_trait;
    use log::{error, warn};
    use reqwest::{header, Method, RequestBuilder, Response, StatusCode, Url};

    pub trait Sealed {}

    /// Marker trait for signifying a Spotify client that includes user authentication;
    /// [AuthorizationCodeUserClient](crate::client::AuthorizationCodeUserClient) and
    /// [ImplicitGrantUserClient](crate::client::ImplicitGrantUserClient). This is used to separate the clients into
    /// [scoped clients](crate::client::ScopedClient) and [unscoped clients](crate::client::UnscopedClient).
    pub trait UserAuthenticatedClient: Sealed {}

    /// Every Spotify client implement this trait.
    pub trait BuildHttpRequest: Sealed {
        /// Returns a new [RequestBuilder](reqwest::RequestBuilder) with any necessary information (e.g. authentication
        /// headers) filled in. You probably shouldn't call this function directly; instead use
        /// [send_http_request](crate::client::private::SendHttpRequest::send_http_request).
        fn build_http_request(&self, method: Method, url: Url) -> RequestBuilder;
    }

    /// Every Spotify client implement this trait.
    #[async_trait]
    pub trait SendHttpRequest: BuildHttpRequest {
        /// Builds an HTTP request, sends it, and handles rate limiting and possible access token refreshes.
        async fn send_http_request(&self, method: Method, url: Url) -> Result<reqwest::Response>;
    }

    /// Every Spotify client implements this trait.
    #[async_trait]
    pub trait AccessTokenExpiry: Sealed {
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

    #[async_trait]
    impl<C> SendHttpRequest for C
    where
        C: BuildHttpRequest + AccessTokenExpiry + Sync,
    {
        async fn send_http_request(&self, method: Method, url: Url) -> Result<Response> {
            loop {
                let request = self.build_http_request(method.clone(), url.clone());
                let response = request.send().await?;

                match response.status() {
                    StatusCode::FORBIDDEN => {
                        error!("Got 403 Forbidden response");
                        return Err(Error::Forbidden);
                    }

                    StatusCode::UNAUTHORIZED => {
                        warn!("Got 401 Unauthorized response");
                        let error_response: ApiErrorResponse = response.json().await?;

                        match error_response.message {
                            ApiErrorMessage::PermissionsMissing => {
                                error!("Missing required scope for the endpoint");
                                return Err(Error::MissingScope);
                            }

                            ApiErrorMessage::TokenExpired => {
                                warn!("Access token expired, attempting to refresh");

                                if self.handle_access_token_expired().await? == AccessTokenExpiryResult::Inapplicable {
                                    warn!("Refreshing access tokens is inapplicable to this client");
                                    return Err(Error::AccessTokenExpired);
                                }
                            }

                            ApiErrorMessage::Other(message) => {
                                error!("Unhandled Spotify error: {}", message);
                                return Err(Error::UnhandledSpotifyError(401, message));
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

                    _ => return Ok(response),
                }
            }
        }
    }
}

pub use self::{
    authorization_code::{
        AuthorizationCodeUserClient, AuthorizationCodeUserClientBuilder, IncompleteAuthorizationCodeUserClient,
    },
    implicit_grant::{ImplicitGrantUserClient, ImplicitGrantUserClientBuilder, IncompleteImplicitGrantUserClient},
    scoped::ScopedClient,
    unscoped::{SearchBuilder, UnscopedClient},
};

use crate::{
    error::{Error, Result},
    model::error::{AuthenticationErrorKind, AuthenticationErrorResponse},
};
use async_trait::async_trait;
use const_format::concatcp;
use log::debug;
use reqwest::{header, Client as AsyncClient, Method, RequestBuilder, StatusCode, Url};
use serde::Deserialize;
use std::sync::{Arc, RwLock};

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

const ACCOUNTS_BASE_URL: &str = "https://accounts.spotify.com/";
const ACCOUNTS_AUTHORIZE_ENDPOINT: &str = concatcp!(ACCOUNTS_BASE_URL, "authorize");
const ACCOUNTS_API_TOKEN_ENDPOINT: &str = concatcp!(ACCOUNTS_BASE_URL, "api/token");

/// Clients that have automatically refreshable access tokens implement this trait.
///
/// These are [SpotifyClientWithSecret](crate::client::SpotifyClientWithSecret) and
/// [AuthorizationCodeUserClient](crate::client::AuthorizationCodeUserClient). Note that
/// [ImplicitGrantUserClient](crate::client::ImplicitGrantUserClient) does *not* implement this trait, since even though
/// it has an access token, it cannot be automatically refreshed.
#[async_trait]
pub trait AccessTokenRefresh: private::Sealed {
    /// Request a new access token from Spotify using a refresh token and save it internally in the client.
    async fn refresh_access_token(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct SpotifyClient {
    inner: Arc<SpotifyClientRef>,
    http_client: AsyncClient,
}

#[derive(Debug)]
struct SpotifyClientRef {
    client_id: String,
}

#[derive(Debug, Clone)]
pub struct SpotifyClientWithSecret {
    inner: Arc<SpotifyClientWithSecretRef>,
    http_client: AsyncClient,
}

#[derive(Debug)]
struct SpotifyClientWithSecretRef {
    client_id: String,
    // there's no use for the client secret currently, but there might be in the future
    // client_secret: String,
    access_token: RwLock<String>,
}

#[derive(Debug, Clone)]
pub struct SpotifyClientBuilder {
    client_id: String,
}

#[derive(Debug, Clone)]
pub struct ClientSecretSpotifyClientBuilder {
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
    pub fn implicit_grant_client<S>(&self, redirect_uri: S) -> ImplicitGrantUserClientBuilder
    where
        S: Into<String>,
    {
        ImplicitGrantUserClientBuilder::new(redirect_uri.into(), Arc::clone(&self.inner), self.http_client.clone())
    }

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

    pub fn client_secret<S>(self, client_secret: S) -> ClientSecretSpotifyClientBuilder
    where
        S: Into<String>,
    {
        ClientSecretSpotifyClientBuilder {
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

impl ClientSecretSpotifyClientBuilder {
    fn get_async_http_client(&self) -> AsyncClient {
        let mut default_headers = header::HeaderMap::new();
        default_headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&build_authorization_header(&self.client_id, &self.client_secret))
                // this can only fail if the header value contains non-ASCII characters, which cannot happen since the
                // given header value is in base64
                .expect("failed to insert authorization header into header map"),
        );

        AsyncClient::builder()
            .default_headers(default_headers)
            .build()
            // this can only fail due to a system error, or if called within an async runtime. we cannot detect the
            // latter, so it's up to the library user to be careful about it
            .expect("failed to build blocking HTTP client")
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
            if let Error::UnhandledAuthenticationError(AuthenticationErrorKind::InvalidClient, _) = err {
                Error::InvalidClient
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

impl private::Sealed for SpotifyClientWithSecret {}

impl private::BuildHttpRequest for SpotifyClientWithSecret {
    fn build_http_request(&self, method: Method, url: Url) -> RequestBuilder {
        let access_token = self.inner.access_token.read().expect("access token rwlock poisoned");
        self.http_client.request(method, url).bearer_auth(access_token.as_str())
    }
}

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
                Error::InvalidRefreshToken(description)
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
    format!("Basic {}", base64::encode(&auth))
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
