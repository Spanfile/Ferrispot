//! Various error types exposed by the crate.

use std::borrow::Cow;

use thiserror::Error;

use crate::model::ItemType;
#[cfg(any(feature = "async", feature = "sync"))]
use crate::model::{
    error::AuthenticationErrorKind,
    id::{AlbumId, ArtistId, EpisodeId, Id, PlaylistId, ShowId, TrackId},
};

/// The result type the library returns in the public-facing interface.
#[cfg(any(feature = "async", feature = "sync"))]
pub type Result<T> = std::result::Result<T, Error>;

/// Covers all errors the library may return.
#[cfg(any(feature = "async", feature = "sync"))]
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
    #[error("The refresh token is invalid: {0}")]
    InvalidRefreshToken(String),

    /// The client credentails (ID and possible secret) are invalid.
    #[error("The client ID and/or secret is invalid")]
    InvalidClient(String),

    /// Request rate limit was hit. The required wait time is included.
    #[error("Request rate limit hit; retry after {0} seconds")]
    RateLimit(u64),

    /// The required scope for the endpoint being called has not been granted by the user.
    #[error("The required scope for the endpoint has not been granted by the user")]
    MissingScope,

    /// The endpoint is forbidden. This is likely due to the user removing the application's access to their account.
    /// The user should be reauthorized.
    #[error("The endpoint is forbidden")]
    Forbidden,

    /// No device is currently active in the user's account, the active device didn't respond to the playback in a \
    /// timely manner or the given device could not be activated for playback.
    ///
    /// It is still possible the playback control starts after this error if, for example, the active device is slow to
    /// respond to playback requests (e.g. smartphones).
    #[error(
        "No device is currently active in the user's account, the active device didn't respond to the playback in a \
         timely manner or the given device could not be activated for playback"
    )]
    NoActiveDevice,

    /// The given track ID doesn't refer to any Spotify track.
    #[error("Nonexistent track ID: {0}")]
    NonexistentTrack(Id<'static, TrackId>),

    /// The given album ID doesn't refer to any Spotify album.
    #[error("Nonexistent album ID: {0}")]
    NonexistentAlbum(Id<'static, AlbumId>),

    /// The given track ID doesn't refer to any Spotify track.
    #[error("Nonexistent artist ID: {0}")]
    NonexistentArtist(Id<'static, ArtistId>),

    /// The given playlist ID doesn't refer to any Spotify playlist.
    #[error("Nonexistent playlist ID: {0}")]
    NonexistentPlaylist(Id<'static, PlaylistId>),

    /// The given show ID doesn't refer to any Spotify show.
    #[error("Nonexistent show ID: {0}")]
    NonexistentShow(Id<'static, ShowId>),

    /// The given episode ID doesn't refer to any Spotify episode.
    #[error("Nonexistent episode ID: {0}")]
    NonexistentEpisode(Id<'static, EpisodeId>),

    /// Spotify returned a 429 Too Many Requests, but the Retry-After header could not be parsed as an integer. This is
    /// likely an issue on Spotify's side.
    #[error("Missing or invalid Retry-After header in 429 rate-limit response")]
    InvalidRateLimitResponse,

    /// Spotify returned an authentication error we did not expect.
    #[error("Unhandled authentication error: {0:?}: {1}")]
    UnhandledAuthenticationError(AuthenticationErrorKind, String),

    /// Spotify returned an response status code we did not expect.
    #[error("Unhandled Spotify API response status code {0}")]
    UnhandledSpotifyResponseStatusCode(u16),

    /// Parsing a string to a Spotify [ID](crate::model::id::Id) failed.
    #[error(transparent)]
    InvalidSpotifyId(#[from] IdError),

    /// Converting a Spotify API response JSON into a model object failed.
    ///
    /// If the library returns this error from a standard Spotify API function call, it means there is a mismatch
    /// between Spotify's API response and the library's object model.
    #[error(transparent)]
    Conversion(#[from] ConversionError),

    /// A catch-all for errors from reqwest. Getting this error back likely means something went wrong with sending a
    /// request or receiving and decoding a response.
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
}

/// Error type for parsing a Spotify [ID](crate::model::id::Id).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum IdError {
    /// The item type in the input is not one of known Spotify [item types](crate::model::ItemType), or the item type
    /// is not applicable for the scenario.
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

/// Error when converting serialized objects into model objects fails.
#[derive(Debug)]
#[non_exhaustive]
pub struct ConversionError(pub(crate) Cow<'static, str>);

impl std::error::Error for ConversionError {}
impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "object conversion failed: {}", self.0)
    }
}
