//! Abstraction over the different errors the Spotify API may return.

use serde::{de::Visitor, Deserialize};

#[cfg(any(feature = "async", feature = "sync"))]
use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct AuthenticationErrorResponse {
    pub error: AuthenticationErrorKind,
    #[serde(default)]
    pub error_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ApiErrorResponse {
    pub(crate) error: ApiError,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ApiError {
    pub status: u16,
    pub message: ApiErrorMessage,
}

// TODO: can this be made crate-public?
/// The different causes for OAuth-authentication to fail.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationErrorKind {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum ApiErrorMessage {
    PermissionsMissing,
    TokenExpired,
    NoActiveDevice,

    Other(String),
}

#[cfg(any(feature = "async", feature = "sync"))]
impl AuthenticationErrorResponse {
    pub fn into_unhandled_error(self) -> Error {
        Error::UnhandledAuthenticationError(self.error, self.error_description)
    }
}

impl<'de> Deserialize<'de> for ApiErrorMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SpotifyErrorMessageVisitor;

        impl<'de> Visitor<'de> for SpotifyErrorMessageVisitor {
            type Value = ApiErrorMessage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(v.to_owned())
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.as_str() {
                    "Permissions missing" => Ok(ApiErrorMessage::PermissionsMissing),
                    "The access token expired" => Ok(ApiErrorMessage::TokenExpired),
                    // TODO: oh god this is ugly. there's actually a "reason" field that says NO_ACTIVE_DEVICE but that
                    // field is not in every error response (because of course it isn't)
                    "Player command failed: No active device found" => Ok(ApiErrorMessage::NoActiveDevice),

                    _ => Ok(ApiErrorMessage::Other(v)),
                }
            }
        }

        deserializer.deserialize_str(SpotifyErrorMessageVisitor)
    }
}
