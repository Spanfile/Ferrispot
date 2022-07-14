use dotenvy::dotenv;
use ferrispot::client::{SpotifyClientBuilder, UnscopedClient};

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

    #[allow(unused_variables)]
    let one_track = spotify_client.track("3mXLyNsVeLelMakgpGUp1f").await.unwrap();

    #[allow(unused_variables)]
    let multiple_tracks = spotify_client
        .tracks([
            "3mXLyNsVeLelMakgpGUp1f",
            "367IrkRR4wk5WtSL41rONn",
            "1GxzaUNoSvzNqL4JB9ztXq",
        ])
        .await
        .unwrap();
}
