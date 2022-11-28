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

    spotify_client
        .refresh_access_token()
        .await
        .expect("failed to refresh access token");
}
