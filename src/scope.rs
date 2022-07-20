//! Contains the [Scope]-enum that represents an OAuth authorization scope and various utilities surrounding it.

use std::fmt::Display;

/// Trait for converting an object to a scopes string. This is currently implemented for all iterators of
/// [Scope's](Scope).
pub trait ToScopesString {
    /// Convert `self` to a scopes string.
    fn to_scopes_string(self) -> String;
}

/// An OAuth authorization scope.
///
/// Authorization scopes are granted to the application by the user and restrict which endpoints are available to the
/// application. All [scoped endpoints](crate::client::ScopedClient) require certain scopes to be granted. You choose
/// which scopes to request in the `scopes`-functions of either the
/// [AuthorizationCodeUserClientBuilder](crate::client::authorization_code::AuthorizationCodeUserClientBuilder::scopes)
/// or the [ImplicitGrantUserClientBuilder](crate::client::implicit_grant::ImplicitGrantUserClientBuilder::scopes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Write access to user-provided images.
    UgcImageUpload,
    /// Write access to a user’s playback state.
    UserModifyPlaybackState,
    /// Read access to a user’s player state.
    UserReadPlaybackState,
    /// Read access to a user’s currently playing content.
    UserReadCurrentlyPlaying,
    /// Write/delete access to the list of artists and other users that the user follows.
    UserFollowModify,
    /// Read access to the list of artists and other users that the user follows.
    UserFollowRead,
    /// Read access to a user’s recently played tracks.
    UserReadRecentlyPlayed,
    /// Read access to a user’s playback position in a content.
    UserReadPlaybackPosition,
    /// Read access to a user's top artists and tracks.
    UserTopRead,
    /// Include collaborative playlists when requesting a user's playlists.
    PlaylistReadCollaborative,
    /// Write access to a user's public playlists.
    PlaylistModifyPublic,
    /// Read access to user's private playlists.
    PlaylistReadPrivate,
    /// Write access to a user's private playlists.
    PlaylistModifyPrivate,
    /// Remote control playback of Spotify. This scope is currently available to Spotify iOS and Android SDKs.
    AppRemoteControl,
    /// Control playback of a Spotify track. This scope is currently available to the Web Playback SDK. The user must
    /// have a Spotify Premium account.
    Streaming,
    /// Read access to user’s email address.
    UserReadEmail,
    /// Read access to user’s subscription details (type of user account).
    UserReadPrivate,
    /// Write/delete access to a user's "Your Music" library.
    UserLibraryModify,
    /// Read access to a user's library.
    UserLibraryRead,
}

impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::UgcImageUpload => write!(f, "ugc-image-upload"),
            Scope::UserModifyPlaybackState => write!(f, "user-modify-playback-state"),
            Scope::UserReadPlaybackState => write!(f, "user-read-playback-state"),
            Scope::UserReadCurrentlyPlaying => write!(f, "user-read-currently-playing"),
            Scope::UserFollowModify => write!(f, "user-follow-modify"),
            Scope::UserFollowRead => write!(f, "user-follow-read"),
            Scope::UserReadRecentlyPlayed => write!(f, "user-read-recently-played"),
            Scope::UserReadPlaybackPosition => write!(f, "user-read-playback-position"),
            Scope::UserTopRead => write!(f, "user-top-read"),
            Scope::PlaylistReadCollaborative => write!(f, "playlist-read-collaborative"),
            Scope::PlaylistModifyPublic => write!(f, "playlist-modify-public"),
            Scope::PlaylistReadPrivate => write!(f, "playlist-read-private"),
            Scope::PlaylistModifyPrivate => write!(f, "playlist-modify-private"),
            Scope::AppRemoteControl => write!(f, "app-remote-control"),
            Scope::Streaming => write!(f, "streaming"),
            Scope::UserReadEmail => write!(f, "user-read-email"),
            Scope::UserReadPrivate => write!(f, "user-read-private"),
            Scope::UserLibraryModify => write!(f, "user-library-modify"),
            Scope::UserLibraryRead => write!(f, "user-library-read"),
        }
    }
}

impl<I> ToScopesString for I
where
    I: IntoIterator<Item = Scope>,
{
    fn to_scopes_string(self) -> String {
        self.into_iter()
            .map(|scope| scope.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}
