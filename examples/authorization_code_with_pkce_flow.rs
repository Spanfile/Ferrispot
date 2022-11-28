use dotenvy::dotenv;
use ferrispot::{client::SpotifyClientBuilder, prelude::*};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let spotify_client =
        SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment"))
            // a synchronous (blocking) client may be built with .build_sync() if the "sync" crate feature is enabled
            .build_async();

    let incomplete_auth_code_pkce_client = spotify_client
        .authorization_code_client_with_pkce("http://localhost/callback")
        .build();

    let authorize_url = incomplete_auth_code_pkce_client.get_authorize_url();
    println!("Authorize URL: {}", authorize_url);

    let mut code = String::new();
    let mut state = String::new();

    println!("Code:");
    std::io::stdin().read_line(&mut code).unwrap();

    println!("State:");
    std::io::stdin().read_line(&mut state).unwrap();

    let user_client = incomplete_auth_code_pkce_client
        .finalize(code.trim(), state.trim())
        .await
        .expect("failed to finalize authorization code client");

    user_client
        .refresh_access_token()
        .await
        .expect("failed to refresh access token");
}
