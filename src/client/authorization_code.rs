use super::{
    private::{ClientBase, UserAuthenticatedClient},
    ACCOUNTS_API_TOKEN_ENDPOINT, ACCOUNTS_AUTHORIZE_ENDPOINT, PKCE_VERIFIER_LENGTH, RANDOM_STATE_LENGTH,
};
use crate::{
    error::{Error, Result},
    scope::{Scope, ToScopesString},
};

use async_trait::async_trait;
use futures::lock::Mutex;
use log::debug;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{Client as AsyncClient, IntoUrl, Url};
use serde::Deserialize;
use sha2::Digest;
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
    client_id: Option<String>,
}

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
                client_id,
            }),
            http_client,
        })
    }

    pub async fn refresh_access_token(&self) -> Result<()> {
        let refresh_token = self.inner.refresh_token.lock().await;

        debug!(
            "Attempting to refresh authorization code flow access token with refresh token: {}",
            *refresh_token
        );

        let mut token_request_form = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
        ];

        let response = self
            .http_client
            .post(ACCOUNTS_API_TOKEN_ENDPOINT)
            .form(if let Some(client_id) = self.inner.client_id.as_deref() {
                token_request_form.push(("client_id", client_id));
                &token_request_form
            } else {
                &token_request_form
            })
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

        let token_response: AuthorizeUserTokenResponse = response.json().await?;
        debug!("Got token response for authorization code flow: {:?}", token_response);

        Ok(AuthorizationCodeUserClient {
            inner: Arc::new(AuthorizationCodeUserClientRef {
                access_token: Mutex::new(token_response.access_token),
                refresh_token: Mutex::new(token_response.refresh_token),
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
        I: IntoIterator<Item = Scope>,
    {
        Self {
            scopes: Some(scopes.to_scopes_string()),
            ..self
        }
    }

    pub fn show_dialog(self, show_dialog: bool) -> Self {
        Self { show_dialog, ..self }
    }

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

impl UserAuthenticatedClient for AuthorizationCodeUserClient {}

#[async_trait]
impl ClientBase for AuthorizationCodeUserClient {
    async fn build_http_request<U>(&self, method: reqwest::Method, url: U) -> reqwest::RequestBuilder
    where
        U: IntoUrl + Send,
    {
        let access_token = self.inner.access_token.lock().await;
        self.http_client.request(method, url).bearer_auth(access_token.as_str())
    }
}
