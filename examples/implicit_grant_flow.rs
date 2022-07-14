use dotenvy::dotenv;
use ferrispot::client::SpotifyClientBuilder;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let spotify_client =
        SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment")).build();

    let incomplete_implicit_grant_client = spotify_client
        .implicit_grant_client("http://localhost/callback")
        .build();

    let authorize_url = incomplete_implicit_grant_client.get_authorize_url();

    println!("Authorize URL: {}", authorize_url);

    let mut access_token = String::new();
    let mut state = String::new();

    println!("Access token:");
    std::io::stdin().read_line(&mut access_token).unwrap();

    println!("State:");
    std::io::stdin().read_line(&mut state).unwrap();

    let user_client = incomplete_implicit_grant_client
        .finalize(access_token.trim(), state.trim())
        .await
        .expect("failed to finalize implicit grant flow client");
}
