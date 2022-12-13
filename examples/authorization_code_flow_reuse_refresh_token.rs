use dotenvy::dotenv;
use ferrispot::{
    client::SpotifyClientBuilder,
    model::{id::Id, playback::PlayingType},
    prelude::*,
};

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

    let user_client = spotify_client
        .authorization_code_client_with_refresh_token("a refresh token from a previous session")
        .await
        .expect("failed to build authorization code client");

    // all scoped endpoints are now available...
    let playback_state = user_client.playback_state().send_async().await.unwrap();
    if let Some(playing_item) =
        playback_state.and_then(|playback| playback.take_currently_playing_item().take_public_playing_item())
    {
        if let PlayingType::Track(track) = playing_item.item() {
            println!(
                "Now playing: {} - {}",
                track.name(),
                track.artists().first().unwrap().name()
            );
        }
    }

    // ... as well as all unscoped endpoints
    let one_track = user_client
        .track(Id::from_bare("2PoYyfBkedDBPGAh0ZUoHW").unwrap())
        .send_async()
        .await
        .unwrap();

    println!(
        "{} - {} ({})",
        one_track.name(),
        one_track.artists().first().unwrap().name(),
        one_track.album().name()
    );
}
