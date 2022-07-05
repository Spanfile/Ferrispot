use crate::{
    scope::ToScopesString, Error, Result, Scope, SpotifyClientWithSecretRef, API_TOKEN_ENDPOINT, AUTHORIZE_ENDPOINT,
    RANDOM_STATE_LENGTH,
};
use futures::lock::Mutex;
use log::debug;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{Client as AsyncClient, Url};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AuthorizationCodeUserClient {
    inner: Arc<AuthorizationCodeUserClientRef>,
    http_client: AsyncClient,
}

#[derive(Debug)]
struct AuthorizationCodeUserClientRef {
    access_token: Mutex<String>,
    refresh_token: Mutex<String>,
}

#[derive(Debug)]
pub struct IncompleteAuthorizationCodeUserClient {
    redirect_uri: String, // TODO: figure if this can be &'a str instead
    state: String,
    scopes: Option<String>,
    show_dialog: bool,

    spotify_client_ref: Arc<SpotifyClientWithSecretRef>,
    http_client: AsyncClient,
}

pub struct AuthorizationCodeUserClientBuilder {
    pub(crate) redirect_uri: String, // TODO: figure if this can be &'a str instead
    pub(crate) state: Option<String>,
    pub(crate) scopes: Option<String>,
    pub(crate) show_dialog: bool,

    pub(crate) spotify_client_ref: Arc<SpotifyClientWithSecretRef>,
    pub(crate) http_client: AsyncClient,
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
    pub(crate) async fn new_with_refresh_token(http_client: AsyncClient, refresh_token: String) -> Result<Self> {
        debug!(
            "Attempting to create new authorization code flow client with existng refresh token: {}",
            refresh_token
        );

        let token_request_form = &[("grant_type", "refresh_token"), ("refresh_token", &refresh_token)];

        let response = http_client
            .post(API_TOKEN_ENDPOINT)
            .form(token_request_form)
            .send()
            .await?;

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
                access_token: Mutex::new(token_response.access_token),
                refresh_token: Mutex::new(refresh_token),
            }),
            http_client,
        })
    }

    pub async fn refresh_access_token(&self) -> Result<()> {
        let refresh_token = self.inner.refresh_token.lock().await;

        debug!(
            "Attempting to authorization code flow access token with refresh token: {}",
            *refresh_token
        );

        let token_request_form = &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
        ];

        let response = self
            .http_client
            .post(API_TOKEN_ENDPOINT)
            .form(token_request_form)
            .send()
            .await?;

        let token_response: RefreshUserTokenResponse = response.json().await?;
        debug!(
            "Got token response for refreshing authorization code flow tokens: {:?}",
            token_response
        );

        *self.inner.access_token.lock().await = token_response.access_token;

        if let Some(refresh_token) = token_response.refresh_token {
            *self.inner.refresh_token.lock().await = refresh_token;
        }

        Ok(())
    }
}

impl IncompleteAuthorizationCodeUserClient {
    pub fn get_authorize_url(&self) -> String {
        let mut query_params = vec![
            ("response_type", "code"),
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
            Url::parse_with_params(AUTHORIZE_ENDPOINT, &query_params).expect("failed to build authorize URL");

        authorize_url.into()
    }

    pub async fn finalize(self, code: &str, state: &str) -> Result<AuthorizationCodeUserClient> {
        debug!(
            "Attempting to finalize authorization code flow user client with code: {} and state: {}",
            code, state
        );

        if state != self.state {
            return Err(Error::AuthorizationCodeStateMismatch);
        }

        debug!("Requesting access and refresh tokens for authorization code flow");
        let token_request_form = &[
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("code", code),
        ];

        let response = self
            .http_client
            .post(API_TOKEN_ENDPOINT)
            .form(token_request_form)
            .send()
            .await?;

        let token_response: AuthorizeUserTokenResponse = response.json().await?;
        debug!("Got token response for authorization code flow: {:?}", token_response);

        Ok(AuthorizationCodeUserClient {
            inner: Arc::new(AuthorizationCodeUserClientRef {
                access_token: Mutex::new(token_response.access_token),
                refresh_token: Mutex::new(token_response.refresh_token),
            }),
            http_client: self.http_client,
        })
    }
}

impl AuthorizationCodeUserClientBuilder {
    pub fn state<S>(self, state: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            state: Some(state.into()),
            ..self
        }
    }

    pub fn scopes<I>(self, scopes: I) -> Self
    where
        I: Iterator<Item = Scope>,
    {
        Self {
            scopes: Some(scopes.to_scopes_string()),
            ..self
        }
    }

    pub fn show_dialog(self, show_dialog: bool) -> Self {
        Self { show_dialog, ..self }
    }

    pub fn build(self) -> Result<IncompleteAuthorizationCodeUserClient> {
        let state = if let Some(state) = self.state {
            state
        } else {
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(RANDOM_STATE_LENGTH)
                .map(char::from)
                .collect()
        };

        Ok(IncompleteAuthorizationCodeUserClient {
            redirect_uri: self.redirect_uri,
            state,
            scopes: self.scopes,
            show_dialog: self.show_dialog,

            spotify_client_ref: self.spotify_client_ref,
            http_client: self.http_client,
        })
    }
}
