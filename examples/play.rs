use dotenvy::dotenv;
use ferrispot::{
    client::{ScopedClient, SpotifyClientBuilder},
    model::{
        id::{ParseableId, PlayableItem},
        playback::Play,
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
        .scopes([Scope::UserModifyPlaybackState])
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

    user_client
        .play(
            Play::Items(
                [PlayableItem::from_uri("spotify:track:3mXLyNsVeLelMakgpGUp1f").expect("failed to parse track URI")]
                    .into(),
            ),
            None,
        )
        .await
        .expect("failed to play track");
}
