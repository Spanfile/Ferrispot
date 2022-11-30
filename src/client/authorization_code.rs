//! Contains the [AuthorizationCodeUserClient](AuthorizationCodeUserClient) and its builder structs. The client
//! implements the authorization code flow with optional PKCE.
//!
//! [Spotify documentation on the authorization code flow.](https://developer.spotify.com/documentation/general/guides/authorization/code-flow/).
//!
//! # Usage
//!
//! A new [AuthorizationCodeUserClient] may be built with the
//! [`authorization_code_client`-function](crate::client::SpotifyClientWithSecret::authorization_code_client) in
//! [SpotifyClientWithSecret](crate::client::SpotifyClientWithSecret).
//!
//! ```no_run
//! # use ferrispot::client::SpotifyClientBuilder;
//! # use ferrispot::scope::Scope;
//! # async fn foo() {
//! // build a new Spotify client that has the application secret
//! let spotify_client = SpotifyClientBuilder::new("application client ID")
//!     .client_secret("application client secret")
//!     // a synchronous client may be built with .build_sync()
//!     .build_async()
//!     .await
//!     .expect("failed to build Spotify client");
//!
//! // begin building a new AuthorizationCodeUserClient
//! let incomplete_auth_code_client = spotify_client
//!     // the callback URL here should match one of the callback URLs
//!     // specified in your Spotify application
//!     .authorization_code_client("http://localhost/callback")
//!     // specify any (or none) of the scopes you require access to
//!     .scopes([Scope::UserReadPlaybackState])
//!     // in case the user has already approved the application, this may be
//!     // set to `true` for force the user approve the application again
//!     .show_dialog(true)
//!     .build();
//!
//! // at this point the client is configured but not yet ready for use; it is
//! // still missing the user authorization
//!
//! // generate an authorization URL for the user. this URL takes the user to a
//! // Spotify page where they are prompted to give the application access to
//! // their account and all the scopes you've specified earlier
//! let authorize_url = incomplete_auth_code_client.get_authorize_url();
//!
//! // the user should now be directed to this URL in some manner
//!
//! // when the user accepts, they are redirected to the previously specified
//! // callback URL, which will contain an authorization code (`code`) and a
//! // state code (`state`) in the query parameters. you should extract both of
//! // them from the URL in some manner
//! # let code = "";
//! # let state = "";
//!
//! // finalize the client with the authorization code and state. the client
//! // will use the authorization code to request an access token and a refresh
//! // token from Spotify, which it will use to access the API
//! let user_client = incomplete_auth_code_client
//!     .finalize(code, state)
//!     .await
//!     .expect("failed to finalize authorization code flow client");
//! # }
//! ```
//!
//! # Usage with PKCE
//!
//! In case the application's client secret cannot be safely stored in the environment, PKCE may still be used to
//! strongly authenticate the client with Spotify. A new [AuthorizationCodeUserClient] that uses PKCE may be built with
//! the [`authorization_code_client_with_pkce`-function](crate::client::SpotifyClient::authorization_code_client_with_pkce)
//! in [SpotifyClient](crate::client::SpotifyClient).
//!
//! ```no_run
//! # use ferrispot::client::SpotifyClientBuilder;
//! # use ferrispot::scope::Scope;
//! # async fn foo() {
//! // build a new Spotify client that doesn't have the application secret
//! let spotify_client = SpotifyClientBuilder::new("application client ID")
//!     .build_async();
//!
//! // begin building a new AuthorizationCodeUserClient that uses PKCE
//! let incomplete_auth_code_client = spotify_client
//!     // the callback URL here should match one of the callback URLs
//!     // specified in your Spotify application
//!     .authorization_code_client_with_pkce("http://localhost/callback")
//!     // specify any (or none) of the scopes you require access to
//!     .scopes([Scope::UserReadPlaybackState])
//!     // in case the user has already approved the application, this may be
//!     // set to `true` for force the user approve
//!     // the application again
//!     .show_dialog(true)
//!     .build();
//!
//! // from here on out, the usage is identical as with the usual client. refer
//! // to the documentation above
//! # }

use std::sync::{Arc, RwLock};

use log::debug;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{IntoUrl, Method, Url};
use serde::Deserialize;
use sha2::Digest;

