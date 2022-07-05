use dotenvy::dotenv;
use ferrispot::SpotifyClientBuilder;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let spotify_client =
        SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment"))
            .client_secret(std::env::var("CLIENT_SECRET").expect("Spotify client secret not in environment"))
            .build()
            .await
            .expect("failed to build Spotify client");
}
