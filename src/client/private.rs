#[cfg(feature = "async")]
mod async_client {
    use std::ops::Deref;

    #[derive(Clone)]
    pub struct AsyncClient(pub(crate) reqwest::Client);

    impl super::HttpClient for AsyncClient {
        fn new() -> Self {
            Self(reqwest::Client::new())
        }
    }

    impl Deref for AsyncClient {
        type Target = reqwest::Client;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

#[cfg(feature = "sync")]
mod sync_client {
    use std::ops::Deref;

    #[derive(Clone)]
    pub struct SyncClient(pub(crate) reqwest::blocking::Client);

    impl super::HttpClient for SyncClient {
        fn new() -> Self {
            Self(reqwest::blocking::Client::new())
        }
    }

    impl Deref for SyncClient {
        type Target = reqwest::blocking::Client;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
}

use reqwest::{IntoUrl, Method};

#[cfg(feature = "async")]
pub use self::async_client::AsyncClient;
#[cfg(feature = "sync")]
pub use self::sync_client::SyncClient;
use crate::error::Result;

pub trait HttpClient {
    fn new() -> Self;
}

/// Every Spotify client implements this trait.
#[cfg(feature = "async")]
pub trait BuildHttpRequestAsync: crate::private::Sealed {
    /// Returns a new async [RequestBuilder](reqwest::RequestBuilder) with any necessary information (e.g.
    /// authentication headers) filled in. This method doesn't account for any known Spotify error responses
    /// automatically; for that you should use [send_http_request](SendHttpRequestAsync::send_http_request)
    fn build_http_request<U>(&self, method: Method, url: U) -> reqwest::RequestBuilder
    where
        U: IntoUrl;
}

/// Every Spotify client implements this trait.
#[cfg(feature = "sync")]
pub trait BuildHttpRequestSync: crate::private::Sealed {
    /// Returns a new async [RequestBuilder](reqwest::blocking::RequestBuilder) with any necessary information (e.g.
    /// authentication headers) filled in. This method doesn't account for any known Spotify error responses
    /// automatically; for that you should use [send_http_request](SendHttpRequestAsync::send_http_request)
    fn build_http_request<U>(&self, method: Method, url: U) -> reqwest::blocking::RequestBuilder
    where
        U: IntoUrl;
}

/// Every Spotify client implements this trait.
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AccessTokenExpiryAsync: crate::private::Sealed {
    // if specialisation was a thing, this function could be refactored into two generic trait impls
    async fn handle_access_token_expired(&self) -> Result<AccessTokenExpiryResult>;
}

/// Every Spotify client implements this trait.
#[cfg(feature = "sync")]
pub trait AccessTokenExpirySync: crate::private::Sealed {
    fn handle_access_token_expired(&self) -> Result<AccessTokenExpiryResult>;
}

/// Result to having tried to refresh a client's access token.
#[derive(Debug, PartialEq, Eq)]
pub enum AccessTokenExpiryResult {
    /// Refreshing the token succeeded
    Ok,
    /// Refreshing an access token is not applicable to this client
    Inapplicable,
}
