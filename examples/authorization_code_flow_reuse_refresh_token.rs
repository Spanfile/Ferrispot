use dotenvy::dotenv;
use ferrispot::{client::SpotifyClientBuilder, prelude::*};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let spotify_client =
        SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment"))
            .client_secret(std::env::var("CLIENT_SECRET").expect("Spotify client secret not in environment"))
            .build_async()
            .await
            .expect("failed to build Spotify client");

    let user_client = spotify_client
        .authorization_code_client_with_refresh_token_async("a refresh token from a previous session")
        .await
        .expect("failed to build authorization code client");

    user_client
        .refresh_access_token()
        .await
        .expect("failed to refresh access token");
}
