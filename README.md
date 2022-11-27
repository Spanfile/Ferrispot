# Ferrispot

A wrapper for the Spotify Web API that (hopefully) doesn't suck (too much) (I think).

A lot of the functionality is largely opinionated for my own use. So far only the endpoints I care about are implemented.

## Features

-   Type-safe clients and model
-   Asynchronous (there's a base for a blocking API, coming once I bother implementing it)
-   Every OAuth authorization flow Spotify supports is implemented
-   Supports multiple simultaneous user clients
-   Automatically refreshes the user access token once it expires
-   Reacts to API rate limits using either Tokio's or async-std's sleep functions at your discretion, or not at all

## Crate feature flags

-   `async` (default): enable the asynchronous API
-   `sync`: enable the synchronous API (*not implemented at this time*)
    -   In case neither APIs are enabled (`default-features = false`), the crate only includes the object model structure with minimal dependencies on `serde` and `thiserror`.
-   `tokio_sleep` (default): react to API rate limits using Tokio's sleep function
-   `async_std_sleep`: react to API rate limits using async-std's sleep function
    -   In case both `tokio_sleep` and `async_std_sleep` are enabled, Tokio's sleep function will be used
    -   In case neither are enabled, the library will return a rate limit error when it occurs

## Attribution

This crate draws a lot of inspiration from:

-   [aspotify](https://github.com/SabrinaJewson/aspotify) by Sabrina Jewson, licensed under the MIT license
-   [rspotify](https://github.com/ramsayleung/rspotify) by Ramsay Leung and Mario Ortiz Manero, licensed under the MIT license

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
