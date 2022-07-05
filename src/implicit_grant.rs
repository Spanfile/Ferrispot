use crate::{scope::ToScopesString, Error, Result, Scope, SpotifyClientRef, AUTHORIZE_ENDPOINT, RANDOM_STATE_LENGTH};
use log::debug;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{Client as AsyncClient, Url};
use std::sync::Arc;

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
            Url::parse_with_params(AUTHORIZE_ENDPOINT, &query_params).expect("failed to build authorize URL");

        authorize_url.into()
    }

    pub async fn finalize<S>(self, access_token: S, state: &str) -> Result<ImplicitGrantUserClient>
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
