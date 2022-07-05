mod error;
mod scope;

const RANDOM_STATE_LENGTH: usize = 16;

const AUTHORIZE_ENDPOINT: &str = "https://accounts.spotify.com/authorize";
const API_TOKEN_ENDPOINT: &str = "https://accounts.spotify.com/api/token";

pub use crate::{
    error::{Error, Result},
    scope::Scope,
};

use crate::scope::ToScopesString;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{blocking::Client as BlockingClient, header, Url};
use serde::Deserialize;

#[derive(Debug)]
pub struct SpotifyClient {
    client_id: String,
}

#[derive(Debug)]
pub struct SpotifyClientWithSecret {
    client_id: String,
    client_secret: String,
    access_token: String,

    blocking_client: Option<BlockingClient>,
}

pub struct SpotifyClientBuilder {
    client_id: String,
}

pub struct ClientSecretSpotifyClientBuilder {
    client_id: String,
    client_secret: String,
}

pub struct AuthorizeCodeFlowUserClient<'a> {
    spotify_client: &'a SpotifyClientWithSecret,
    access_token: String,
    refresh_token: String,
}

pub struct IncompleteAuthorizeCodeFlowUserClient<'a> {
    spotify_client: &'a SpotifyClientWithSecret,
    redirect_uri: String, // TODO: figure if this can be &'a str instead
    state: String,
    scopes: Option<String>,
    show_dialog: bool,
}

pub struct AuthorizeCodeFlowUserClientBuilder<'a> {
    spotify_client: &'a SpotifyClientWithSecret,
    redirect_uri: String, // TODO: figure if this can be &'a str instead
    state: Option<String>,
    scopes: Option<String>,
    show_dialog: bool,
}

#[derive(Debug, Deserialize)]
struct ClientTokenResponse {
    access_token: String,
    // token_type: String,
    // expires_in: u32
}

#[derive(Debug, Deserialize)]
struct UserTokenResponse {
    access_token: String,
    refresh_token: String,
    // scope: String,
    // expires_in: u32,
    // token_type: String,
}

impl SpotifyClient {}

impl SpotifyClientWithSecret {
    pub fn authorize_code_flow_client<S>(&self, redirect_uri: S) -> AuthorizeCodeFlowUserClientBuilder
    where
        S: Into<String>,
    {
        AuthorizeCodeFlowUserClientBuilder {
            spotify_client: self,
            redirect_uri: redirect_uri.into(),
            state: None,
            scopes: None,
            show_dialog: false,
        }
    }

    fn get_blocking_http_client(&self) -> &BlockingClient {
        self.blocking_client.as_ref().unwrap()
    }
}

impl<'a> IncompleteAuthorizeCodeFlowUserClient<'a> {
    pub fn get_authorize_url(&self) -> String {
        let mut query_params = vec![
            ("response_type", "code"),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("client_id", self.spotify_client.client_id.as_str()),
            ("state", self.state.as_str()),
        ];

        if let Some(scopes) = &self.scopes {
            query_params.push(("scope", scopes.as_str()));
        }

        if self.show_dialog {
            query_params.push(("show_dialog", "true"));
        }

        // parsing the URL fails only if the base URL is invalid, not the parameters. if this method fails, there's a
        // bug in the library
        let authorize_url =
            Url::parse_with_params(AUTHORIZE_ENDPOINT, &query_params).expect("failed to build authorize URL");

        authorize_url.into()
    }

    pub fn finalize(self, code: &str, state: &str) -> Result<AuthorizeCodeFlowUserClient<'a>> {
        if state != self.state {
            return Err(Error::AuthorizationCodeStateMismatch);
        }

        let token_request_form = &[
            ("grant_type", "authorization_code"),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("code", code),
        ];

        let http_client = self.spotify_client.get_blocking_http_client();
        let response = http_client.post(API_TOKEN_ENDPOINT).form(token_request_form).send()?;
        let token_response: UserTokenResponse = response.json()?;

        Ok(AuthorizeCodeFlowUserClient {
            spotify_client: self.spotify_client,
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
        })
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
            client_id: self.client_id,
        }
    }
}

impl ClientSecretSpotifyClientBuilder {
    fn get_blocking_http_client(&self) -> BlockingClient {
        let mut default_headers = header::HeaderMap::new();
        default_headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&build_authorization_header(&self.client_id, &self.client_secret))
                .expect("failed to insert authorization header into header map"),
        );

        BlockingClient::builder()
            .default_headers(default_headers)
            .build()
            .expect("failed to build blocking HTTP client")
    }

    pub fn build(self) -> Result<SpotifyClientWithSecret> {
        let token_request_form = &[("grant_type", "client_credentials")];

        let http_client = self.get_blocking_http_client();
        let response = http_client.post(API_TOKEN_ENDPOINT).form(token_request_form).send()?;
        let token_response: ClientTokenResponse = response.json()?;

        Ok(SpotifyClientWithSecret {
            client_id: self.client_id,
            client_secret: self.client_secret,
            access_token: token_response.access_token,
            blocking_client: Some(http_client),
        })
    }
}

impl<'a> AuthorizeCodeFlowUserClientBuilder<'a> {
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

    pub fn build(self) -> Result<IncompleteAuthorizeCodeFlowUserClient<'a>> {
        let state = if let Some(state) = self.state {
            state
        } else {
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(RANDOM_STATE_LENGTH)
                .map(char::from)
                .collect()
        };

        Ok(IncompleteAuthorizeCodeFlowUserClient {
            spotify_client: self.spotify_client,
            redirect_uri: self.redirect_uri,
            state,
            scopes: self.scopes,
            show_dialog: self.show_dialog,
        })
    }

    pub fn existing_refresh_token<S>(self, refresh_token: S) -> AuthorizeCodeFlowUserClient<'a>
    where
        S: Into<String>,
    {
        unimplemented!()
    }
}

fn build_authorization_header(client_id: &str, client_secret: &str) -> String {
    let auth = format!("{}:{}", client_id, client_secret);
    format!("Basic {}", base64::encode(&auth))
}
