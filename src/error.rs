use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("The given state does not match the original state")]
    AuthorizationCodeStateMismatch,

    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
}
