//! Contains the [ImplicitGrantUserClient](ImplicitGrantUserClient) and its builder structs.
//!
//! # Note
//!
//! The implicit grant user client is not recommended for use. The access token is returned in the callback URL instead
//! through a trusted channel, and the token cannot be automatically refreshed.
//!
//! In an environment where the application's client secret cannot be safely stored, it is recommended to use the
//! [AuthorizationCodeUserClient](crate::client::authorization_code::AuthorizationCodeUserClient) with PKCE, which can
//! be built with the
//! [`authorization_code_client_with_pkce`-function](crate::client::SpotifyClient::authorization_code_client_with_pkce).
//!
//! # Usage
//!
//! A new [ImplicitGrantUserClient] can be built with the [`implicit_grant_client`-function in
//! SpotifyClient](crate::client::SpotifyClient::implicit_grant_client).
//!
//! ```no_run
//! # use ferrispot::client::SpotifyClientBuilder;
//! # use ferrispot::scope::Scope;
//! # async fn foo() {
//! // build a new Spotify client that doesn't have the application secret
//! let spotify_client = SpotifyClientBuilder::new("application client ID")
//!     .build_async();
//!
//! // begin building a new ImplicitGrantUserClient
//! let incomplete_implicit_grant_client = spotify_client
//!     // the callback URL here should match one of the callback URLs
//!     // specified in your Spotify application
//!     .implicit_grant_client("http://localhost/callback")
//!     // specify any (or none) of the scopes you require access to
//!     .scopes([Scope::UserReadPlaybackState])
//!     // in case the user has already approved the application, this may be
//!     // set to `true` for force the user approve
//!     // the application again
//!     .show_dialog(true)
//!     .build();
//!
//! // at this point the client is configured but not yet ready for use; it is
//! // still missing the user authorization
//!
//! // generate an authorization URL for the user. this URL takes the user to a
//! // Spotify page where they are prompted to give the application access to
//! // their account and all the scopes you've specified earlier
//! let authorize_url = incomplete_implicit_grant_client.get_authorize_url();
//!
//! // the user should now be directed to this URL in some manner
//!
//! // when the user accepts, they are redirected to the previously specified
//! // callback URL, which will contain an access token (`access_token`) and a
//! // state code (`state`) in the query parameters. you should extract both of
//! // them from the URL in some manner
//! # let access_token = "";
//! # let state = "";
//!
//! // finalize the client with the access token and state. the client will use
//! // the access token to access the API. once the access token expires, this
//! // client creation flow will have to be gone through again to get a new
//! // access token and client
//! let user_client = incomplete_implicit_grant_client
//!     .finalize(access_token, state)
//!     .expect("failed to finalize implicit grant flow client");
//! # }

use std::sync::Arc;

use log::debug;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{IntoUrl, Method, Url};

#[cfg(feature = "async")]
use super::private::AsyncClient;
#[cfg(feature = "sync")]
use super::private::SyncClient;
use super::{
    private::{self, HttpClient},
    SpotifyClientRef, ACCOUNTS_AUTHORIZE_ENDPOINT, RANDOM_STATE_LENGTH,
};
use crate::{
    error::{Error, Result},
    scope::ToScopesString,
};

/// Type alias for an asynchronous implicit grant user client. See [ImplicitGrantUserClient](ImplicitGrantUserClient).
#[cfg(feature = "async")]
pub type AsyncImplicitGrantUserClient = ImplicitGrantUserClient<AsyncClient>;

/// Type alias for a synchronous implicit grant user client. See [ImplicitGrantUserClient](ImplicitGrantUserClient).
#[cfg(feature = "sync")]
pub type SyncImplicitGrantUserClient = ImplicitGrantUserClient<SyncClient>;

/// Type alias for an incomplete asynchronous implicit grant user client. See
/// [IncompleteImplicitGrantUserClient](IncompleteImplicitGrantUserClient).
#[cfg(feature = "async")]
pub type AsyncIncompleteImplicitGrantUserClient = IncompleteImplicitGrantUserClient<AsyncClient>;

/// Type alias for an incomplete synchronous implicit grant user client. See
/// [IncompleteImplicitGrantUserClient](IncompleteImplicitGrantUserClient).
#[cfg(feature = "sync")]
pub type SyncIncompleteImplicitGrantUserClient = IncompleteImplicitGrantUserClient<SyncClient>;

/// Type alias for an asynchronous implicit grant user client builder. See
/// [ImplicitGrantUserClientBuilder](ImplicitGrantUserClientBuilder).
#[cfg(feature = "async")]
pub type AsyncImplicitGrantUserClientBuilder = ImplicitGrantUserClientBuilder<AsyncClient>;

/// Type alias for a synchronous implicit grant user client builder. See
/// [ImplicitGrantUserClientBuilder](ImplicitGrantUserClientBuilder).
#[cfg(feature = "sync")]
pub type SyncImplicitGrantUserClientBuilder = ImplicitGrantUserClientBuilder<SyncClient>;

/// A client that uses the implicit grant flow to authenticate an user with Spotify. See the [module-level docs](self)
/// for more information.
///
/// Implements all the [scoped](crate::client::ScopedAsyncClient) and [unscoped
/// endpoints](crate::client::UnscopedAsyncClient).
///
/// This struct is generic over its internal asynchronous/synchronous HTTP client. You cannot refer to the internal
/// client types directly, hence there are type aliases for both kinds of clients: [AsyncImplicitGrantUserClient] and
/// [SyncImplicitGrantUserClient]. Likewise, both the builder struct and the incomplete client struct are similarly
/// generic, and have equivalent type aliases.
///
/// This client uses `Arc` internally, so you do not need to wrap it in an `Arc` in order to reuse it.
#[derive(Debug, Clone)]
pub struct ImplicitGrantUserClient<C>
where
    C: HttpClient + Clone,
{
    inner: Arc<ImplicitGrantUserClientRef>,
    http_client: C,
}