use super::{
    private, ACCOUNTS_API_TOKEN_ENDPOINT, ACCOUNTS_AUTHORIZE_ENDPOINT, PKCE_VERIFIER_LENGTH, RANDOM_STATE_LENGTH,
};
#[cfg(feature = "async")]
use super::{
    private::{AsyncClient, BuildHttpRequestAsync},
    AccessTokenRefreshAsync,
};
#[cfg(feature = "sync")]
use super::{
    private::{BuildHttpRequestSync, SyncClient},
    AccessTokenRefreshSync,
};
use crate::{
    error::{Error, Result},
    model::error::AuthenticationErrorKind,
    scope::ToScopesString,
};

/// Type alias for an asynchronous authorization code user client. See
/// [AuthorizationCodeUserClient](AuthorizationCodeUserClient).
#[cfg(feature = "async")]
pub type AsyncAuthorizationCodeUserClient = AuthorizationCodeUserClient<AsyncClient>;

/// Type alias for a synchronous authorization code user client. See
/// [AuthorizationCodeUserClient](AuthorizationCodeUserClient).
#[cfg(feature = "sync")]
pub type SyncAuthorizationCodeUserClient = AuthorizationCodeUserClient<SyncClient>;

/// Type alias for an incomplete asynchronous authorization code user client. See
/// [IncompleteAuthorizationCodeUserClient](IncompleteAuthorizationCodeUserClient).
#[cfg(feature = "async")]
pub type AsyncIncompleteAuthorizationCodeUserClient = IncompleteAuthorizationCodeUserClient<AsyncClient>;

/// Type alias for a incomplete synchronous authorization code user client. See
/// [IncompleteAuthorizationCodeUserClient](IncompleteAuthorizationCodeUserClient).
#[cfg(feature = "sync")]
pub type SyncIncompleteAuthorizationCodeUserClient = IncompleteAuthorizationCodeUserClient<SyncClient>;

/// Type alias for an asynchronous authorization code client user client builder. See
/// [AuthorizationCodeUserClientBuilder](AuthorizationCodeUserClientBuilder).
#[cfg(feature = "async")]
pub type AsyncAuthorizationCodeUserClientBuilder = AuthorizationCodeUserClientBuilder<AsyncClient>;

/// Type alias for a synchronous authorization code client user client builder. See
/// [AuthorizationCodeUserClientBuilder](AuthorizationCodeUserClientBuilder).
#[cfg(feature = "sync")]
pub type SyncAuthorizationCodeUserClientBuilder = AuthorizationCodeUserClientBuilder<SyncClient>;

/// A client that implements the authorization code flow to authenticate an user with Spotify. May optionally use PKCE
/// if the client secret is not available. See the [module-level documentation](self) for more information.
///
/// Implements all the [scoped](crate::client::ScopedAsyncClient) and [unscoped
/// endpoints](crate::client::UnscopedAsyncClient).
///
/// This struct is generic over its internal asynchronous/synchronous HTTP client. You cannot refer to the internal
/// client types directly, hence there are type aliases for both kinds of clients: [AsyncAuthorizationCodeUserClient]
/// and [SyncAuthorizationCodeUserClient]. Likewise, both the builder struct and the incomplete client struct are
/// similarly generic, and have equivalent type aliases.
///
/// This client uses `Arc` and interior mutability internally, so you do not need to wrap it in an `Arc` or a `Mutex` in
/// order to reuse it.
#[derive(Debug, Clone)]
pub struct AuthorizationCodeUserClient<C>
where
    C: private::HttpClient + Clone,
{
    inner: Arc<AuthorizationCodeUserClientRef>,
    http_client: C,
}

#[derive(Debug)]
struct AuthorizationCodeUserClientRef {
    access_token: RwLock<String>,
    refresh_token: RwLock<String>,
    client_id: Option<String>,
}

/// An incomplete authorization code user client.
///
/// The client has been configured, and it has to be [finalized](IncompleteAuthorizationCodeUserClient::finalize) by
/// directing the user to the [authorize URL](IncompleteAuthorizationCodeUserClient::get_authorize_url) and retrieving
/// an authorization code and a state parameter from the redirect callback URL.
#[derive(Debug)]
pub struct IncompleteAuthorizationCodeUserClient<C>
where
    C: private::HttpClient + Clone,
{
    client_id: String,
    redirect_uri: String,
    state: String,
    scopes: Option<String>,
    show_dialog: bool,
    pkce_verifier: Option<String>,

    http_client: C,
}

