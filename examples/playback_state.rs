use dotenvy::dotenv;
use ferrispot::{
    client::{ScopedClient, SpotifyClientBuilder},
    model::{
        album::CommonAlbumInformation,
        artist::CommonArtistInformation,
        playback::PlayingType,
        track::{CommonTrackInformation, FullTrackInformation},
    },
    Scope,
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let spotify_client =
        SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment"))
            .client_secret(std::env::var("CLIENT_SECRET").expect("Spotify client secret not in environment"))
            .build()
            .await
            .expect("failed to build Spotify client");

    let incomplete_auth_code_client = spotify_client
        .authorization_code_client("http://localhost/callback")
        .show_dialog(true)
        .scopes([Scope::UserReadPlaybackState])
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

    // optionally, if you have a valid refresh token (with the correct scope), you may use it as such:
    // let user_client = spotify_client.authorization_code_client_with_refresh_token("refresh token").await.unwrap();

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
