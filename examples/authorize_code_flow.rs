use dotenvy::dotenv;
use ferrispot::SpotifyClientBuilder;

fn main() {
    dotenv().ok();

    let client = SpotifyClientBuilder::new(std::env::var("CLIENT_ID").expect("Spotify client ID not in environment"))
        .client_secret(std::env::var("CLIENT_SECRET").expect("Spotify client secret not in environment"))
        .build()
        .expect("failed to build Spotify client");

    let incomplete_auth_code_client = client
        .authorize_code_flow_client("http://localhost/callback")
        .build()
        .expect("failed to build authorization code client");

    let authorize_url = incomplete_auth_code_client.get_authorize_url();
    println!("Authorize URL: {}", authorize_url);

    let mut code = String::new();
    let mut state = String::new();

    println!("Code:");
    std::io::stdin().read_line(&mut code).unwrap();

    println!("State:");
    std::io::stdin().read_line(&mut state).unwrap();

    let auth_code_client = incomplete_auth_code_client
        .finalize(code.trim(), state.trim())
        .expect("failed to finalize authorization code client");
}
