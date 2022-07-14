use super::private::ClientBase;
use async_trait::async_trait;

/// All scoped Spotify endpoints.
///
/// The functions in this trait require user authentication, since they're specific to a certain user.
/// [AuthorizationCodeUserClient] and [ImplicitGrantUserClient] implement this trait.
///
/// [AuthorizationCodeUserClient]: crate::client::AuthorizationCodeUserClient
/// [ImplicitGrantUserClient]: crate::client::ImplicitGrantUserClient
#[async_trait]
pub trait ScopedClient: ClientBase {}
