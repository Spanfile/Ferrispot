use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("The given state does not match the original state")]
    AuthorizationCodeStateMismatch,
    #[error("The access token expired")]
    AccessTokenExpired,

    #[error(
        "Missing or invalid Retry-After header in 429 rate-limit response. This is likely an issue on Spotify's side"
    )]
    InvalidRateLimitResponse,

    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
}
