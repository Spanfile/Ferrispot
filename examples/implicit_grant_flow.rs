use dotenvy::dotenv;
use ferrispot::{client::SpotifyClientBuilder, model::playback::PlayingType, prelude::*};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let spotify_client =
        SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment")).build();

    let incomplete_implicit_grant_client = spotify_client
        .implicit_grant_client("http://localhost/callback")
        .build();

    let authorize_url = incomplete_implicit_grant_client.get_authorize_url();

    println!("Authorize URL: {}", authorize_url);

    let mut access_token = String::new();
    let mut state = String::new();

    println!("Access token:");
    std::io::stdin().read_line(&mut access_token).unwrap();

    println!("State:");
    std::io::stdin().read_line(&mut state).unwrap();

    let user_client = incomplete_implicit_grant_client
        .finalize(access_token.trim(), state.trim())
        .expect("failed to finalize implicit grant flow client");

    let playback_state = user_client.playback_state().await.unwrap();

    if let Some(playback_state) = playback_state {
        if let Some(item) = playback_state.currently_playing_item().public_playing_item() {
            match item.item() {
                PlayingType::Track(full_track) => println!(
                    "{} - {} ({})",
                    full_track.name(),
                    full_track.artists().first().unwrap().name(),
                    full_track.album().name()
                ),
            }
        }
    }
}
