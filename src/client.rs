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
//! building a [SpotifyClientWithSecret] which can access all [unscoped endpoints](UnscopedAsyncClient). From there, you
//! can retrieve a user-authorized [authorization code flow client](authorization_code) which can access all [scoped
//! endpoints](ScopedAsyncClient) in addition to the [unscoped endpoints](UnscopedAsyncClient).
//!
//! However, if the client secret cannot be safely stored in your environment, you may still access all
//! [unscoped](UnscopedAsyncClient) and [scoped endpoints](ScopedAsyncClient) by using the [authorization code flow with
//! PKCE](SpotifyClient::authorization_code_client_with_pkce). The [implicit grant flow is also
//! supported](SpotifyClient::implicit_grant_client), but it is not recommended for use.
//!
//! [Spotify documentation on authorization.](https://developer.spotify.com/documentation/general/guides/authorization/)

// TODO: this table would be really neat to have if rustfmt didn't mess it up
// | Authorization flow | [Access user resources](ScopedAsyncClient) | Requires secret key | [Access token
// refresh](AccessTokenRefresh) | |-|-|-|-|
// | [AuthorizationCodeUserClient with PKCE](authorization_code) | Yes | No | Yes |
// | [AuthorizationCodeUserClient](authorization_code) | Yes | Yes | Yes |
// | [ImplicitGrantUserClient](implicit_grant) | Yes | No | No |
// | [Client credentials](SpotifyClientWithSecret) | No | Yes | Yes |

pub mod authorization_code;
pub mod implicit_grant;

pub(crate) mod private;
pub(crate) mod scoped;
pub(crate) mod unscoped;

use std::sync::{Arc, RwLock};

use const_format::concatcp;
use log::debug;
use reqwest::{
    header::{self, HeaderMap},
    Method, StatusCode, Url,
};
use serde::Deserialize;

#[cfg(feature = "async")]
use self::private::AsyncClient;
#[cfg(feature = "sync")]
use self::private::SyncClient;
pub use self::unscoped::SearchBuilder;
use self::{
    authorization_code::{
        AsyncAuthorizationCodeUserClient, AsyncAuthorizationCodeUserClientBuilder, SyncAuthorizationCodeUserClient,
        SyncAuthorizationCodeUserClientBuilder,
    },
    implicit_grant::{
        AsyncImplicitGrantUserClientBuilder, ImplicitGrantUserClientBuilder, SyncImplicitGrantUserClientBuilder,
    },
};
#[cfg(feature = "async")]
pub use self::{scoped::ScopedAsyncClient, unscoped::UnscopedAsyncClient};
use crate::{
    error::{Error, Result},
    model::error::{AuthenticationErrorKind, AuthenticationErrorResponse},
};

/// Type alias for an asynchronous Spotify client. See [SpotifyClient](SpotifyClient).
#[cfg(feature = "async")]
pub type AsyncSpotifyClient = SpotifyClient<AsyncClient>;

/// Type alias for a synchronous Spotify client. See [SpotifyClient](SpotifyClient).
#[cfg(feature = "sync")]
pub type SyncSpotifyClient = SpotifyClient<SyncClient>;

/// Type alias for an asynchronous Spotify client with client secret. See
/// [SpotifyClientWithSecret](SpotifyClientWithSecret).
#[cfg(feature = "async")]
pub type AsyncSpotifyClientWithSecret = SpotifyClientWithSecret<AsyncClient>;

/// Type alias for a synchronous Spotify client with client secret. See
/// [SpotifyClientWithSecret](SpotifyClientWithSecret).
#[cfg(feature = "sync")]
pub type SyncSpotifyClientWithSecret = SpotifyClientWithSecret<SyncClient>;

const RANDOM_STATE_LENGTH: usize = 16;
const PKCE_VERIFIER_LENGTH: usize = 128; // maximum Spotify allows
const CLIENT_CREDENTIALS_TOKEN_REQUEST_FORM: &[(&str, &str)] = &[("grant_type", "client_credentials")];

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
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AccessTokenRefreshAsync: crate::private::Sealed {
    /// Asynchronously request a new access token from Spotify and save it internally in the client.
    async fn refresh_access_token(&self) -> Result<()>;
}

