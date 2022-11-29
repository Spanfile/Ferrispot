# Unreleased
- Change equality comparison method for tracks, albums and objects: only compare their IDs for full and partial items, and all the fields for local items.
- Introduce method of comparing tracks when Spotify track relinking is applied to either or both of them.

# 0.2.1
- Fix authorization code user client with PKCE freezing while refreshing the access token.
- Remove chance of authorization code user client state parameter being an empty string.

# 0.2.0
- A functional object model for the endpoints so far.
- Synchronous (blocking) API, available with the `sync` crate feature.

# 0.1.0
- Initial release.