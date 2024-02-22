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
        .scopes([Scope::UserReadPlaybackState, Scope::UserReadCurrentlyPlaying])
        .show_dialog(true)
        .build();

    let authorize_url = incomplete_auth_code_client.get_authorize_url();
    println!("Authorize URL: {authorize_url}");

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
    // let user_client = spotify_client
    //     .authorization_code_client_with_refresh_token("refresh token")
    //     .await
    //     .unwrap();

    // there are two way to get the currently playing item: .playback_state(), and .currently_playing_item()
    // playback state returns a superset of the currently playing item and includes information about the device that is
    // playing and the player's shuffle and repeat states

    // .playback_state()

    let playback_state = user_client.playback_state().send_async().await.unwrap();
    // println!("{playback_state:#?}");

    if let Some(playback_state) = playback_state {
        println!(
            "Repeat: {:?}, shuffle: {}",
            playback_state.repeat_state(),
            playback_state.shuffle_state()
        );

        println!(
            "Playing on: {} ({}), which is a {:?}",
            playback_state.device().name(),
            playback_state.device().id(),
            playback_state.device().device_type()
        );

        if let Some(item) = playback_state.currently_playing_item().public_playing_item() {
            if let PlayingType::Track(full_track) = item.item() {
                println!(
                    "{} - {} ({}) [{}] - {:?}/{:?}",
                    full_track.name(),
                    full_track.artists().first().unwrap().name(),
                    full_track.album().name(),
                    full_track.id(),
                    item.progress(),
                    full_track.duration(),
                );
            } else {
                println!("Playing item isn't a track");
            }
        } else {
            println!("Playing item isn't available (private session?)");
        }
    } else {
        println!("Nothing playing");
    }

    // .currently_playing_item()

    let currently_playing_item = user_client.currently_playing_item().send_async().await.unwrap();

    if let Some(currently_playing_item) = currently_playing_item {
        if let Some(item) = currently_playing_item.public_playing_item() {
            if let PlayingType::Track(full_track) = item.item() {
                println!(
                    "{} - {} ({}) [{}] - {:?}/{:?}",
                    full_track.name(),
                    full_track.artists().first().unwrap().name(),
                    full_track.album().name(),
                    full_track.id(),
                    item.progress(),
                    full_track.duration(),
                );
            } else {
                println!("Playing item isn't a track");
            }
        } else {
            println!("Playing item isn't available (private session?)");
        }
    } else {
        println!("Nothing playing");
    }
}
