use std::fmt::Display;

pub(crate) trait ToScopesString
where
    Self: IntoIterator<Item = Scope>,
{
    fn to_scopes_string(self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    UgcImageUpload,
    UserModifyPlaybackState,
    UserReadPlaybackState,
    UserReadCurrentlyPlaying,
    UserFollowModify,
    UserFollowRead,
    UserReadRecentlyPlayed,
    UserReadPlaybackPosition,
    UserTopRead,
    PlaylistReadCollaborative,
    PlaylistModifyPublic,
    PlaylistReadPrivate,
    PlaylistModifyPrivate,
    AppRemoteControl,
    Streaming,
    UserReadEmail,
    UserReadPrivate,
    UserLibraryModify,
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
