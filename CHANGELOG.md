# Unreleased
- **Changed**: Request builders internally uses a hash map to store query parameters. This prevents certain bugs with trying to append the same parameter multiple times.
- **Changed**: Better handling of 403 Forbidden errors to account for:
  - Restricted player control calls failing (see the [play example](examples/play.rs)).
  - Target user not having a Spotify Premium account.
- **Fixed**: Build fails with just the model enabled (`default-features = false`).

# 0.4.0

## The neat thing

Every endpoint function has been changed to return a request builder that allows for setting certain parameters for the request before sending it. The request builder may be sent with the `.send_async()` and `.send_sync()` functions on the builder. Because of this change, there are no longer separate async and sync traits for scoped and unscoped clients; they have been merged into just `ScopedClient` and `UnscopedClient`.

This change allows, for one:
  - Choosing whether or not Ferrispot automatically reacts to being rate limited per-request using the `react_to_rate_limit`-function in every request builder. Default: `true`.
  - Choosing whether or not Ferrispot automatically refreshes the client's access token when it expires, if applicable, using the `auto_refresh_access_token`-function in every request builder. Default: `true`.
  - Neater way of choosing opt-in options such as the device ID in player requests or the market country code in catalog searches.
  - More easily implementing new endpoints in the future.

## As for the rest...

- **Breaking**: `.context()` in `PublicPlayingItem` now returns `Option<Context>` to account for cases where the playing context is not publicly available.
- **Breaking**: The previously deprecated functions in Id have been removed.
- **Fixed**: The album in a relinked track object fails to deserialize.
- ... and likely something else as well I've missed, my commit messages are terrible.

# 0.3.4
- **Fixed**: Synchronous player control calls fail with an unhandled HTTP 411 error.

# 0.3.3
- **New**: Add support for user IDs and collection URIs / URLs, which refer to a user's Liked Songs playlist.

# 0.3.2
- **New**: The following new endpoints have been implemented:
    - Scoped: `next` ([Skip to next](https://developer.spotify.com/documentation/web-api/reference/#/operations/skip-users-playback-to-next-track))
    - Scoped: `previous` ([Skip to previous](https://developer.spotify.com/documentation/web-api/reference/#/operations/skip-users-playback-to-previous-track))
    - Scoped: `seek` ([Seek to position](https://developer.spotify.com/documentation/web-api/reference/#/operations/seek-to-position-in-currently-playing-track))
- **Fixed**: Player control calls (pause, resume, volume etc.) fail with an unhandled HTTP 411 error.

# 0.3.1
- **Fixed**: Access token refreshing uses the wrong credentials and fails with `InvalidClient` (#2).

# 0.3.0
- **Breaking**: The various `.id()` functions in model objects have been changed to return an `Id` struct that borrow from the models respectively.
- **New**: `id()`, `uri()` and `url()` in `IdTrait` have been deprecated and new functions `as_str()`, `as_uri()` and `as_url()` respectively have been added. Their functionality is identical, but the new functions better describe their return values and fit the naming scheme better. The old functions will be removed in a future release.
- **New**: The scoped function `currently_playing_track()` has been deprecated and a new function `currently_playing_item()` has been added. Its functionality is identical, but the new function more accurately describes that the currently playing item may be something other than a track. The old function will be removed in a future release.
- **New**: Add `IdTrait::as_borrowed()` that returns a new `Id` borrowing from the given Id which must not outlive the given `Id`, essentially acting as a reference without being a reference (`&Id`).
- **New**: Add `RelinkedTrackEquality` trait that allows comparing tracks when Spotify track relinking is applied to either or both of them.
- **New**: Add ability to compare full, partial and local tracks, albums and artists between each other of the same kind. Item IDs will be compared for full and partial items, otherwise all available fields are compared for local items.
- **Changed**: Change equality comparison method for tracks, albums and objects: only compare their IDs for full and partial items, and all the fields for local items.
- **Changed**: The unscoped `.track()` call returns a `NonexistentTrack` error for 404 responses.
- **Changed**: `Device` equality is now based only on its ID.
- **Fixed**: Properly build URL for single track lookup.

# 0.2.1
- Fix authorization code user client with PKCE freezing while refreshing the access token.
- Remove chance of authorization code user client state parameter being an empty string.

# 0.2.0
- A functional object model for the endpoints so far.
- Synchronous (blocking) API, available with the `sync` crate feature.

# 0.1.0
- Initial release.