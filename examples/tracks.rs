use dotenvy::dotenv;
use ferrispot::{self, client::SpotifyClientBuilder, model::id::Id, prelude::*};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let spotify_client =
        SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment"))
            .client_secret(std::env::var("CLIENT_SECRET").expect("Spotify client secret not in environment"))
            // a synchronous (blocking) client may be built with .build_sync() if the "sync" crate feature is enabled
            .build_async()
            .await
            .expect("failed to build Spotify client");

    let one_track = spotify_client
        .track(Id::from_bare("0871AdnvzzSGr5XdTJaDHC").unwrap(), None)
        .await
        .unwrap();

    println!(
        "{} - {} ({})",
        one_track.name(),
        one_track.artists().first().unwrap().name(),
        one_track.album().name()
    );

    let multiple_tracks = spotify_client
        .tracks(
            [
                Id::from_bare("3mXLyNsVeLelMakgpGUp1f").unwrap(),
                Id::from_bare("367IrkRR4wk5WtSL41rONn").unwrap(),
                Id::from_bare("1GxzaUNoSvzNqL4JB9ztXq").unwrap(),
                // nonexistent tracks are omitted from the result without returning an error
                Id::from_bare("aaaaaaaaaaaaaaaaaaaaaa").unwrap(),
            ],
            None,
        )
        .await
        .unwrap();

    for track in multiple_tracks {
        println!(
            "{} - {} ({})",
            track.name(),
            track.artists().first().unwrap().name(),
            track.album().name()
        );
    }

    let nonexistent_id_error = spotify_client
        .track(Id::from_bare("aaaaaaaaaaaaaaaaaaaaaa").unwrap(), None)
        .await
        .unwrap_err();

    println!("{}", nonexistent_id_error);
}
