use crate::model::{error::AuthenticationErrorKind, ItemType};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("The given state does not match the original state")]
    AuthorizationCodeStateMismatch,
    #[error("The access token expired")]
    AccessTokenExpired,
    #[error("The refresh token is invalid: {0}. The user should be reauthorized")]
    InvalidRefreshToken(String),
    #[error("The authorization code is invalid")]
    InvalidAuthorizationCode,
    #[error("The client ID and/or secret is invalid")]
    InvalidClient,
    #[error("Request rate limit hit; retry after {0} seconds")]
    RateLimit(u64),
    #[error("The required scope for the endpoint hasn't been granted by the user")]
    MissingScope,
    #[error(
        "The endpoint is forbidden. The user likely removed the application's access to their account. Any future \
         calls with this client will likely fail. The user should be reauthorized."
    )]
    Forbidden,

    #[error(
        "Missing or invalid Retry-After header in 429 rate-limit response. This is likely an issue on Spotify's side"
    )]
    InvalidRateLimitResponse,

    #[error("Unhandled authentication error: {0:?}: {1}")]
    UnhandledAuthenticationError(AuthenticationErrorKind, String),
    #[error("Unhandled API error {0}: {1}")]
    UnhandledSpotifyError(u16, String),

    #[error(transparent)]
    InvalidSpotifyId(#[from] IdError),

    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IdError {
    #[error("Invalid item type: {0}")]
    InvalidItemType(String),
    #[error("Wrong item type in ID ({0:?})")]
    WrongItemType(ItemType),
    /// The ID in the input does not look like a valid Spotify ID. The ID may still be nonexistent in Spotify even if
    /// it looks valid.
    #[error("Invalid ID: {0}")]
    InvalidId(String),
    #[error("Malformed string: {0}")]
    MalformedString(String),
}
