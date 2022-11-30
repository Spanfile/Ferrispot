# Unreleased
- **Breaking**: `id`, `uri` and `url` in `IdTrait` have been renamed to `as_str`, `as_uri` and `as_url` respectively.
- **New**: Add `IdTrait::as_borrowed()` that returns a new `Id` borrowing from the given Id which must not outlive the given `Id`, essentially acting as a reference without being a reference (`&Id`).
- **New**: Add `RelinkedTrackEquality` trait that allows comparing tracks when Spotify track relinking is applied to either or both of them.
- **Changed**: Change equality comparison method for tracks, albums and objects: only compare their IDs for full and partial items, and all the fields for local items.
- **Changed**: The unscoped `.track()` call returns a `NonexistentTrack` error for 404 responses.
- **Fixed**: Properly build URL for single track lookup.

# 0.2.1
- Fix authorization code user client with PKCE freezing while refreshing the access token.
- Remove chance of authorization code user client state parameter being an empty string.

# 0.2.0
- A functional object model for the endpoints so far.
- Synchronous (blocking) API, available with the `sync` crate feature.

# 0.1.0
- Initial release.