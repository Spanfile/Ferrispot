use crate::model::error::AuthenticationErrorKind;
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
    HttpError(#[from] reqwest::Error),
}
