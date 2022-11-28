//! A wrapper for the Spotify Web API that (hopefully) doesn't suck (too much) (I think).
//!
//! A lot of the functionality is largely opinionated for my own use. So far only the endpoints I care about are
//! implemented.
//!
//! # Features
//!
//! - Type-safe clients and model
//! - Asynchronous (there's a base for a blocking API, coming once I bother implementing it)
//! - Every OAuth authorization flow Spotify supports is implemented
//! - Supports multiple simultaneous user clients
//! - Automatically refreshes the user access token once it expires
//! - Reacts to API rate limits using either Tokio's or async-std's sleep functions at your discretion, or not at all
//!
//! # Usage
//!
//! See the module-level docs for [client] for instructions how to use the various Spotify clients.
//!
//! # Feature flags
//!
//! - `async` (default): enable the asynchronous API
//! - `sync`: enable the synchronous API (*not implemented at this time*)
//!   - In case neither APIs are enabled (`default-features = false`), the crate only includes the object model
//!     structure with minimal dependencies on `serde` and `thiserror`.
//! - `tokio_sleep` (default): react to API rate limits using Tokio's sleep function
//! - `async_std_sleep`: react to API rate limits using async-std's sleep function
//!   - In case both `tokio_sleep` and `async_std_sleep` are enabled, Tokio's sleep function will be used
//!   - In case neither are enabled, the library will return a [rate limit error](crate::error::Error::RateLimit) when
//!     it occurs

#[cfg(any(feature = "async", feature = "sync"))]
pub mod client;

pub mod error;
pub mod model;
pub mod scope;

mod util;

pub(crate) mod private {
    pub trait Sealed {}
}

pub mod prelude {
    //! Re-exports of all common traits in the crate.
    //!
    //! A lot of various objects are similar in the crate; Spotify clients, model tracks (the song kind, not the train
    //! kind), IDs etc. Their common functionality is grouped into traits. All such traits are re-exported here for
    //! convenience.

    #[cfg(feature = "sync")]
    pub use crate::client::AccessTokenRefreshSync;
    #[cfg(feature = "async")]
    pub use crate::client::{scoped::ScopedAsyncClient, unscoped::UnscopedAsyncClient, AccessTokenRefreshAsync};
    pub use crate::{
        model::{
            album::{CommonAlbumInformation, FullAlbumInformation, NonLocalAlbumInformation},
            artist::{CommonArtistInformation, FullArtistInformation, NonLocalArtistInformation},
            id::{IdFromBare, IdFromKnownKind, IdTrait},
            search::ToTypesString,
            track::{CommonTrackInformation, FullTrackInformation, NonLocalTrackInformation},
        },
        scope::ToScopesString,
    };
}
