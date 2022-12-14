use std::time::Duration;

use dotenvy::dotenv;
use ferrispot::{
    client::SpotifyClientBuilder,
    error::Error,
    model::id::{Id, PlayableContext, PlayableItem, TrackId},
    prelude::*,
    scope::Scope,
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

    let incomplete_auth_code_client = spotify_client
        .authorization_code_client("http://localhost/callback")
        .scopes([Scope::UserModifyPlaybackState, Scope::UserReadPlaybackState])
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

    let devices = user_client
        .devices()
        .send_async()
        .await
        .expect("failed to get available devices");
    println!("Available devices: {devices:#?}");

    user_client
        .play_items([
            // the function expects an iterator of PlayableItems. single PlayableItems can be parsed from either a
            // Spotify URI or a share URL, but "bare" IDs have to be specified as being track or episode IDs, then
            // converted into PlayableItems with .into()
            PlayableItem::from_uri("spotify:track:3mXLyNsVeLelMakgpGUp1f").expect("failed to parse track URI"),
            PlayableItem::from_url("https://open.spotify.com/track/367IrkRR4wk5WtSL41rONn?si=asdasdasdasd")
                .expect("failed to parse track URL"),
            Id::<TrackId>::from_bare("1GxzaUNoSvzNqL4JB9ztXq")
                .expect("failed to parse bare ID")
                .into(),
        ])
        .send_async()
        .await
        .expect("failed to play tracks");

    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("Pause");
    user_client.pause().send_async().await.unwrap();

    // player controls may be restricted for the current playback. for example, pausing an already paused playback is
    // disallowed
    let currently_playing_item = user_client.currently_playing_item().send_async().await.unwrap();

    if let Some(currently_playing_item) = currently_playing_item {
        println!(
            "Pausing disallowed: {}",
            currently_playing_item.actions().disallows.pausing
        );
    }

    // attempting a restricted player control will return a Restricted error
    let restricted_error = user_client.pause().send_async().await.unwrap_err();
    assert!(matches!(restricted_error, Error::Restricted));

    tokio::time::sleep(Duration::from_secs(3)).await;
    println!("Resume");
    user_client.resume().send_async().await.unwrap();

    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("Next");
    user_client.next().send_async().await.unwrap();

    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("Seek");
    user_client.seek(60 * 1000u32).send_async().await.unwrap();

    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("Previous");
    user_client.previous().send_async().await.unwrap();

    tokio::time::sleep(Duration::from_secs(5)).await;
    println!("Play context");

    user_client
        .play_context(PlayableContext::from_url("https://open.spotify.com/album/4muEF5biWb506ZojGMfHb7").unwrap())
        .offset(1u32)
        .send_async()
        .await
        .unwrap();
}