/// Builder for [AuthorizationCodeUserClient].
#[derive(Debug)]
pub struct AuthorizationCodeUserClientBuilder<C>
where
    C: private::HttpClient + Clone,
{
    client_id: String,
    redirect_uri: String,
    scopes: Option<String>,
    show_dialog: bool,
    pkce_verifier: Option<String>,

    http_client: C,
}

#[derive(Debug, Deserialize)]
struct AuthorizeUserTokenResponse {
    access_token: String,
    refresh_token: String,

    // these fields are in the response but the library doesn't need them. keep them here for logging purposes
    #[allow(dead_code)]
    scope: Option<String>,
    #[allow(dead_code)]
    expires_in: u32,
    #[allow(dead_code)]
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct RefreshUserTokenResponse {
    access_token: String,
    refresh_token: Option<String>,

    // these fields are in the response but the library doesn't need them. keep them here for logging purposes
    #[allow(dead_code)]
    scope: Option<String>,
    #[allow(dead_code)]
    expires_in: u32,
    #[allow(dead_code)]
    token_type: String,
}

impl<C> AuthorizationCodeUserClient<C>
where
    C: private::HttpClient + Clone,
{
    fn new_from_refresh_token(
        token_response: RefreshUserTokenResponse,
        refresh_token: String,
        client_id: Option<String>,
        http_client: C,
    ) -> Self {
        debug!(
            "Got token response for refreshing authorization code flow tokens: {:?}",
            token_response
        );

        let refresh_token = token_response.refresh_token.unwrap_or(refresh_token);

        Self {
            inner: Arc::new(AuthorizationCodeUserClientRef {
                access_token: RwLock::new(token_response.access_token),
                refresh_token: RwLock::new(refresh_token),
                client_id,
            }),
            http_client,
        }
    }

    /// Returns the current refresh token.
    ///
    /// The refresh token may be saved and reused later when creating a new client with the
    /// [`authorization_code_client_with_refresh_token`-function](crate::client::SpotifyClientWithSecret::authorization_code_client_with_refresh_token)
    /// or the [`authorization_code_client_with_refresh_token_and_pkce`-function](crate::client::SpotifyClient::authorization_code_client_with_refresh_token_and_pkce).
    ///
    /// This function returns an owned String by cloning the internal refresh token.
    pub fn get_refresh_token(&self) -> String {
        self.inner
            .refresh_token
            .read()
            .expect("refresh token rwlock poisoned")
            .to_owned()
    }

    fn update_access_and_refresh_tokens(&self, token_response: RefreshUserTokenResponse) {
        debug!(
            "Got token response for refreshing authorization code flow tokens: {:?}",
            token_response
        );

        *self.inner.access_token.write().expect("access token rwlock poisoned") = token_response.access_token;

        if let Some(refresh_token) = token_response.refresh_token {
            *self.inner.refresh_token.write().expect("refresh token rwlock poisoned") = refresh_token;
        }
    }
}

#[cfg(feature = "async")]
impl AsyncAuthorizationCodeUserClient {
    pub(crate) async fn new_with_refresh_token(
        http_client: AsyncClient,
        refresh_token: String,
        client_id: Option<String>,
    ) -> Result<Self> {
        debug!(
            "Attempting to create new authorization code flow client with existng refresh token: {} and client ID \
             (for PKCE): {:?}",
            refresh_token, client_id
        );

        let response = http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(&build_refresh_token_request_form(&refresh_token, client_id.as_deref()))
            .send()
            .await?;

        let response = super::extract_authentication_error_async(response)
            .await
            .map_err(map_refresh_token_error)?;

        let token_response = response.json().await?;

        Ok(Self::new_from_refresh_token(
            token_response,
            refresh_token,
            client_id,
            http_client,
        ))
    }
}

#[cfg(feature = "sync")]
impl SyncAuthorizationCodeUserClient {
    pub(crate) fn new_with_refresh_token(
        http_client: SyncClient,
        refresh_token: String,
        client_id: Option<String>,
    ) -> Result<Self> {
        debug!(
            "Attempting to create new authorization code flow client with existng refresh token: {} and client ID \
             (for PKCE): {:?}",
            refresh_token, client_id
        );

        let response = http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(&build_refresh_token_request_form(&refresh_token, client_id.as_deref()))
            .send()?;

        let response = super::extract_authentication_error_sync(response).map_err(map_refresh_token_error)?;
        let token_response = response.json()?;

        Ok(Self::new_from_refresh_token(
            token_response,
            refresh_token,
            client_id,
            http_client,
        ))
    }
}

impl<C> IncompleteAuthorizationCodeUserClient<C>
where
    C: private::HttpClient + Clone,
{
    /// Returns an authorization URL the user should be directed to in some manner.
    ///
    /// Once the user approves the application, they are redirected back to the application's callback URL. The URL
    /// query in the callback will contain a `code` parameter and a `state` parameter, which should be passed to the
    /// [`finalize`-function](IncompleteAuthorizationCodeUserClient::finalize) in order to complete the client and get
    /// an [AuthorizationCodeUserClient].
    pub fn get_authorize_url(&self) -> String {
        let mut query_params = vec![
            ("response_type", "code"),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("client_id", self.client_id.as_str()),
            ("state", self.state.as_str()),
            ("show_dialog", if self.show_dialog { "true" } else { "false" }),
        ];

        if let Some(scopes) = &self.scopes {
            query_params.push(("scope", scopes.as_str()));
        }

        let authorize_url = if let Some(pkce_verifier) = self.pkce_verifier.as_deref() {
            let mut hasher = sha2::Sha256::new();
            hasher.update(pkce_verifier);
            let pkce_challenge = hasher.finalize();
            let pkce_challenge = base64::encode_config(pkce_challenge, base64::URL_SAFE_NO_PAD);

            debug!(
                "Using PKCE extension with verifier: {} and challenge: {}",
                pkce_verifier, pkce_challenge
            );

            query_params.extend([("code_challenge_method", "S256"), ("code_challenge", &pkce_challenge)]);

            // parsing the URL fails only if the base URL is invalid, not the parameters. if this method fails, there's
            // a bug in the library

            // while both these branches end the same way, this one borrows the pkce_challenge string in query_params so
            // the URL must be built before the string falls out of scope
            Url::parse_with_params(ACCOUNTS_AUTHORIZE_ENDPOINT, &query_params)
                .expect("failed to build authorize URL: invalid base URL (this is likely a bug)")
        } else {
            Url::parse_with_params(ACCOUNTS_AUTHORIZE_ENDPOINT, &query_params)
                .expect("failed to build authorize URL: invalid base URL (this is likely a bug)")
        };

        authorize_url.into()
    }

    fn build_authorization_code_token_request_form<'a>(
        &'a self,
        code: &'a str,
        state: &str,
    ) -> Result<Vec<(&'a str, &'a str)>> {
        debug!(
            "Attempting to finalize authorization code flow user client with code: {} and state: {}",
            code, state
        );

        if state != self.state {
            return Err(Error::AuthorizationCodeStateMismatch);
        }

        let mut token_request_form = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];

