pub(crate) mod authorization_code;
pub(crate) mod implicit_grant;
pub(crate) mod scoped;
pub(crate) mod unscoped;

mod private {
    pub trait ClientBase {
        fn get_http_client(&self) -> &reqwest::Client;
    }
}

use self::{
    authorization_code::{AuthorizationCodeUserClient, AuthorizationCodeUserClientBuilder},
    implicit_grant::ImplicitGrantUserClientBuilder,
};
use crate::error::Result;

use futures::lock::Mutex;
use log::debug;
use reqwest::{header, Client as AsyncClient};
use serde::Deserialize;
use std::sync::Arc;

const RANDOM_STATE_LENGTH: usize = 16;
const PKCE_VERIFIER_LENGTH: usize = 128; // maximum Spotify allows

const AUTHORIZE_ENDPOINT: &str = "https://accounts.spotify.com/authorize";
const API_TOKEN_ENDPOINT: &str = "https://accounts.spotify.com/api/token";

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
    // client_secret: String,
    access_token: Mutex<String>,
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
    pub async fn refresh_access_token(&self) -> Result<()> {
        debug!("Refreshing access token for client credentials flow");
        let token_request_form = &[("grant_type", "client_credentials")];

        let response = self
            .http_client
            .post(API_TOKEN_ENDPOINT)
            .form(token_request_form)
            .send()
            .await?;

        let token_response: ClientTokenResponse = response.json().await?;
        debug!("Got token response for client credentials flow: {:?}", token_response);

        *self.inner.access_token.lock().await = token_response.access_token;

        Ok(())
    }

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
                // given client ID and secret are base64-encoded
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
            .post(API_TOKEN_ENDPOINT)
            .form(token_request_form)
            .send()
            .await?;

        let token_response: ClientTokenResponse = response.json().await?;
        debug!("Got token response for client credentials flow: {:?}", token_response);

        Ok(SpotifyClientWithSecret {
            inner: Arc::new(SpotifyClientWithSecretRef {
                client_id: self.client_id,
                // client_secret: self.client_secret,
                access_token: Mutex::new(token_response.access_token),
            }),
            http_client,
        })
    }
}

fn build_authorization_header(client_id: &str, client_secret: &str) -> String {
    let auth = format!("{}:{}", client_id, client_secret);
    format!("Basic {}", base64::encode(&auth))
}
