#![cfg_attr(docsrs, feature(doc_auto_cfg))]

//! A wrapper for the Spotify Web API that (hopefully) doesn't suck (too much) (I think).
//!
//! A lot of the functionality is largely opinionated for my own use but I'm trying to make the library ergonimic and
//! efficient to use. So far only the endpoints I care about are implemented, but if you need some endpoints
//! implemented, feel free to open an issue.
//!
//! # Features
//!
//! - Type-safe clients and model.
//! - Asynchronous and synchronous (blocking) clients.
//! - Every OAuth authorization flow Spotify supports is implemented.
//! - Supports multiple simultaneous user clients.
//! - Automatically refreshes access tokens when they expire, where applicable.
//! - Reacts to API rate limits using either Tokio's or async-std's sleep functions at your discretion when using an
//!   asynchronous client. Synchronous clients block the running thread.
//!
//! # Usage
//!
//! See the module-level docs for [client] for instructions how to use the various Spotify clients.
//!
//! # Feature flags
//!
//! - `async` (default): enable the asynchronous API.
//! - `sync`: enable the synchronous API.
//!   - In case neither APIs are enabled (`default-features = false`), the crate only includes the object model
//!     structure with minimal dependencies on `serde` and `thiserror`.
//! - `tokio_sleep` (default): react to API rate limits using Tokio's sleep function.
//! - `async_std_sleep`: react to API rate limits using async-std's sleep function.
//!   - In case both `tokio_sleep` and `async_std_sleep` are enabled, Tokio's sleep function will be used.
//!   - In case neither are enabled, the library will return a [rate limit error](crate::error::Error::RateLimit) when
//!     it occurs.
//!   - These features are meaningless unless the `async` feature is also enabled.

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

    #[cfg(feature = "async")]
    pub use crate::client::{AccessTokenRefreshAsync, ScopedAsyncClient, UnscopedAsyncClient};
    #[cfg(feature = "sync")]
    pub use crate::client::{AccessTokenRefreshSync, ScopedSyncClient, UnscopedSyncClient};
    pub use crate::{
        model::{
            album::{CommonAlbumInformation, FullAlbumInformation, NonLocalAlbumInformation},
            artist::{CommonArtistInformation, FullArtistInformation, NonLocalArtistInformation},
            id::{IdFromBare, IdFromKnownKind, IdTrait},
            search::ToTypesString,
            track::{CommonTrackInformation, FullTrackInformation, NonLocalTrackInformation, RelinkedTrackEquality},
        },
        scope::ToScopesString,
    };
}