        if let Some(pkce_verifier) = self.pkce_verifier.as_deref() {
            debug!("Requesting access and refresh tokens for authorization code flow with PKCE");
            token_request_form.extend([("client_id", self.client_id.as_str()), ("code_verifier", pkce_verifier)]);
        } else {
            debug!("Requesting access and refresh tokens for authorization code flow");
        }

        Ok(token_request_form)
    }

    fn build_client(self, token_response: AuthorizeUserTokenResponse) -> AuthorizationCodeUserClient<C> {
        debug!("Got token response for authorization code flow: {:?}", token_response);

        AuthorizationCodeUserClient {
            http_client: self.http_client,
            // from here on out, using PKCE only requires us supplying our client ID when refreshing the access
            // token. if the PKCE verifier is used, include the client ID
            inner: Arc::new(AuthorizationCodeUserClientRef {
                access_token: RwLock::new(token_response.access_token),
                refresh_token: RwLock::new(token_response.refresh_token),
                client_id: self.pkce_verifier.and(Some(self.client_id)),
            }),
        }
    }
}

#[cfg(feature = "async")]
impl AsyncIncompleteAuthorizationCodeUserClient {
    /// Finalize this client with a code and a state from the callback URL query the user was redirected to after they
    /// approved the application and return an usable [AuthorizationCodeUserClient].
    ///
    /// This function will use the authorization code to request an access and a refresh token from Spotify. If the
    /// originally generated state does not match the `state` parameter, the function will return an
    /// [AuthorizationCodeStateMismatch-error](Error::AuthorizationCodeStateMismatch).
    pub async fn finalize(self, code: &str, state: &str) -> Result<AsyncAuthorizationCodeUserClient> {
        let token_request_form = self.build_authorization_code_token_request_form(code, state)?;
        let response = self
            .http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(&token_request_form)
            .send()
            .await?;

        let response = super::extract_authentication_error_async(response)
            .await
            .map_err(map_authentication_error)?;

        let token_response = response.json().await?;

        Ok(self.build_client(token_response))
    }
}

