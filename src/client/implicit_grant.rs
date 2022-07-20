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
//!     .build();
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

use super::{
    private, ScopedClient, SpotifyClientRef, UnscopedClient, ACCOUNTS_AUTHORIZE_ENDPOINT, RANDOM_STATE_LENGTH,
};
use crate::{
    error::{Error, Result},
    scope::ToScopesString,
};

use async_trait::async_trait;
use log::debug;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{Client as AsyncClient, Method, RequestBuilder, Url};
use std::sync::Arc;

/// A client that uses the implicit grant flow to authenticate an user with Spotify. See the [module-level docs](self)
/// for more information.
#[derive(Debug, Clone)]
pub struct ImplicitGrantUserClient {
    access_token: String,
    http_client: AsyncClient,
}

#[derive(Debug, Clone)]
pub struct IncompleteImplicitGrantUserClient {
    redirect_uri: String, // TODO: figure if this can be &'a str instead
    state: String,
    scopes: Option<String>,
    show_dialog: bool,

    spotify_client_ref: Arc<SpotifyClientRef>,
    http_client: AsyncClient,
}

pub struct ImplicitGrantUserClientBuilder {
    redirect_uri: String, // TODO: figure if this can be &'a str instead
    state: Option<String>,
    scopes: Option<String>,
    show_dialog: bool,

    spotify_client_ref: Arc<SpotifyClientRef>,
    http_client: AsyncClient,
}

impl IncompleteImplicitGrantUserClient {
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
        let authorize_url =
            Url::parse_with_params(ACCOUNTS_AUTHORIZE_ENDPOINT, &query_params).expect("failed to build authorize URL");

        authorize_url.into()
    }

    pub fn finalize<S>(self, access_token: S, state: &str) -> Result<ImplicitGrantUserClient>
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
            access_token,
            http_client: self.http_client,
        })
    }
}

impl ImplicitGrantUserClientBuilder {
    pub(super) fn new(
        redirect_uri: String,
        spotify_client_ref: Arc<SpotifyClientRef>,
        http_client: AsyncClient,
    ) -> Self {
        Self {
            redirect_uri,
            state: None,
            scopes: None,
            show_dialog: false,

            spotify_client_ref,
            http_client,
        }
    }

    pub fn state<S>(self, state: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            state: Some(state.into()),
            ..self
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

    pub fn build(self) -> IncompleteImplicitGrantUserClient {
        let state = if let Some(state) = self.state {
            state
        } else {
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(RANDOM_STATE_LENGTH)
                .map(char::from)
                .collect()
        };

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

impl private::Sealed for ImplicitGrantUserClient {}

impl private::BuildHttpRequest for ImplicitGrantUserClient {
    fn build_http_request(&self, method: Method, url: Url) -> RequestBuilder {
        self.http_client
            .request(method, url)
            .bearer_auth(self.access_token.as_str())
    }
}

#[async_trait]
impl<'a> ScopedClient<'a> for ImplicitGrantUserClient {}

#[async_trait]
impl<'a> UnscopedClient<'a> for ImplicitGrantUserClient {}

#[async_trait]
impl private::AccessTokenExpiry for ImplicitGrantUserClient {
    async fn handle_access_token_expired(&self) -> Result<private::AccessTokenExpiryResult> {
        Ok(private::AccessTokenExpiryResult::Inapplicable)
    }
}
