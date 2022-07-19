//! Contains the [AuthorizationCodeUserClient](AuthorizationCodeUserClient) and its builder structs.
//!
//! # Usage
//!
//! A new [AuthorizationCodeUserClient] can be built with the [`authorization_code_client`-function in
//! SpotifyClientWithSecret](crate::client::SpotifyClientWithSecret::authorization_code_client).
//!
//! ```
//! // TODO
//! ```

use super::{
    private, AccessTokenRefresh, ACCOUNTS_API_TOKEN_ENDPOINT, ACCOUNTS_AUTHORIZE_ENDPOINT, PKCE_VERIFIER_LENGTH,
    RANDOM_STATE_LENGTH,
};
use crate::{
    error::{Error, Result},
    model::error::AuthenticationErrorKind,
    scope::ToScopesString,
};

use async_trait::async_trait;
use log::debug;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{Client as AsyncClient, Method, RequestBuilder, Url};
use serde::Deserialize;
use sha2::Digest;
use std::sync::{Arc, RwLock};

/// A client that uses the authorization code flow to authenticate an user with Spotify. May optionally use PKCE if the
/// client secret is not available. See the [module-level docs](self) for more information.
///
/// Implements all the [scoped](crate::client::ScopedClient) and [unscoped endpoints](crate::client::UnscopedClient).
#[derive(Debug, Clone)]
pub struct AuthorizationCodeUserClient {
    inner: Arc<AuthorizationCodeUserClientRef>,
    http_client: AsyncClient,
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
/// directing the user to the [authorize URL](IncompleteAuthorizationCodeUserClient::authorize) and retrieving an
/// authorization code and a state parameter from the redirect callback URL.
#[derive(Debug)]
pub struct IncompleteAuthorizationCodeUserClient {
    client_id: String,
    redirect_uri: String, // TODO: figure if this can be &'a str instead
    state: String,
    scopes: Option<String>,
    show_dialog: bool,
    pkce_verifier: Option<String>,

    http_client: AsyncClient,
}

/// Builder for [AuthorizationCodeUserClient].
#[derive(Debug)]
pub struct AuthorizationCodeUserClientBuilder {
    client_id: String,
    redirect_uri: String, // TODO: figure if this can be &'a str instead
    state: Option<String>,
    scopes: Option<String>,
    show_dialog: bool,
    pkce_verifier: Option<String>,

    http_client: AsyncClient,
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

impl AuthorizationCodeUserClient {
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

        let mut token_request_form = vec![("grant_type", "refresh_token"), ("refresh_token", &refresh_token)];

        let response = http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(if let Some(client_id) = client_id.as_deref() {
                token_request_form.push(("client_id", client_id));
                &token_request_form
            } else {
                &token_request_form
            })
            .send()
            .await?;

        let response = super::extract_authentication_error(response).await.map_err(|err| {
            if let Error::UnhandledAuthenticationError(AuthenticationErrorKind::InvalidGrant, description) = err {
                Error::InvalidRefreshToken(description)
            } else {
                err
            }
        })?;

        let token_response: RefreshUserTokenResponse = response.json().await?;
        debug!(
            "Got token response for refreshing authorization code flow tokens: {:?}",
            token_response
        );

        let refresh_token = if let Some(refresh_token) = token_response.refresh_token {
            refresh_token
        } else {
            refresh_token.to_owned()
        };

        Ok(Self {
            inner: Arc::new(AuthorizationCodeUserClientRef {
                access_token: RwLock::new(token_response.access_token),
                refresh_token: RwLock::new(refresh_token),
                client_id,
            }),
            http_client,
        })
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
}

impl IncompleteAuthorizationCodeUserClient {
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

            // while both these branches end the same way, this one borrows the pkce_challenge string so the URL must be
            // built before the string falls out of scope
            Url::parse_with_params(ACCOUNTS_AUTHORIZE_ENDPOINT, &query_params).expect("failed to build authorize URL")
        } else {
            Url::parse_with_params(ACCOUNTS_AUTHORIZE_ENDPOINT, &query_params).expect("failed to build authorize URL")
        };

        authorize_url.into()
    }