#[cfg(feature = "sync")]
impl SyncIncompleteAuthorizationCodeUserClient {
    /// Finalize this client with a code and a state from the callback URL query the user was redirected to after they
    /// approved the application and return an usable [AuthorizationCodeUserClient].
    ///
    /// This function will use the authorization code to request an access and a refresh token from Spotify. If the
    /// originally generated state does not match the `state` parameter, the function will return an
    /// [AuthorizationCodeStateMismatch-error](Error::AuthorizationCodeStateMismatch).
    pub fn finalize(self, code: &str, state: &str) -> Result<SyncAuthorizationCodeUserClient> {
        let token_request_form = self.build_authorization_code_token_request_form(code, state)?;
        let response = self
            .http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(&token_request_form)
            .send()?;

        let response = super::extract_authentication_error_sync(response).map_err(map_authentication_error)?;
        let token_response = response.json()?;

        Ok(self.build_client(token_response))
    }
}

#[cfg(feature = "async")]
impl AsyncAuthorizationCodeUserClientBuilder {
    pub(super) fn new(redirect_uri: String, client_id: String, http_client: AsyncClient) -> Self {
        Self {
            client_id,
            redirect_uri,
            scopes: None,
            show_dialog: false,
            pkce_verifier: None,

            http_client,
        }
    }
}

#[cfg(feature = "sync")]
impl SyncAuthorizationCodeUserClientBuilder {
    pub(super) fn new(redirect_uri: String, client_id: String, http_client: SyncClient) -> Self {
        Self {
            client_id,
            redirect_uri,
            scopes: None,
            show_dialog: false,
            pkce_verifier: None,

            http_client,
        }
    }
}

impl<C> AuthorizationCodeUserClientBuilder<C>
where
    C: private::HttpClient + Clone,
{
    /// Generates a PKCE code verifier to be used in the authentication process.
    pub(super) fn with_pkce(self) -> Self {
        let code_verifier = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(PKCE_VERIFIER_LENGTH)
            .map(char::from)
            .collect();

        Self {
            pkce_verifier: Some(code_verifier),
            ..self
        }
    }

    /// Specify the [OAuth authorization scopes](crate::scope::Scope) that the user is asked to grant for the
    /// application.
    pub fn scopes<T>(self, scopes: T) -> Self
    where
        T: ToScopesString,
    {
        Self {
            scopes: Some(scopes.to_scopes_string()),
            ..self
        }
    }

    /// Set whether or not to force the user to approve the application again, if they've already done so.
    ///
    /// If false (default), a user who has already approved the application is automatically redirected to the specified
    /// redirect URL. If true, the user will not be automatically redirected and will have to approve the application
    /// again.
    pub fn show_dialog(self, show_dialog: bool) -> Self {
        Self { show_dialog, ..self }
    }

    /// Finalize the builder and return an [IncompleteAuthorizationCodeUserClient].
    pub fn build(self) -> IncompleteAuthorizationCodeUserClient<C> {
        let state = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(RANDOM_STATE_LENGTH)
            .map(char::from)
            .collect();

        IncompleteAuthorizationCodeUserClient {
            redirect_uri: self.redirect_uri,
            state,
            scopes: self.scopes,
            show_dialog: self.show_dialog,
            client_id: self.client_id,
            pkce_verifier: self.pkce_verifier,

            http_client: self.http_client,
        }
    }
}

impl<C> crate::private::Sealed for AuthorizationCodeUserClient<C> where C: private::HttpClient + Clone {}

#[cfg(feature = "async")]
impl private::BuildHttpRequestAsync for AsyncAuthorizationCodeUserClient {
    fn build_http_request<U>(&self, method: Method, url: U) -> reqwest::RequestBuilder
    where
        U: IntoUrl,
    {
        let access_token = self.inner.access_token.read().expect("access token rwlock poisoned");
        self.http_client.request(method, url).bearer_auth(access_token.as_str())
    }
}