#[derive(Debug)]
struct ImplicitGrantUserClientRef {
    access_token: String,
}

#[derive(Debug, Clone)]
pub struct IncompleteImplicitGrantUserClient<C>
where
    C: HttpClient + Clone,
{
    redirect_uri: String, // TODO: figure if this can be &'a str instead
    state: String,
    scopes: Option<String>,
    show_dialog: bool,

    spotify_client_ref: Arc<SpotifyClientRef>,
    http_client: C,
}

pub struct ImplicitGrantUserClientBuilder<C>
where
    C: HttpClient + Clone,
{
    redirect_uri: String, // TODO: figure if this can be &'a str instead
    scopes: Option<String>,
    show_dialog: bool,

    spotify_client_ref: Arc<SpotifyClientRef>,
    http_client: C,
}

impl<C> IncompleteImplicitGrantUserClient<C>
where
    C: HttpClient + Clone,
{
    pub fn get_authorize_url(&self) -> String {
        let mut query_params = vec![
            ("response_type", "token"),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("client_id", self.spotify_client_ref.client_id.as_str()),
            ("state", self.state.as_str()),
        ];

        if let Some(scopes) = &self.scopes {
            query_params.push(("scope", scopes.as_str()));
        }

        if self.show_dialog {
            // spotify's default for show_dialog is false if it's not specified
            query_params.push(("show_dialog", "true"));
        }

        // parsing the URL fails only if the base URL is invalid, not the parameters. if this method fails, there's a
        // bug in the library
        let authorize_url = Url::parse_with_params(ACCOUNTS_AUTHORIZE_ENDPOINT, &query_params)
            .expect("failed to build authorize URL: invalid base URL (this is likely a bug)");

        authorize_url.into()
    }

    pub fn finalize<S>(self, access_token: S, state: &str) -> Result<ImplicitGrantUserClient<C>>
    where
        S: Into<String>,
    {
        let access_token = access_token.into();
        debug!(
            "Attempting to finalize implicit grant flow user client with access_token: {} and state: {}",
            access_token, state
        );

        if state != self.state {
            return Err(Error::AuthorizationCodeStateMismatch);
        }

        Ok(ImplicitGrantUserClient {
            inner: Arc::new(ImplicitGrantUserClientRef { access_token }),
            http_client: self.http_client,
        })
    }
}

impl<C> ImplicitGrantUserClientBuilder<C>
where
    C: HttpClient + Clone,
{
    pub(super) fn new(redirect_uri: String, spotify_client_ref: Arc<SpotifyClientRef>, http_client: C) -> Self {
        Self {
            redirect_uri,
            scopes: None,
            show_dialog: false,

            spotify_client_ref,
            http_client,
        }
    }

    pub fn scopes<T>(self, scopes: T) -> Self
    where
        T: ToScopesString,
    {
        Self {
            scopes: Some(scopes.to_scopes_string()),
            ..self
        }
    }

    pub fn show_dialog(self, show_dialog: bool) -> Self {
        Self { show_dialog, ..self }
    }

    pub fn build(self) -> IncompleteImplicitGrantUserClient<C> {
        let state = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(RANDOM_STATE_LENGTH)
            .map(char::from)
            .collect();

        IncompleteImplicitGrantUserClient {
            redirect_uri: self.redirect_uri,
            state,
            scopes: self.scopes,
            show_dialog: self.show_dialog,

            spotify_client_ref: self.spotify_client_ref,
            http_client: self.http_client,
        }
    }
}

impl<C> crate::private::Sealed for ImplicitGrantUserClient<C> where C: HttpClient + Clone {}

#[cfg(feature = "async")]
impl private::BuildHttpRequestAsync for AsyncImplicitGrantUserClient {
    fn build_http_request<U>(&self, method: Method, url: U) -> reqwest::RequestBuilder
    where
        U: IntoUrl,
    {
        self.http_client
            .request(method, url)
            .bearer_auth(self.inner.access_token.as_str())
    }
}

#[cfg(feature = "sync")]
impl private::BuildHttpRequestSync for SyncImplicitGrantUserClient {
    fn build_http_request<U>(&self, method: Method, url: U) -> reqwest::blocking::RequestBuilder
    where
        U: IntoUrl,
    {
        self.http_client
            .request(method, url)
            .bearer_auth(self.inner.access_token.as_str())
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl<'a> super::ScopedAsyncClient<'a> for AsyncImplicitGrantUserClient {}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl<'a> super::UnscopedAsyncClient<'a> for AsyncImplicitGrantUserClient {}

#[cfg(feature = "sync")]
impl<'a> super::ScopedSyncClient<'a> for SyncImplicitGrantUserClient {}

#[cfg(feature = "sync")]
impl<'a> super::UnscopedSyncClient<'a> for SyncImplicitGrantUserClient {}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl private::AccessTokenExpiryAsync for AsyncImplicitGrantUserClient {
    async fn handle_access_token_expired(&self) -> Result<private::AccessTokenExpiryResult> {
        Ok(private::AccessTokenExpiryResult::Inapplicable)
    }
}

#[cfg(feature = "sync")]
impl private::AccessTokenExpirySync for SyncImplicitGrantUserClient {
    fn handle_access_token_expired(&self) -> Result<private::AccessTokenExpiryResult> {
        Ok(private::AccessTokenExpiryResult::Inapplicable)
    }
}
