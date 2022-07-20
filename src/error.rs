//! Various error types exposed by the crate.

use crate::model::{error::AuthenticationErrorKind, ItemType};
use thiserror::Error;

/// The result type the library returns in the public-facing interface.
pub type Result<T> = std::result::Result<T, Error>;

/// Covers all errors the library may return.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// When attempting to finalize an
    /// [AuthorizationCodeUserClient](crate::client::authorization_code::AuthorizationCodeUserClient), the
    /// given state string does not match the original one in the client.
    #[error("The given state does not match the original state")]
    AuthorizationCodeStateMismatch,

    /// When attempting to finalize an
    /// [AuthorizationCodeUserClient](crate::client::authorization_code::AuthorizationCodeUserClient), the
    /// given authorization code is invalid.
    #[error("The authorization code is invalid")]
    InvalidAuthorizationCode,

    // TODO: actually let the user disable automatic token refreshing if they so wish to
    /// The access token expired and was not automatically refreshed, due to it being impossible
    /// ([ImplicitGrantUserClient](crate::client::implicit_grant::ImplicitGrantUserClient) does not support refreshing
    /// its access token), or automatic token refreshing is disabled.
    #[error("The access token expired")]
    AccessTokenExpired,

    /// The refresh token is invalid; it cannot be used to retrieve an access token. This is likely due to the user
    /// removing the application's access to their account. The error message from Spotify is included. The user should
    /// be reauthorized.
    #[error("The refresh token is invalid: {0}. The user should be reauthorized")]
    InvalidRefreshToken(String),

    /// The client credentails (ID and possible secret) are invalid.
    #[error("The client ID and/or secret is invalid")]
    InvalidClient,

    /// Request rate limit was hit. The required wait time is included.
    #[error("Request rate limit hit; retry after {0} seconds")]
    RateLimit(u64),

    /// The required scope for the endpoint being called has not been granted by the user.
    #[error("The required scope for the endpoint has not been granted by the user")]
    MissingScope,

    /// The endpoint is forbidden. This is likely due to the user removing the application's access to their account.
    /// The error message from Spotify is included. The user should be reauthorized.
    #[error(
        "The endpoint is forbidden. The user likely removed the application's access to their account. Any future \
         calls with this client will likely fail. The user should be reauthorized."
    )]
    Forbidden,

    /// No device is currently active in the user's account, or the given device could not be activated for playback.
    #[error(
        "No device is currently active in the user's account, or the given device could not be activated for playback"
    )]
    NoActiveDevice,

    /// Spotify returned a 429 Too Many Requests, but the Retry-After header could not be parsed as an integer. This is
    /// likely an issue on Spotify's side.
    #[error(
        "Missing or invalid Retry-After header in 429 rate-limit response. This is likely an issue on Spotify's side"
    )]
    InvalidRateLimitResponse,

    /// Spotify returned an authentication error we did not expect.
    #[error("Unhandled authentication error: {0:?}: {1}")]
    UnhandledAuthenticationError(AuthenticationErrorKind, String),

    /// Spotify returned an error we did not expect.
    #[error("Unhandled API error {0}: {1}")]
    UnhandledSpotifyError(u16, String),

    /// Parsing a string to a Spotify [ID](crate::model::id::Id) failed.
    #[error(transparent)]
    InvalidSpotifyId(#[from] IdError),

    /// A catch-all for errors from reqwest. Getting this error back likely means either the library isn't handling a
    /// known error, or something went wrong with sending a request or receiving a response.
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
}

/// Error type for parsing a Spotify [ID](crate::model::id::Id).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IdError {
    /// The item type in the input is not one of known Spotify [item types](crate::model::ItemType).
    #[error("Invalid item type: {0}")]
    InvalidItemType(String),

    /// The item type in the input is not the expected type for the target type.
    #[error("Wrong item type in ID ({0:?})")]
    WrongItemType(ItemType),

    /// The ID in the input does not look like a valid Spotify ID. The ID may still be nonexistent in Spotify even if
    /// it looks valid.
    #[error("Invalid ID: {0}")]
    InvalidId(String),

    /// The input string is malformed.
    #[error("Malformed string: {0}")]
    MalformedString(String),
}
