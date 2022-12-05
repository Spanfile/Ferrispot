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