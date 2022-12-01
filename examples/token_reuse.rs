use dotenvy::dotenv;
use ferrispot::{client::SpotifyClientBuilder, model::playback::PlayingType, prelude::*, scope::Scope};

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

    let incomplete_auth_code_client = spotify_client
        .authorization_code_client("http://localhost/callback")
        .scopes([Scope::UserReadPlaybackState])
        .show_dialog(true)
        .build();

    let authorize_url = incomplete_auth_code_client.get_authorize_url();
    println!("Authorize URL: {}", authorize_url);

    let mut code = String::new();
    let mut state = String::new();

    println!("Code:");
    std::io::stdin().read_line(&mut code).unwrap();

    println!("State:");
    std::io::stdin().read_line(&mut state).unwrap();

    let user_client = incomplete_auth_code_client
        .finalize(code.trim(), state.trim())
        .await
        .expect("failed to finalize authorization code client");

    // grab the refresh token
    let refresh_token = user_client.get_refresh_token();

    // at this point you may save the token in some manner
    // imagine at this point some time passes and you're in an entirely new process

    // you can get an AuthorizationCodeClient from the Spotify client directly by reusing the previous refresh token
    let user_client = spotify_client
        .authorization_code_client_with_refresh_token(&refresh_token)
        .await
        .expect("failed to create authorization code client");

    let playback_state = user_client.playback_state().await.unwrap();

    if let Some(playback_state) = playback_state {
        if let Some(item) = playback_state.currently_playing_item().public_playing_item() {
            if let PlayingType::Track(full_track) = item.item() {
                println!(
                    "{} - {} ({})",
                    full_track.name(),
                    full_track.artists().first().unwrap().name(),
                    full_track.album().name()
                )
            }
        }
    }
}
