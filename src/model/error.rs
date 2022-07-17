use crate::error::Error;
use serde::{de::Visitor, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct AuthenticationErrorResponse {
    pub error: AuthenticationErrorKind,
    pub error_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ApiErrorResponse {
    pub status: u16,
    pub message: ApiErrorMessage,
}

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

    Other(String),
}

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
                    "Token expired" => Ok(ApiErrorMessage::TokenExpired),

                    _ => Ok(ApiErrorMessage::Other(v)),
                }
            }
        }

        deserializer.deserialize_str(SpotifyErrorMessageVisitor)
    }
}
