use dotenvy::dotenv;
use ferrispot::SpotifyClientBuilder;

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

    let auth_code_client = spotify_client
        .authorization_code_client_with_refresh_token("a refresh token from a previous session")
        .await
        .expect("failed to build authorization code client");
}