#[cfg(feature = "sync")]
impl private::BuildHttpRequestSync for SyncAuthorizationCodeUserClient {
    fn build_http_request<U>(&self, method: Method, url: U) -> reqwest::blocking::RequestBuilder
    where
        U: IntoUrl,
    {
        let access_token = self.inner.access_token.read().expect("access token rwlock poisoned");
        self.http_client.request(method, url).bearer_auth(access_token.as_str())
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl<'a> super::ScopedAsyncClient<'a> for AsyncAuthorizationCodeUserClient {}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl<'a> super::UnscopedAsyncClient<'a> for AsyncAuthorizationCodeUserClient {}

#[cfg(feature = "sync")]
impl<'a> super::ScopedSyncClient<'a> for SyncAuthorizationCodeUserClient {}

#[cfg(feature = "sync")]
impl<'a> super::UnscopedSyncClient<'a> for SyncAuthorizationCodeUserClient {}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl super::AccessTokenRefreshAsync for AsyncAuthorizationCodeUserClient {
    async fn refresh_access_token(&self) -> Result<()> {
        // build and send the request this way to not hold the non-async RwLockReadGuard across await points
        let response = {
            let refresh_token = self.inner.refresh_token.read().expect("refresh token rwlock poisoned");
            debug!(
                "Attempting to refresh authorization code flow access token with refresh token: {}",
                refresh_token
            );

            let request = self
                .build_http_request(Method::POST, ACCOUNTS_API_TOKEN_ENDPOINT)
                .form(&build_refresh_token_request_form(
                    &refresh_token,
                    self.inner.client_id.as_deref(),
                ))
                .send();

            // for some reason if I just let the refresh token read guard drop by its own at the end of this scope, it
            // doesn't actually drop by the end and is kept across the await, causing issues
            drop(refresh_token);
            request
        }
        .await?;

        let response = super::extract_authentication_error_async(response)
            .await
            .map_err(map_refresh_token_error)?;

        let token_response = response.json().await?;
        self.update_access_and_refresh_tokens(token_response);

        Ok(())
    }
}

#[cfg(feature = "sync")]
impl super::AccessTokenRefreshSync for SyncAuthorizationCodeUserClient {
    fn refresh_access_token(&self) -> Result<()> {
        let refresh_token = self.inner.refresh_token.read().expect("refresh token rwlock poisoned");
        debug!(
            "Attempting to refresh authorization code flow access token with refresh token: {}",
            refresh_token
        );

        let response = self
            .build_http_request(Method::POST, ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(&build_refresh_token_request_form(
                &refresh_token,
                self.inner.client_id.as_deref(),
            ))
            .send()?;

        // the refresh token may later be written to, drop our read guard
        drop(refresh_token);

        let response = super::extract_authentication_error_sync(response).map_err(map_refresh_token_error)?;
        let token_response = response.json()?;
        self.update_access_and_refresh_tokens(token_response);

        Ok(())
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl private::AccessTokenExpiryAsync for AsyncAuthorizationCodeUserClient {
    async fn handle_access_token_expired(&self) -> Result<private::AccessTokenExpiryResult> {
        self.refresh_access_token().await?;
        Ok(private::AccessTokenExpiryResult::Ok)
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl private::AccessTokenExpirySync for SyncAuthorizationCodeUserClient {
    fn handle_access_token_expired(&self) -> Result<private::AccessTokenExpiryResult> {
        self.refresh_access_token()?;
        Ok(private::AccessTokenExpiryResult::Ok)
    }
}

fn build_refresh_token_request_form<'a>(refresh_token: &'a str, client_id: Option<&'a str>) -> Vec<(&'a str, &'a str)> {
    let mut token_request_form = vec![("grant_type", "refresh_token"), ("refresh_token", refresh_token)];

    if let Some(client_id) = client_id {
        token_request_form.push(("client_id", client_id));
    }

    token_request_form
}

fn map_authentication_error(err: Error) -> Error {
    if let Error::UnhandledAuthenticationError(AuthenticationErrorKind::InvalidGrant, _) = err {
        Error::InvalidAuthorizationCode
    } else {
        err
    }
}

fn map_refresh_token_error(err: Error) -> Error {
    if let Error::UnhandledAuthenticationError(AuthenticationErrorKind::InvalidGrant, description) = err {
        Error::InvalidRefreshToken(description)
    } else {
        err
    }
}