/// Clients that have automatically refreshable access tokens implement this trait.
/// [SpotifyClientWithSecret](SpotifyClientWithSecret) and
/// [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient) implement this trait.
///
/// Note that [ImplicitGrantUserClient](implicit_grant::ImplicitGrantUserClient) does *not* implement
/// this trait, since even though it has an access token, it cannot be automatically refreshed.
#[cfg(feature = "sync")]
pub trait AccessTokenRefreshSync: crate::private::Sealed {
    /// Synchronously request a new access token from Spotify and save it internally in the client.
    fn refresh_access_token(&self) -> Result<()>;
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
pub struct SpotifyClient<C>
where
    C: private::HttpClient,
{
    inner: Arc<SpotifyClientRef>,
    http_client: C,
}

#[derive(Debug)]
struct SpotifyClientRef {
    client_id: String,
}

/// A base Spotify client that has a client secret.
///
/// This client can be used to access all [unscoped Spotify endpoints](UnscopedAsyncClient). It can also be used to
/// retrieve an user-authenticated [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient) that
/// can access all [scoped endpoints](ScopedAsyncClient).
///
/// This client uses `Arc` and interior mutability internally, so you do not need to wrap it in an `Arc` or a `Mutex` in
/// order to reuse it.
#[derive(Debug, Clone)]
pub struct SpotifyClientWithSecret<C>
where
    C: private::HttpClient,
{
    inner: Arc<SpotifyClientWithSecretRef>,
    http_client: C,
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

#[cfg(feature = "async")]
impl AsyncSpotifyClient {
    /// Returns a new builder for an [ImplicitGrantUserClient](implicit_grant::ImplicitGrantUserClient).
    ///
    /// # Note
    ///
    /// The implicit grant user client is not recommended for use. The access token is returned in the callback URL
    /// instead of through a trusted channel, and the token cannot be automatically refreshed. It is recommended to use
    /// the [authorization code flow with PKCE flow](SpotifyClient::authorization_code_client_with_pkce) instead.
    pub fn implicit_grant_client<S>(&self, redirect_uri: S) -> AsyncImplicitGrantUserClientBuilder
    where
        S: Into<String>,
    {
        ImplicitGrantUserClientBuilder::new(redirect_uri.into(), Arc::clone(&self.inner), self.http_client.clone())
    }

    /// Returns a new builder for an [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient)
    /// that uses PKCE.
    ///
    /// PKCE is required for strong authentication when the client secret cannot be securely stored in the environment.
    pub fn authorization_code_client_with_pkce<S>(&self, redirect_uri: S) -> AsyncAuthorizationCodeUserClientBuilder
    where
        S: Into<String>,
    {
        AsyncAuthorizationCodeUserClientBuilder::new(
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
    ) -> Result<AsyncAuthorizationCodeUserClient>
    where
        S: Into<String>,
    {
        AsyncAuthorizationCodeUserClient::new_with_refresh_token(
            self.http_client.clone(),
            refresh_token.into(),
            Some(self.inner.client_id.clone()),
        )
        .await
    }
}

#[cfg(feature = "sync")]
impl SyncSpotifyClient {
    /// Returns a new builder for an [ImplicitGrantUserClient](implicit_grant::ImplicitGrantUserClient).
    ///
    /// # Note
    ///
    /// The implicit grant user client is not recommended for use. The access token is returned in the callback URL
    /// instead of through a trusted channel, and the token cannot be automatically refreshed. It is recommended to use
    /// the [authorization code flow with PKCE flow](SpotifyClient::authorization_code_client_with_pkce) instead.
    pub fn implicit_grant_client<S>(&self, redirect_uri: S) -> SyncImplicitGrantUserClientBuilder
    where
        S: Into<String>,
    {
        ImplicitGrantUserClientBuilder::new(redirect_uri.into(), Arc::clone(&self.inner), self.http_client.clone())
    }

    /// Returns a new builder for an [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient)
    /// that uses PKCE.
    ///
    /// PKCE is required for strong authentication when the client secret cannot be securely stored in the environment.
    pub fn authorization_code_client_with_pkce<S>(&self, redirect_uri: S) -> SyncAuthorizationCodeUserClientBuilder
    where
        S: Into<String>,
    {
        SyncAuthorizationCodeUserClientBuilder::new(
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
    ) -> Result<SyncAuthorizationCodeUserClient>
    where
        S: Into<String>,
    {
        SyncAuthorizationCodeUserClient::new_with_refresh_token(
            self.http_client.clone(),
            refresh_token.into(),
            Some(self.inner.client_id.clone()),
        )
    }
}

#[cfg(feature = "async")]
impl AsyncSpotifyClientWithSecret {
    /// Returns a new builder for an asynchronous
    /// [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient).
    pub fn authorization_code_client<S>(&self, redirect_uri: S) -> AsyncAuthorizationCodeUserClientBuilder
    where
        S: Into<String>,
    {
        AsyncAuthorizationCodeUserClientBuilder::new(
            redirect_uri.into(),
            self.inner.client_id.clone(),
            self.http_client.clone(),
        )
    }

    /// Returns a new asynchronous [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient) that
    /// uses an existing refresh token.
    ///
    /// The refresh token will be used to retrieve a new access token before the client is returned.
    pub async fn authorization_code_client_with_refresh_token<S>(
        &self,
        refresh_token: S,
    ) -> Result<AsyncAuthorizationCodeUserClient>
    where
        S: Into<String>,
    {
        AsyncAuthorizationCodeUserClient::new_with_refresh_token(self.http_client.clone(), refresh_token.into(), None)
            .await
    }
}

#[cfg(feature = "sync")]
impl SyncSpotifyClientWithSecret {
    /// Returns a new builder for a synchronous
    /// [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient).
    pub fn authorization_code_client<S>(&self, redirect_uri: S) -> SyncAuthorizationCodeUserClientBuilder
    where
        S: Into<String>,
    {
        SyncAuthorizationCodeUserClientBuilder::new(
            redirect_uri.into(),
            self.inner.client_id.clone(),
            self.http_client.clone(),
        )
    }

    /// Returns a new synchronous [AuthorizationCodeUserClient](authorization_code::AuthorizationCodeUserClient) that
    /// uses an existing refresh token.
    ///
    /// The refresh token will be used to retrieve a new access token before the client is returned.
    pub async fn authorization_code_client_with_refresh_token<S>(
        &self,
        refresh_token: S,
    ) -> Result<SyncAuthorizationCodeUserClient>
    where
        S: Into<String>,
    {
        SyncAuthorizationCodeUserClient::new_with_refresh_token(self.http_client.clone(), refresh_token.into(), None)
    }
}

impl SpotifyClientBuilder {
    /// Return a new Spotify client builder with a given client ID.
    pub fn new<S>(client_id: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            client_id: client_id.into(),
        }
    }

    /// Set the Spotify client's application secret.
    pub fn client_secret<S>(self, client_secret: S) -> SpotifyClientWithSecretBuilder
    where
        S: Into<String>,
    {
        SpotifyClientWithSecretBuilder {
            client_id: self.client_id,
            client_secret: client_secret.into(),
        }
    }

    /// Finalize the builder and return an asynchronous Spotify client.
    #[cfg(feature = "async")]
    pub fn build_async(self) -> AsyncSpotifyClient {
        self.build_client()
    }

    /// Finalize the builder and return a synchronous Spotify client.
    #[cfg(feature = "sync")]
    pub fn build_sync(self) -> SyncSpotifyClient {
        self.build_client()
    }

    fn build_client<C>(self) -> SpotifyClient<C>
    where
        C: private::HttpClient + Clone,
    {
        SpotifyClient {
            inner: Arc::new(SpotifyClientRef {
                client_id: self.client_id,
            }),
            http_client: C::new(),
        }
    }
}

impl SpotifyClientWithSecretBuilder {
    fn get_default_headers(&self) -> HeaderMap {
        let mut default_headers = header::HeaderMap::new();
        default_headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&build_authorization_header(&self.client_id, &self.client_secret))
                // this can only fail if the header value contains non-ASCII characters, which shouldn't happen since
                // the given header value is in base64
                .expect(
                    "failed to insert authorization header into header map: non-ASCII characters in value (this is \
                     likely a bug)",
                ),
        );

        default_headers
    }

    fn build_client<C>(self, token_response: ClientTokenResponse, http_client: C) -> SpotifyClientWithSecret<C>
    where
        C: private::HttpClient + Clone,
    {
        debug!("Got token response for client credentials flow: {:?}", token_response);

        SpotifyClientWithSecret {
            inner: Arc::new(SpotifyClientWithSecretRef {
                client_id: self.client_id,
                // client_secret: self.client_secret,
                access_token: RwLock::new(token_response.access_token),
            }),
            http_client,
        }
    }
}

#[cfg(feature = "async")]
impl SpotifyClientWithSecretBuilder {
    /// Request an access token from Spotify using the client credentials flow and return an asynchronous Spotify
    /// client.
    pub async fn build_async(self) -> Result<AsyncSpotifyClientWithSecret> {
        debug!("Requesting access token for client credentials flow");

        let http_client = AsyncClient(
            reqwest::Client::builder()
                .default_headers(self.get_default_headers())
                .build()
                // this can only fail due to a system error or system misconfiguration
                .expect("failed to build HTTP client: system error or system misconfiguration"),
        );

        let response = http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(CLIENT_CREDENTIALS_TOKEN_REQUEST_FORM)
            .send()
            .await?;

        let response = extract_authentication_error_async(response)
            .await
            .map_err(map_client_authentication_error)?;

        let token_response = response.json().await?;

        Ok(self.build_client(token_response, http_client))
    }
}

#[cfg(feature = "sync")]
impl SpotifyClientWithSecretBuilder {
    /// Request an access token from Spotify using the client credentials flow and return a synchronous Spotify
    /// client.
    pub fn build_sync(self) -> Result<SyncSpotifyClientWithSecret> {
        debug!("Requesting access token for client credentials flow");

        let http_client = SyncClient(
            reqwest::blocking::Client::builder()
                .default_headers(self.get_default_headers())
                .build()
                // this can only fail due to a system error or system misconfiguration
                .expect("failed to build blocking HTTP client: system error or system misconfiguration"),
        );

        let response = http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(CLIENT_CREDENTIALS_TOKEN_REQUEST_FORM)
            .send()?;

        let response = extract_authentication_error_sync(response).map_err(map_client_authentication_error)?;
        let token_response = response.json()?;

        Ok(self.build_client(token_response, http_client))
    }
}

impl<C> crate::private::Sealed for SpotifyClientWithSecret<C> where C: private::HttpClient + Clone {}

#[cfg(feature = "async")]
impl private::BuildHttpRequestAsync for AsyncSpotifyClientWithSecret {
    fn build_http_request(&self, method: Method, url: Url) -> reqwest::RequestBuilder {
        let access_token = self.inner.access_token.read().expect("access token rwlock poisoned");
        self.http_client.request(method, url).bearer_auth(access_token.as_str())
    }
}

#[cfg(feature = "sync")]
impl private::BuildHttpRequestSync for SyncSpotifyClientWithSecret {
    fn build_http_request(&self, method: Method, url: Url) -> reqwest::blocking::RequestBuilder {
        let access_token = self.inner.access_token.read().expect("access token rwlock poisoned");
        self.http_client.request(method, url).bearer_auth(access_token.as_str())
    }
}

impl<C> SpotifyClientWithSecret<C>
where
    C: private::HttpClient + Clone,
{
    fn save_access_token(&self, token_response: ClientTokenResponse) {
        debug!("Got token response for client credentials flow: {:?}", token_response);
        *self.inner.access_token.write().expect("access token rwlock poisoned") = token_response.access_token;
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl<'a> UnscopedAsyncClient<'a> for AsyncSpotifyClientWithSecret {}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl AccessTokenRefreshAsync for AsyncSpotifyClientWithSecret {
    async fn refresh_access_token(&self) -> Result<()> {
        debug!("Refreshing access token for client credentials flow");

        let response = self
            .http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(CLIENT_CREDENTIALS_TOKEN_REQUEST_FORM)
            .send()
            .await?;

        let response = extract_authentication_error_async(response)
            .await
            .map_err(map_client_authentication_error)?;

        let token_response = response.json().await?;
        self.save_access_token(token_response);

        Ok(())
    }
}

#[cfg(feature = "sync")]
impl AccessTokenRefreshSync for SyncSpotifyClientWithSecret {
    fn refresh_access_token(&self) -> Result<()> {
        debug!("Refreshing access token for client credentials flow");

        let response = self
            .http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(CLIENT_CREDENTIALS_TOKEN_REQUEST_FORM)
            .send()?;

        let response = extract_authentication_error_sync(response).map_err(map_client_authentication_error)?;
        let token_response = response.json()?;
        self.save_access_token(token_response);

        Ok(())
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl private::AccessTokenExpiryAsync for AsyncSpotifyClientWithSecret {
    async fn handle_access_token_expired(&self) -> Result<private::AccessTokenExpiryResult> {
        self.refresh_access_token().await?;
        Ok(private::AccessTokenExpiryResult::Ok)
    }
}

#[cfg(feature = "sync")]
impl private::AccessTokenExpirySync for SyncSpotifyClientWithSecret {
    fn handle_access_token_expired(&self) -> Result<private::AccessTokenExpiryResult> {
        self.refresh_access_token()?;
        Ok(private::AccessTokenExpiryResult::Ok)
    }
}

fn build_authorization_header(client_id: &str, client_secret: &str) -> String {
    let auth = format!("{}:{}", client_id, client_secret);
    format!("Basic {}", base64::encode(auth))
}

/// Takes a response for an authentication request and if its status is 400, parses its body as an authentication error.
/// On success returns the given response without modifying it.
#[cfg(feature = "async")]
async fn extract_authentication_error_async(response: reqwest::Response) -> Result<reqwest::Response> {
    if let StatusCode::BAD_REQUEST = response.status() {
        let error_response: AuthenticationErrorResponse = response.json().await?;
        Err(error_response.into_unhandled_error())
    } else {
        Ok(response)
    }
}

/// Takes a response for an authentication request and if its status is 400, parses its body as an authentication error.
/// On success returns the given response without modifying it.
#[cfg(feature = "sync")]
fn extract_authentication_error_sync(response: reqwest::blocking::Response) -> Result<reqwest::blocking::Response> {
    if let StatusCode::BAD_REQUEST = response.status() {
        let error_response: AuthenticationErrorResponse = response.json()?;
        Err(error_response.into_unhandled_error())
    } else {
        Ok(response)
    }
}

#[cfg(feature = "sync")]
fn rate_limit_sleep_sync(sleep_time: u64) -> Result<()> {
    Err(crate::error::Error::RateLimit(sleep_time))
}

/// Return a rate limit error since no sleep utility has been enabled.
#[cfg(all(feature = "async", not(feature = "tokio_sleep"), not(feature = "async_std_sleep")))]
async fn rate_limit_sleep_async(sleep_time: u64) -> Result<()> {
    Err(crate::error::Error::RateLimit(sleep_time))
}

// sleeping with tokio takes precedence over async_std so if the user enables both features for some reason, they get
// tokio sleep
/// Sleep for the specified amount of time using tokio's sleep function.
#[cfg(all(feature = "async", feature = "tokio_sleep"))]
async fn rate_limit_sleep_async(sleep_time: u64) -> Result<()> {
    tokio::time::sleep(std::time::Duration::from_secs(sleep_time)).await;
    Ok(())
}

/// Sleep for the specified amount of time using async_std's sleep function.
#[cfg(all(feature = "async", feature = "async_std_sleep", not(feature = "tokio_sleep")))]
async fn rate_limit_sleep_async(sleep_time: u64) -> Result<()> {
    async_std::task::sleep(std::time::Duration::from_secs(sleep_time)).await;
    Ok(())
}

fn map_client_authentication_error(err: Error) -> Error {
    if let Error::UnhandledAuthenticationError(AuthenticationErrorKind::InvalidClient, description) = err {
        Error::InvalidClient(description)
    } else {
        err
    }
}
