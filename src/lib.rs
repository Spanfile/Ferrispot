pub mod client;
pub mod error;
pub mod model;
pub mod scope;

mod util;

pub mod prelude {
    //! Re-exports of all common traits in the crate.
    //!
    //! A lot of various objects are similar in the crate; Spotify clients, model tracks (the song kind, not the train
    //! kind), IDs etc. Their common functionality is grouped into traits. All such traits are re-exported here for
    //! convenience.

    pub use crate::{
        client::{scoped::ScopedClient, unscoped::UnscopedClient, AccessTokenRefresh},
        model::{
            album::{CommonAlbumInformation, FullAlbumInformation, NonLocalAlbumInformation},
            artist::{CommonArtistInformation, FullArtistInformation, NonLocalArtistInformation},
            id::{IdFromBare, IdFromUri, IdFromUrl, IdTrait},
            search::ToTypesString,
            track::{CommonTrackInformation, FullTrackInformation, NonLocalTrackInformation},
        },
        scope::ToScopesString,
    };
}
