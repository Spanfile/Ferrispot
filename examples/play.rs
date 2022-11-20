use dotenvy::dotenv;
use ferrispot::{
    client::{ScopedClient, SpotifyClientBuilder},
    model::id::{Id, PlayableItem, TrackId},
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
            .build()
            .await
            .expect("failed to build Spotify client");

    let incomplete_auth_code_client = spotify_client
        .authorization_code_client("http://localhost/callback")
        .show_dialog(true)
        .scopes([Scope::UserModifyPlaybackState, Scope::UserReadPlaybackState])
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

    let devices = user_client.devices().await.expect("failed to get available devices");
    println!("Available devices: {:#?}", devices);

    user_client
        .play_items(
            [
                // the function expects an iterator of PlayableItems. single PlayableItems can be parsed from either a
                // Spotify URI or a share URL, but "bare" IDs have to be specified as being track or episode IDs, then
                // converted into PlayableItems with .into()
                PlayableItem::from_uri("spotify:track:3mXLyNsVeLelMakgpGUp1f").expect("failed to parse track URI"),
                PlayableItem::from_url("https://open.spotify.com/track/367IrkRR4wk5WtSL41rONn?si=asdasdasdasd")
                    .expect("failed to parse track URL"),
                Id::<TrackId>::from_bare("1GxzaUNoSvzNqL4JB9ztXq")
                    .expect("failed to parse bare ID")
                    .into(),
            ],
            None,
        )
        .await
        .expect("failed to play tracks");

    // user_client
    //     .play_context(
    //         ferrispot::model::id::PlayableContext::from_url(
    //             "https://open.spotify.com/album/0tDsHtvN9YNuZjlqHvDY2P?si=E9RNAcdrSlCYmQGltzTILg",
    //         )
    //         .expect("failed to parse album URL"),
    //         None,
    //     )
    //     .await
    //     .expect("failed to play album");
}