    /// Finalize this client with a code and a state from the callback URL query the user was redirected to after they
    /// approved the application and return an usable [AuthorizationCodeUserClient].
    ///
    /// This function will use the authorization code to request an access and a refresh token from Spotify. If the
    /// originally generated state does not match the `state` parameter, the function will return an
    /// [AuthorizationCodeStateMismatch-error](Error::AuthorizationCodeStateMismatch).
    pub async fn finalize(self, code: &str, state: &str) -> Result<AuthorizationCodeUserClient> {
        debug!(
            "Attempting to finalize authorization code flow user client with code: {} and state: {}",
            code, state
        );

        if state != self.state {
            return Err(Error::AuthorizationCodeStateMismatch);
        }

        let mut token_request_form = vec![
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("code", code),
        ];

        let response = self
            .http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(if let Some(pkce_verifier) = self.pkce_verifier.as_deref() {
                debug!("Requesting access and refresh tokens for authorization code flow with PKCE");

                token_request_form.extend([("client_id", self.client_id.as_str()), ("code_verifier", pkce_verifier)]);
                &token_request_form
            } else {
                debug!("Requesting access and refresh tokens for authorization code flow");
                &token_request_form
            })
            .send()
            .await?;

        let response = super::extract_authentication_error(response).await.map_err(|err| {
            if let Error::UnhandledAuthenticationError(AuthenticationErrorKind::InvalidGrant, _) = err {
                Error::InvalidAuthorizationCode
            } else {
                err
            }
        })?;

        let token_response: AuthorizeUserTokenResponse = response.json().await?;
        debug!("Got token response for authorization code flow: {:?}", token_response);

        Ok(AuthorizationCodeUserClient {
            inner: Arc::new(AuthorizationCodeUserClientRef {
                access_token: RwLock::new(token_response.access_token),
                refresh_token: RwLock::new(token_response.refresh_token),
                // from here on out, using PKCE only requires us supplying our client ID when refreshing the access
                // token. if the PKCE verifier is used, include the client ID
                client_id: self.pkce_verifier.and(Some(self.client_id)),
            }),
            http_client: self.http_client,
        })
    }
}

impl AuthorizationCodeUserClientBuilder {
    pub(super) fn new(redirect_uri: String, client_id: String, http_client: AsyncClient) -> Self {
        Self {
            client_id,
            redirect_uri,
            state: None,
            scopes: None,
            show_dialog: false,
            pkce_verifier: None,

            http_client,
        }
    }

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

    // TODO: I'm not sure there's a reason to let the user specify the state string themselves
    // pub fn state<S>(self, state: S) -> Self
    // where
    //     S: Into<String>,
    // {
    //     Self {
    //         state: Some(state.into()),
    //         ..self
    //     }
    // }

    /// Specify the [OAuth authorization scopes](crate::Scope) that the user is asked to grant for the application.
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
    pub fn build(self) -> IncompleteAuthorizationCodeUserClient {
        let state = if let Some(state) = self.state {
            state
        } else {
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(RANDOM_STATE_LENGTH)
                .map(char::from)
                .collect()
        };

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

impl private::Sealed for AuthorizationCodeUserClient {}
impl private::UserAuthenticatedClient for AuthorizationCodeUserClient {}

impl private::BuildHttpRequest for AuthorizationCodeUserClient {
    fn build_http_request(&self, method: Method, url: Url) -> RequestBuilder {
        let access_token = self.inner.access_token.read().expect("access token rwlock poisoned");
        self.http_client.request(method, url).bearer_auth(access_token.as_str())
    }
}

#[async_trait]
impl AccessTokenRefresh for AuthorizationCodeUserClient {
    async fn refresh_access_token(&self) -> Result<()> {
        // build and send the request this way to not hold the non-async RwLockReadGuard across await points
        let response = {
            let refresh_token = self.inner.refresh_token.read().expect("refresh token rwlock poisoned");

            debug!(
                "Attempting to refresh authorization code flow access token with refresh token: {}",
                *refresh_token
            );

            let mut token_request_form = vec![
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.as_str()),
            ];

            self.http_client.post(ACCOUNTS_API_TOKEN_ENDPOINT).form(
                if let Some(client_id) = self.inner.client_id.as_deref() {
                    token_request_form.push(("client_id", client_id));
                    &token_request_form
                } else {
                    &token_request_form
                },
            )
        }
        .send()
        .await?;

        let response = super::extract_authentication_error(response).await.map_err(|err| {
            if let Error::UnhandledAuthenticationError(AuthenticationErrorKind::InvalidGrant, description) = err {
                Error::InvalidRefreshToken(description)
            } else {
                err
            }
        })?;

        let token_response: RefreshUserTokenResponse = response.json().await?;
        debug!(
            "Got token response for refreshing authorization code flow tokens: {:?}",
            token_response
        );

        *self.inner.access_token.write().expect("access token rwlock poisoned") = token_response.access_token;

        if let Some(refresh_token) = token_response.refresh_token {
            *self.inner.refresh_token.write().expect("refresh token rwlock poisoned") = refresh_token;
        }

        Ok(())
    }
}

#[async_trait]
impl private::AccessTokenExpiry for AuthorizationCodeUserClient {
    async fn handle_access_token_expired(&self) -> Result<private::AccessTokenExpiryResult> {
        self.refresh_access_token().await?;
        Ok(private::AccessTokenExpiryResult::Ok)
    }
}
