use dotenvy::dotenv;
use ferrispot::{
    client::SpotifyClientBuilder,
    model::{id::Id, playback::PlayingType},
    prelude::*,
    scope::Scope,
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let spotify_client =
        SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment"))
            // a synchronous (blocking) client may be built with .build_sync() if the "sync" crate feature is enabled
            .build_async();

    let incomplete_implicit_grant_client = spotify_client
        .implicit_grant_client("http://localhost/callback")
        .scopes([Scope::UserReadPlaybackState])
        .show_dialog(true)
        .build();

    let authorize_url = incomplete_implicit_grant_client.get_authorize_url();

    println!("Authorize URL: {authorize_url}");

    let mut access_token = String::new();
    let mut state = String::new();

    println!("Access token:");
    std::io::stdin().read_line(&mut access_token).unwrap();

    println!("State:");
    std::io::stdin().read_line(&mut state).unwrap();

    let user_client = incomplete_implicit_grant_client
        .finalize(access_token.trim(), state.trim())
        .expect("failed to finalize implicit grant flow client");

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
